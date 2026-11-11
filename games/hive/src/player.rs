use general_minimax::state::GameState;

use crate::{
    bug::Bug,
    hex_grid::Hex,
    state::{
        HiveState, Turn,
        display::SELECTED_BG,
    },
};

/// What the player has picked so far this turn.
#[derive(Clone, Copy, PartialEq)]
enum Selection {
    Nothing,
    /// The piece on this hex, to move.
    Piece(Hex),
    /// A bug from the reserve, to place.
    Bug(Bug),
}

/// Asks the player to move for their turn with the mouse: a piece and then where it goes, or a
/// bug from their list on the left and then where it goes. Passes by itself when that's the
/// only move.
pub fn human<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    state: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
) -> Turn {
    use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};

    let moves = state.candidate_moves();
    if moves[0] == Turn::Pass {
        println!("Nothing to do but pass.");
        return Turn::Pass;
    }

    enable_raw_mode().unwrap();
    execute!(std::io::stdout(), EnableMouseCapture).unwrap();

    let me = state.current_player();
    let placeable = |bug| {
        moves
            .iter()
            .any(|&turn| matches!(turn, Turn::Place(_, placed) if placed == bug))
    };

    let mut selection = Selection::Nothing;
    // Why the last click did nothing, shown until the next one.
    let mut notice = None;
    let result = loop {
        let message = match (notice.take(), selection) {
            (Some(notice), _) => notice,
            (None, Selection::Nothing) => "Click a piece, or a bug on the left to place.".to_owned(),
            (None, Selection::Piece(_)) => "Click where it goes.".to_owned(),
            (None, Selection::Bug(bug)) => format!("Click where the {} goes.", bug.name()),
        };
        let layout = draw(state, me, &marks(&moves, selection), selection, &message);
        // A resize moves the board, so it's drawn again before the next click.
        let Some((x, y)) = read_click() else {
            continue;
        };

        if let Some(bug) = layout.list_bug(x, y) {
            if placeable(bug) {
                selection = Selection::Bug(bug);
            } else {
                notice = Some(if bug == Bug::Queen && state.ply() < 2 {
                    "The queen can't be placed on your first turn.".to_owned()
                } else if placeable(Bug::Queen) {
                    // Only the queen is offered, so it's due.
                    "Your queen has to be placed by your fourth turn.".to_owned()
                } else {
                    format!("There's nowhere to place the {}.", bug.name())
                });
            }
            continue;
        }

        let clicked = x
            .checked_sub(layout.col)
            .zip(y.checked_sub(layout.row))
            .and_then(|(x, y)| state.hex_at(x, y));
        let Some(hex) = clicked else {
            selection = Selection::Nothing;
            continue;
        };
        let turn = match selection {
            Selection::Piece(start) => Some(Turn::Move(start, hex)),
            Selection::Bug(bug) => Some(Turn::Place(hex, bug)),
            Selection::Nothing => None,
        };
        if let Some(turn) = turn.filter(|turn| moves.contains(turn)) {
            // The board is drawn again only on the next turn, after the opponent's move.
            let mut next = state.clone();
            next.make_move(turn);
            draw(&next, me, &[], Selection::Nothing, "Waiting for the other player...");
            break turn;
        }
        let movable = moves
            .iter()
            .any(|&turn| matches!(turn, Turn::Move(start, _) if start == hex));
        selection = if movable && selection != Selection::Piece(hex) {
            Selection::Piece(hex)
        } else {
            Selection::Nothing
        };
    };

    restore();
    result
}

/// Columns between a list of unplaced bugs and the side of the terminal.
const SIDE_MARGIN: u16 = 2;
/// Columns a list of unplaced bugs takes: the bug, a space and the count.
const LIST_WIDTH: u16 = 5;

/// The hexes to mark for `selection`: the pieces that can move, a piece's destinations, or
/// where a bug can be placed.
fn marks(moves: &[Turn], selection: Selection) -> Vec<Hex> {
    moves
        .iter()
        .filter_map(|&turn| match (selection, turn) {
            (Selection::Nothing, Turn::Move(start, _)) => Some(start),
            (Selection::Piece(piece), Turn::Move(start, end)) if piece == start => Some(end),
            (Selection::Bug(picked), Turn::Place(hex, bug)) if picked == bug => Some(hex),
            _ => None,
        })
        .collect()
}

/// Where `draw` put the board and the player's list on the terminal.
struct Layout {
    /// The column of the board's left edge.
    col: u16,
    /// The row of the board's top edge.
    row: u16,
    /// The row of the lists' headers, with a bug on each row below.
    list_top: u16,
    /// The player's unplaced bugs, top to bottom.
    mine: Vec<Bug>,
}

impl Layout {
    /// The bug in the player's list at column `x` and row `y`, if any.
    fn list_bug(&self, x: u16, y: u16) -> Option<Bug> {
        if !(SIDE_MARGIN..SIDE_MARGIN + LIST_WIDTH).contains(&x) {
            return None;
        }
        let i = y.checked_sub(self.list_top + 1)?;
        self.mine.get(usize::from(i)).copied()
    }
}

/// Clears the terminal and draws `state` centred with `marks` and the selection coloured, and
/// `message` along the top. The bugs `me` has left go down the left side, the opponent's down
/// the right.
fn draw<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    state: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
    me: u8,
    marks: &[Hex],
    selection: Selection,
    message: &str,
) -> Layout {
    use std::io::Write;

    use crossterm::{
        cursor::MoveTo,
        execute,
        style::Stylize,
        terminal::{Clear, ClearType, size},
    };

    let mut stdout = std::io::stdout();
    let selected = match selection {
        Selection::Piece(hex) => Some(hex),
        _ => None,
    };
    let (board_width, board_height) = state.board_size();
    let (term_width, term_height) = size().unwrap_or((80, 24));
    let col = term_width.saturating_sub(board_width) / 2;
    // Clear of the message.
    let row = (term_height.saturating_sub(board_height) / 2).max(3);

    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
    let message_col = term_width.saturating_sub(message.len() as u16) / 2;
    execute!(stdout, MoveTo(message_col, 1)).unwrap();
    print!("{message}");
    for (i, line) in state.marked(marks, selected).lines().enumerate() {
        execute!(stdout, MoveTo(col, row + i as u16)).unwrap();
        print!("{line}");
    }

    // A header and at most 8 bugs.
    let list_top = term_height.saturating_sub(9) / 2;
    let right = term_width.saturating_sub(SIDE_MARGIN + LIST_WIDTH);
    for (list_col, player) in [(SIDE_MARGIN, me), (right, 1 - me)] {
        execute!(stdout, MoveTo(list_col, list_top)).unwrap();
        print!("{}", if player == 0 { "White" } else { "Black" });
        for (i, (bug, count)) in state.unplaced(player).enumerate() {
            execute!(stdout, MoveTo(list_col, list_top + 1 + i as u16)).unwrap();
            let codepoint = bug.codepoint().stylize();
            let styled = if player == me && selection == Selection::Bug(bug) {
                codepoint.on(SELECTED_BG)
            } else if player == 0 {
                codepoint.reverse()
            } else {
                codepoint
            };
            print!("{styled} {count}");
        }
    }
    let _ = stdout.flush();

    Layout {
        col,
        row,
        list_top,
        mine: state.unplaced(me).map(|(bug, _)| bug).collect(),
    }
}

/// Waits for a left click and returns its column and row, or `None` when the terminal is
/// resized. Ctrl-C ends the program.
fn read_click() -> Option<(u16, u16)> {
    use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

    loop {
        match event::read() {
            Ok(Event::Mouse(m)) if m.kind == MouseEventKind::Down(MouseButton::Left) => {
                return Some((m.column, m.row));
            }
            Ok(Event::Resize(..)) => return None,
            // Raw mode turns Ctrl-C into a key press instead of a SIGINT.
            Ok(Event::Key(k))
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                restore();
                std::process::exit(130);
            }
            _ => {}
        }
    }
}

/// Undoes the raw mode and mouse capture `human` turns on.
fn restore() {
    use crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};

    execute!(std::io::stdout(), DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
}
