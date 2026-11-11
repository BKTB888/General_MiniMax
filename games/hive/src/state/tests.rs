use crossterm::style::SetBackgroundColor;
use general_minimax::{
    coordinate::Coordinate,
    player::{
        evals::stupid_eval,
        search::{ABSearch, alphabeta},
    },
    result::GameResult,
    state::GameState,
    utils::{perft, position},
};

use super::{
    display::MARK_BG,
    evaluation::{eval, placeable},
    *,
};
use crate::hex_grid::{GRID_MASK, hex_to_loc, loc_to_hex};

type Mlp = HiveState<true, true, true>;

fn hex(col: i8, row: i8) -> Hex {
    loc_to_hex(Coordinate(col, row))
}

fn loc(hex: Hex) -> (i8, i8) {
    let Coordinate(col, row) = hex_to_loc(hex);
    (col, row)
}

impl Mlp {
    fn insert_loc(&mut self, (col, row): (i8, i8), bug: Bug, color: Color) {
        self.insert(hex(col, row), bug, 0, color);
    }

    fn remove_loc(&mut self, (col, row): (i8, i8)) {
        self.remove(hex(col, row));
    }

    fn fill_board(&mut self, locs: &[(i8, i8)], bug: Bug) {
        for &loc in locs {
            self.insert_loc(loc, bug, Color::Black);
        }
    }
}

fn assert_placements(turns: &[Turn], expected: &[((i8, i8), Bug)]) {
    let mut actual: Vec<_> = turns
        .iter()
        .filter_map(|&turn| match turn {
            Turn::Place(hex, bug) => Some((loc(hex), bug as u8)),
            _ => None,
        })
        .collect();
    actual.sort();
    let mut expected: Vec<_> = expected.iter().map(|&(loc, bug)| (loc, bug as u8)).collect();
    expected.sort();
    assert_eq!(actual, expected);
}

fn assert_movements(turns: &[Turn], start: (i8, i8), ends: &[(i8, i8)]) {
    let mut actual: Vec<_> = turns
        .iter()
        .filter_map(|&turn| match turn {
            Turn::Move(from, to) if loc(from) == start => Some(loc(to)),
            _ => None,
        })
        .collect();
    actual.sort();
    let mut expected = ends.to_vec();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn test_gen_placement() {
    let mut board = Mlp::default();
    for i in 1..8 {
        board.remaining[0][i] = 0;
        board.remaining[1][i] = 0;
    }
    board.insert_loc((0, 0), Bug::Queen, Color::White);
    board.insert_loc((1, 0), Bug::Queen, Color::Black);
    let mut turns = Turns::new();
    board.generate_placements(&mut turns);
    assert_placements(
        &turns,
        &[((-1, -1), Bug::Queen), ((-1, 0), Bug::Queen), ((0, 1), Bug::Queen)],
    );
}

#[test]
fn test_cut_vertex() {
    let mut board = Mlp::default();
    //．．🐝🐝🐝🐝
    // ．．．🐝．🐝🐝
    //．．．．🐝🐝
    board.fill_board(
        &[(0, 0), (0, 1), (1, 0), (2, 1), (1, 2), (2, 2), (-1, 0), (-2, 0), (3, 1)],
        Bug::Queen,
    );
    let cuts = board.find_cut_vertexes();
    let cut_locs: Vec<_> = (0..GRID_MASK).filter(|&hex| cuts.get(hex)).map(loc).collect();
    assert_eq!(cut_locs, [(-1, 0), (2, 1)]);
}

#[test]
fn test_slidable() {
    let mut board = Mlp::default();
    let x = START_HEX;
    let slidable = |board: &Mlp| board.slidable_adjacent(x, x).collect::<Vec<_>>();
    // One neighbor.
    board.insert_loc((0, 0), Bug::Queen, Color::Black);
    board.insert_loc((1, 0), Bug::Queen, Color::Black);
    assert_eq!(slidable(&board), [hex(0, -1), hex(1, 1)]);
    // Two adjacent neighbors.
    board.insert_loc((1, 1), Bug::Queen, Color::Black);
    assert_eq!(slidable(&board), [hex(0, -1), hex(0, 1)]);
    // Four adjacent neighbors.
    board.insert_loc((0, 1), Bug::Queen, Color::Black);
    board.insert_loc((-1, 0), Bug::Queen, Color::Black);
    assert_eq!(slidable(&board), [hex(-1, -1), hex(0, -1)]);
    // Five adjacent neighbors.
    board.insert_loc((-1, -1), Bug::Queen, Color::Black);
    assert!(slidable(&board).is_empty());
    // Two separated groups of neighbors.
    board.remove_loc((0, 1));
    assert!(slidable(&board).is_empty());
    // Two opposite single neighbors.
    board.remove_loc((1, 1));
    board.remove_loc((-1, -1));
    assert_eq!(
        slidable(&board),
        [hex(-1, -1), hex(0, -1), hex(1, 1), hex(0, 1)]
    );
}

#[test]
fn test_generate_jumps() {
    let mut board = Mlp::default();
    //．．．🦗🦗🦗．
    // ．．🦗．．．
    //．．．．．．
    // ．🦗．．
    board.fill_board(&[(0, 0), (0, 1), (0, 3), (1, 0), (2, 0)], Bug::Grasshopper);
    let mut turns = Turns::new();
    board.generate_jumps(START_HEX, &mut turns);
    assert_movements(&turns, (0, 0), &[(0, 2), (3, 0)]);
}

#[test]
fn test_generate_beetle() {
    let mut board = Mlp::default();
    board.fill_board(
        &[
            (0, 0),
            (0, 0),
            (-1, -1),
            (-1, -1),
            (0, 1),
            (0, 1),
            (0, -1),
            (0, -1),
            (0, -1),
            (1, 0),
            (1, 0),
            (1, 1),
            (1, 1),
            (1, 1),
        ],
        Bug::Beetle,
    );
    // Stack heights:
    //   2   3
    //  0 (2) 2
    //   2   3
    // Blocked left (down) and right (up) by the stacks, but can climb onto all 4.
    let mut turns = Turns::new();
    board.generate_stack_walking(START_HEX, &mut turns);
    assert_movements(&turns, (0, 0), &[(-1, -1), (0, -1), (0, 1), (1, 1)]);
}

#[test]
fn test_generate_walk3() {
    let mut board = Mlp::default();
    //．．．🕷．．．．．
    // ．．．🕷．🕷．．
    //．．．🕷．．🕷．
    // ．．．🕷🕷🕷
    board.fill_board(
        &[(-1, -1), (0, 0), (2, 0), (0, 1), (3, 1), (1, 2), (2, 2), (3, 2)],
        Bug::Spider,
    );
    let mut turns = Turns::new();
    board.generate_walk3(hex(-1, -1), &mut turns);
    assert_movements(&turns, (-1, -1), &[(0, 2), (1, -1), (1, 1), (2, 1)]);

    // ．．🕷．🕷．．
    //．．🕷🕷．🕷．
    // ．．🕷🕷🕷
    board.remove_loc((-1, -1));
    board.insert_loc((1, 1), Bug::Spider, Color::Black);
    turns.clear();
    board.generate_walk3(hex(1, 1), &mut turns);
    assert_movements(&turns, (1, 1), &[(-1, -1), (0, -1), (1, -1), (2, -1)]);
}

#[test]
fn test_generate_walk_all() {
    let mut board = Mlp::default();
    //．．．🐜．．．．
    // ．．．🐜．．．
    //．．．🐜．🐜．
    // ．．．🐜🐜
    board.fill_board(&[(-1, -1), (0, 0), (0, 1), (2, 1), (1, 2), (2, 2)], Bug::Ant);
    let mut turns = Turns::new();
    board.generate_walk_all(hex(-1, -1), &mut turns);
    assert_movements(
        &turns,
        (-1, -1),
        &[
            (0, -1),
            (-1, 0),
            (1, 0),
            (2, 0),
            (-1, 1),
            (3, 1),
            (0, 2),
            (3, 2),
            (1, 3),
            (2, 3),
            (3, 3),
        ],
    );
}

#[test]
fn test_generate_mosquito() {
    let mut board = Mlp::default();
    board.fill_board(&[(0, 0), (1, 1)], Bug::Mosquito);
    let mut turns = Turns::new();
    board.generate_mosquito(hex(0, 0), &mut turns);
    // A mosquito next to only a mosquito can't move.
    assert_movements(&turns, (0, 0), &[]);

    //．．🦟🦗．
    // ．🐜🪲．
    board.insert_loc((0, 1), Bug::Ant, Color::Black);
    board.insert_loc((1, 1), Bug::Beetle, Color::Black);
    board.insert_loc((1, 0), Bug::Grasshopper, Color::Black);
    turns.clear();
    // Black to move, whose pieces these are.
    board.turn_num += 1;
    board.generate_movements(&mut turns);
    assert_movements(
        &turns,
        (0, 0),
        &[
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (0, 2),
            (1, -1),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ],
    );
}

#[test]
fn test_generate_ladybug() {
    let mut board = Mlp::default();
    //．．．🐞．．．
    // ．．🐞．🐞．．
    //．．．🐞🐞．．
    // ．．．🐞．．
    board.fill_board(&[(2, 3), (0, 0), (0, 1), (2, 1), (1, 2), (2, 2)], Bug::Ladybug);
    let mut turns = Turns::new();
    board.generate_ladybug(hex(2, 3), &mut turns);
    assert_movements(
        &turns,
        (2, 3),
        &[(-1, 0), (1, 0), (2, 0), (-1, 1), (1, 1), (3, 1), (0, 2), (3, 2), (1, 3), (3, 3)],
    );
}

#[test]
fn test_generate_throws() {
    let mut board = Mlp::default();
    board.fill_board(&[(1, 1), (0, 0), (0, 0), (2, 2), (2, 2), (0, 1), (1, 2)], Bug::Pillbug);
    // ．．💊．．．
    //．．💊💊．．
    // ．．💊💊．
    let mut turns = Turns::new();
    let immovable = HexSet::new();
    let mut starts = HexSet::new();
    let mut ends = HexSet::new();
    let start = hex(1, 1);
    board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
    assert_eq!(turns.len(), 4);
    assert_movements(&turns[..2], (1, 2), &[(1, 0), (2, 1)]);
    assert_movements(&turns[2..], (0, 1), &[(1, 0), (2, 1)]);

    // A level-2 gate keeps one piece from being thrown.
    board.remove_loc((0, 0));
    board.insert_loc((0, 1), Bug::Pillbug, Color::Black);
    turns.clear();
    board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
    assert_eq!(turns.len(), 2);
    assert_movements(&turns, (0, 0), &[(1, 0), (2, 1)]);

    // A level-2 gate keeps one hex from being thrown to.
    board.insert_loc((1, 0), Bug::Pillbug, Color::Black);
    board.insert_loc((1, 0), Bug::Pillbug, Color::Black);
    board.remove_loc((0, 1));
    board.remove_loc((0, 1));
    board.remove_loc((1, 2));
    turns.clear();
    board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
    assert_eq!(turns.len(), 2);
    assert_movements(&turns, (0, 0), &[(0, 1), (1, 2)]);
}

#[test]
fn test_repetition_draw() {
    let mut board = Mlp::default();
    let (x1, x2, y1, y2) = (hex(-1, -1), hex(-1, 0), hex(1, 1), hex(1, 0));
    let setup = [
        Turn::Place(hex(0, 0), Bug::Spider),
        Turn::Place(x1, Bug::Queen),
        Turn::Place(y1, Bug::Queen),
    ];
    let shuffle = [
        Turn::Move(x1, x2),
        Turn::Move(y1, y2),
        Turn::Move(x2, x1),
        Turn::Move(y2, y1),
    ];
    // Setting up makes the position once, and each shuffle makes it again.
    for turn in setup.into_iter().chain(shuffle).chain(shuffle) {
        assert_eq!(board.get_result(), None);
        board.make_move(turn);
    }
    assert_eq!(board.get_result(), Some(GameResult::Draw));
    board.undo();
    assert_eq!(board.get_result(), None);
    board.make_move(Turn::Move(y2, y1));
    assert_eq!(board.get_result(), Some(GameResult::Draw));
}

#[test]
fn test_finds_winning_move() {
    // ．．．🐝🕷．．
    //．．🐜🐜🐝．．
    // ．．．🦗🪲
    let mut board = Mlp::default();
    for turn in [
        Turn::Place(hex(0, 0), Bug::Queen),
        Turn::Place(hex(1, 0), Bug::Spider),
        Turn::Place(hex(-1, 1), Bug::Ant),
        Turn::Place(hex(0, 1), Bug::Ant),
        Turn::Place(hex(1, 2), Bug::Grasshopper),
        Turn::Place(hex(1, 1), Bug::Queen),
        Turn::Place(hex(2, 2), Bug::Beetle),
        Turn::Pass,
    ] {
        board.make_move(turn);
    }
    for depth in 0..3 {
        let (turn, score) = alphabeta(stupid_eval::<Mlp>).find_best(&mut board, depth, None);
        assert_eq!(turn, Turn::Move(hex(-1, 1), hex(2, 1)));
        assert_eq!(score, f32::INFINITY);

        let (turn, score) = alphabeta(eval::<true, true, true>).find_best(&mut board, depth, None);
        assert_eq!(turn, Turn::Move(hex(-1, 1), hex(2, 1)));
        assert_eq!(score, f32::INFINITY);
    }
}

#[test]
fn test_finds_queen_escape() {
    //．．🕷🐝🐝．
    // ．．🦗🕷．
    let mut board = Mlp::default();
    for turn in [
        Turn::Place(hex(0, 0), Bug::Queen),
        Turn::Place(hex(1, 0), Bug::Queen),
        Turn::Place(hex(1, 1), Bug::Spider),
        Turn::Place(hex(0, 1), Bug::Grasshopper),
        Turn::Place(hex(-1, 0), Bug::Beetle),
        Turn::Pass,
    ] {
        board.make_move(turn);
    }
    for depth in 0..2 {
        let (turn, _) = alphabeta(eval::<true, true, true>).find_best(&mut board, depth, None);
        assert_eq!(turn, Turn::Move(hex(0, 0), hex(0, -1)));
    }
}

#[test]
fn test_placeable() {
    let mut board = Mlp::default();
    board.insert_loc((0, 0), Bug::Ant, Color::White);
    board.insert_loc((1, 0), Bug::Ant, Color::Black);
    board.insert_loc((0, 1), Bug::Ant, Color::White);
    let [nw, ne, _, se, _, _] = adjacent(START_HEX);
    assert!(!placeable(&board, se, Color::White));
    assert!(!placeable(&board, ne, Color::White));
    assert!(placeable(&board, nw, Color::White));
    assert!(!placeable(&board, se, Color::Black));
    assert!(!placeable(&board, ne, Color::Black));
    assert!(!placeable(&board, nw, Color::Black));
}

#[test]
fn test_undo_restores_a_long_game() {
    // Undoing can reorder a color's pieces, and so the moves.
    let sorted_moves = |board: &Mlp| {
        let mut moves = board.candidate_moves();
        moves.sort();
        moves
    };
    let mut board = Mlp::default();
    let mut hashes = vec![board.hash()];
    let mut moves = vec![sorted_moves(&board)];
    for i in 0..300 {
        if board.get_result().is_some() {
            break;
        }
        let candidates = board.candidate_moves();
        board.make_move(candidates[i * 7 % candidates.len()]);
        hashes.push(board.hash());
        moves.push(sorted_moves(&board));
    }
    while hashes.len() > 1 {
        hashes.pop();
        moves.pop();
        board.undo();
        assert_eq!(board.hash(), *hashes.last().unwrap());
        assert_eq!(sorted_moves(&board), *moves.last().unwrap());
    }
}

#[test]
fn test_neighbors_follow_moves_and_undos() {
    let assert_neighbors = |board: &Mlp| {
        for hex in 0..=GRID_MASK {
            let expected = (0..6)
                .filter(|&i| board.occupied(adjacent(hex)[i]))
                .fold(0u8, |mask, i| mask | 1 << i);
            assert_eq!(board.neighbors[hex as usize], expected, "at {:?}", loc(hex));
        }
    };
    let mut board = Mlp::default();
    for i in 0..300 {
        if board.get_result().is_some() {
            break;
        }
        let candidates = board.candidate_moves();
        board.make_move(candidates[i * 7 % candidates.len()]);
        assert_neighbors(&board);
    }
    while board.ply() > 0 {
        board.undo();
        assert_neighbors(&board);
    }
}

#[test]
fn test_turn_round_trips_through_text() {
    for turn in [
        Turn::Place(hex(1, -1), Bug::Ant),
        Turn::Move(hex(0, 0), hex(-3, 2)),
        Turn::Pass,
    ] {
        assert_eq!(turn.to_string().parse::<Turn>().unwrap(), turn);
    }
}

#[test]
fn test_perft_matches_nokamute() {
    fn counts<S: GameState>() -> Vec<u64> {
        (0..5).map(|depth| perft(&mut S::default(), depth)).collect()
    }
    assert_eq!(counts::<HiveState<false, false, false>>(), [1, 4, 96, 1440, 21600]);
    assert_eq!(counts::<Mlp>(), [1, 7, 294, 6678, 151686]);
}

#[test]
fn test_hex_at_finds_the_drawn_hex() {
    let board: Mlp = position(0, 20);
    let hexes = board.candidate_moves().into_iter().flat_map(|turn| match turn {
        Turn::Place(hex, _) => vec![hex],
        Turn::Move(start, end) => vec![start, end],
        Turn::Pass => vec![],
    });
    let mark = SetBackgroundColor(MARK_BG).to_string();
    for hex in hexes {
        let drawn = board.marked(&[hex], None);
        let (y, line) = drawn
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains(&mark))
            .unwrap();
        let x = width(&line[..line.find(&mark).unwrap()]);
        // Both columns of the hex.
        assert_eq!(board.hex_at(x, y as u16), Some(hex));
        assert_eq!(board.hex_at(x + 1, y as u16), Some(hex));
    }
}

/// The terminal columns `s` takes: none for an escape, one for ASCII, and two for the rest,
/// the bugs and the empty hexes.
fn width(s: &str) -> u16 {
    let mut width = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            chars.by_ref().find(|&c| c == 'm');
        } else {
            width += if c.is_ascii() { 1 } else { 2 };
        }
    }
    width
}
