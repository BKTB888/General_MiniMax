pub(crate) mod display;
mod evaluation;
#[cfg(test)]
mod tests;

pub use evaluation::{BasicEvaluator, eval};

use arrayvec::ArrayVec;
use general_minimax::{mixers::Splitmix, result::GameResult, state::GameState};
use smallvec::SmallVec;

use crate::{
    bug::Bug,
    hex_grid::{Direction, GRID_SIZE, Hex, HexSet, START_HEX, adjacent},
    node::{Color, Node, UnderNode},
};

/// Candidate moves held without a heap allocation.
const INLINE_MOVES: usize = 64;

/// The moves of one position, as the move generators fill them.
type Turns = SmallVec<[Turn; INLINE_MOVES]>;

/// Flips the hash with the player to move.
const TO_MOVE_KEY: u64 = 0xa6c1_1b62_6b10_5b7c;

/// The most pieces one color has, with every expansion.
const MAX_PIECES: usize = 14;

/// Place a new bug, move the top piece of one hex to another, or pass when nothing else is
/// legal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Turn {
    Place(Hex, Bug),
    Move(Hex, Hex),
    Pass,
}

/// Hive with the base pieces, plus each expansion piece whose parameter is true.
#[derive(Clone)]
pub struct HiveState<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool> {
    /// The top piece of each hex, indexed by `Hex`.
    nodes: [Node; GRID_SIZE],
    /// Bit i of a hex's mask is set when `adjacent(hex)[i]` is occupied.
    neighbors: [u8; GRID_SIZE],
    /// The pieces under the tops, sorted by height for each hex.
    underworld: ArrayVec<UnderNode, 8>,
    /// How many of each bug each color has left to place, indexed by `Color` then `Bug`.
    remaining: [[u8; 8]; 2],
    /// Where each color's queen is, or `START_HEX` while it isn't on the board.
    queens: [Hex; 2],
    /// The hexes each color has on top.
    occupied_hexes: [ArrayVec<Hex, MAX_PIECES>; 2],
    // Weak players can wander for over 65535 turns without repeating.
    turn_num: u32,
    zobrist_hash: u64,
    /// The hash after each turn, to spot repetitions.
    zobrist_history: Vec<u64>,
    turn_history: Vec<Turn>,
}

impl<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>
    HiveState<MOSQUITO, LADYBUG, PILLBUG>
{
    /// How many of each bug a color starts with, indexed by `Bug`.
    const COUNTS: [u8; 8] = [1, 3, 2, 3, 2, MOSQUITO as u8, LADYBUG as u8, PILLBUG as u8];

    fn to_move(&self) -> Color {
        if self.turn_num.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        }
    }

    /// The bugs `player` has left to place, with how many of each.
    pub(crate) fn unplaced(&self, player: u8) -> impl Iterator<Item = (Bug, u8)> {
        let remaining = self.remaining[usize::from(player)];
        Bug::ALL
            .into_iter()
            .map(move |bug| (bug, remaining[bug as usize]))
            .filter(|&(_, count)| count > 0)
    }

    /// How many of each bug the player to move has left to place.
    fn get_remaining(&self) -> &[u8; 8] {
        &self.remaining[self.to_move() as usize]
    }

    fn mut_remaining(&mut self) -> &mut [u8; 8] {
        &mut self.remaining[self.to_move() as usize]
    }

    /// Puts a `bug` of `color` on top of `hex`.
    fn insert(&mut self, hex: Hex, bug: Bug, bug_num: u8, color: Color) {
        let prev = self.node(hex);
        if prev.occupied() {
            if prev.color() != color {
                self.occupied_remove(prev.color(), hex);
                self.occupied_add(color, hex);
            }
            self.insert_underworld(prev, hex);
        } else {
            self.occupied_add(color, hex);
            self.toggle_neighbors(hex);
        }
        self.nodes[hex as usize] = Node::new(bug, color, bug_num, prev.clipped_height() + 1);
        self.zobrist_hash ^= zobrist_key(hex, bug, color, self.height(hex));
        if bug == Bug::Queen {
            self.queens[color as usize] = hex;
        }
    }

    /// Takes the top piece off `hex`, which must have one, and returns its bug, number and
    /// color.
    fn remove(&mut self, hex: Hex) -> (Bug, u8, Color) {
        let height = self.height(hex);
        let prev = self.node(hex);
        let uncovered = if height > 1 {
            self.remove_underworld(hex)
        } else {
            Node::EMPTY
        };
        self.nodes[hex as usize] = uncovered;
        let (bug, color) = (prev.bug(), prev.color());
        if !uncovered.occupied() {
            self.occupied_remove(color, hex);
            self.toggle_neighbors(hex);
        } else if uncovered.color() != color {
            self.occupied_remove(color, hex);
            self.occupied_add(uncovered.color(), hex);
        }
        self.zobrist_hash ^= zobrist_key(hex, bug, color, height);
        if bug == Bug::Queen {
            self.queens[color as usize] = START_HEX;
        }
        (bug, prev.bug_num(), color)
    }

    /// Flips `hex` in its neighbors' masks, for when it fills or empties.
    fn toggle_neighbors(&mut self, hex: Hex) {
        for (i, adj) in adjacent(hex).into_iter().enumerate() {
            // `adj` sees `hex` in the opposite direction.
            self.neighbors[adj as usize] ^= 1 << ((i + 3) % 6);
        }
    }

    /// Covers `node`, the top of `hex`, before something climbs onto it.
    fn insert_underworld(&mut self, node: Node, hex: Hex) {
        let height = self.underworld_height(hex, node);
        self.underworld.push(UnderNode::new(node, hex, height));
    }

    /// Uncovers the highest piece under the top of `hex`, which must have one.
    fn remove_underworld(&mut self, hex: Hex) -> Node {
        let i = self
            .underworld
            .iter()
            .rposition(|under| under.hex == hex)
            .expect("nothing under the top of hex");
        self.underworld.remove(i).node
    }

    /// The height of `node`, the top of `hex`.
    fn underworld_height(&self, hex: Hex, node: Node) -> u8 {
        let height = node.clipped_height();
        if height < 3 {
            return height;
        }
        // Past the cap, it's one above the highest piece it covers, which is listed last.
        let below = self.underworld.iter().rev().find(|under| under.hex == hex);
        1 + below.expect("a stack of 3 covers pieces").height
    }

    fn height(&self, hex: Hex) -> u8 {
        self.underworld_height(hex, self.node(hex))
    }

    fn node(&self, hex: Hex) -> Node {
        self.nodes[hex as usize]
    }

    fn occupied(&self, hex: Hex) -> bool {
        self.node(hex).occupied()
    }

    fn occupied_add(&mut self, color: Color, hex: Hex) {
        self.occupied_hexes[color as usize].push(hex);
    }

    fn occupied_remove(&mut self, color: Color, hex: Hex) {
        let hexes = &mut self.occupied_hexes[color as usize];
        let i = hexes.iter().position(|&h| h == hex).unwrap();
        hexes.swap_remove(i);
    }

    /// Whether this position came up twice before, in the last 32 turns.
    fn is_repetition(&self) -> bool {
        // Both players have to move away and back, so a position can only recur every 4 turns.
        self.zobrist_history.len() > 10
            && self
                .zobrist_history
                .iter()
                .rev()
                .step_by(4)
                .skip(1)
                .take(8)
                .filter(|&&hash| hash == self.zobrist_hash)
                .count()
                >= 2
    }

    /// Each color's first turn, which can be any bug but the queen. Leaving the queen out is
    /// the tournament rule, which makes for fewer draws.
    fn generate_openings(&self, turns: &mut Turns) {
        for bug in self.unplaced_bugs().filter(|&bug| bug != Bug::Queen) {
            if self.turn_num == 0 {
                turns.push(Turn::Place(START_HEX, bug));
            } else {
                for hex in adjacent(START_HEX) {
                    turns.push(Turn::Place(hex, bug));
                }
            }
        }
    }

    /// New bugs go next to the player's own pieces and nowhere next to the opponent's.
    fn generate_placements(&self, turns: &mut Turns) {
        let bugs: ArrayVec<Bug, 8> = if self.queen_required() {
            [Bug::Queen].into_iter().collect()
        } else {
            self.unplaced_bugs().collect()
        };

        // Next to an opponent's piece, or already looked at.
        let mut no_placement = HexSet::new();
        for &enemy in &self.occupied_hexes[self.to_move().other()] {
            for adj in adjacent(enemy) {
                no_placement.set(adj);
            }
        }

        for &friend in &self.occupied_hexes[self.to_move() as usize] {
            for hex in adjacent(friend) {
                if no_placement.get(hex) {
                    continue;
                }
                no_placement.set(hex);
                if self.occupied(hex) {
                    continue;
                }
                // Pushing one at a time was 20% slower: `extend` reserves once, then skips
                // the capacity check per turn.
                turns.extend(bugs.iter().map(|&bug| Turn::Place(hex, bug)));
            }
        }
    }

    /// Every move of a piece on the board, for a player whose queen is down.
    fn generate_movements(&self, turns: &mut Turns) {
        let mut immovable = self.find_cut_vertexes();
        let stunned = match self.turn_history.last() {
            Some(&Turn::Move(_, end)) => Some(end),
            _ => None,
        };
        if let Some(moved) = stunned {
            // A piece moved on the opponent's turn can't move, or be thrown, on this one.
            immovable.set(moved);
        }

        // Throws need deduping against the bugs' own moves, so they come first.
        let mut throw_starts = HexSet::new();
        let mut throw_ends = HexSet::new();
        let first_move = turns.len();
        if PILLBUG {
            for &hex in &self.occupied_hexes[self.to_move() as usize] {
                let marker = turns.len();
                let node = self.node(hex);
                if stunned == Some(hex) {
                    continue;
                }
                let throws = node.bug() == Bug::Pillbug
                    || MOSQUITO
                        && node.bug() == Bug::Mosquito
                        && !node.is_stacked()
                        && adjacent(hex).into_iter().any(|adj| {
                            let n = self.node(adj);
                            n.occupied() && n.bug() == Bug::Pillbug
                        });
                if !throws {
                    continue;
                }
                self.generate_throws(&immovable, hex, turns, &mut throw_starts, &mut throw_ends);
                // A pillbug and a mosquito next to it can throw the same piece.
                if marker > 0 {
                    let mut i = marker;
                    while i < turns.len() {
                        if turns[first_move..marker].contains(&turns[i]) {
                            turns.swap_remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
        let num_throws = turns.len();

        for &hex in &self.occupied_hexes[self.to_move() as usize] {
            let marker = turns.len();
            let node = self.node(hex);
            if node.is_stacked() {
                self.generate_stack_walking(hex, turns);
                continue;
            }
            if immovable.get(hex) {
                continue;
            }
            match node.bug() {
                Bug::Queen | Bug::Pillbug => self.generate_walk1(hex, turns),
                Bug::Grasshopper => self.generate_jumps(hex, turns),
                Bug::Spider => self.generate_walk3(hex, turns),
                Bug::Ant => self.generate_walk_all(hex, turns),
                Bug::Beetle => {
                    self.generate_walk1(hex, turns);
                    self.generate_stack_walking(hex, turns);
                }
                Bug::Mosquito => self.generate_mosquito(hex, turns),
                Bug::Ladybug => self.generate_ladybug(hex, turns),
            }

            // A throw by the player's own pillbug can repeat a move the bug makes itself.
            if PILLBUG && throw_starts.get(hex) {
                let mut i = marker;
                while i < turns.len() {
                    let turn = turns[i];
                    if let Turn::Move(_, end) = turn
                        && throw_ends.get(end)
                        && turns[first_move..num_throws].contains(&turn)
                    {
                        turns.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    /// The hexes whose pieces would split the hive by leaving.
    fn find_cut_vertexes(&self) -> HexSet {
        // Any piece will do: the cuts don't depend on where the search starts.
        let Some(&start) = self.occupied_hexes.iter().flatten().next() else {
            return HexSet::new();
        };

        // Articulation points, found in one DFS: https://cp-algorithms.com/graph/cutpoints.html
        struct Search<'a, const M: bool, const L: bool, const P: bool> {
            state: &'a HiveState<M, L, P>,
            visited: HexSet,
            cuts: HexSet,
            /// The order each hex was visited in, from 1.
            num: [u8; GRID_SIZE],
            /// The lowest `num` reachable by DFS edges and then at most one back edge.
            low: [u8; GRID_SIZE],
            visit_num: u8,
        }

        fn dfs<const M: bool, const L: bool, const P: bool>(
            search: &mut Search<M, L, P>,
            hex: Hex,
            parent: Hex,
        ) {
            search.visited.set(hex);
            search.num[hex as usize] = search.visit_num;
            search.low[hex as usize] = search.visit_num;
            search.visit_num += 1;
            let root = hex == parent;
            let mut children = 0;
            for adj in adjacent(hex) {
                if !search.state.occupied(adj) || adj == parent {
                    continue;
                }
                if search.visited.get(adj) {
                    search.low[hex as usize] = search.low[hex as usize].min(search.num[adj as usize]);
                } else {
                    dfs(search, adj, hex);
                    search.low[hex as usize] = search.low[hex as usize].min(search.low[adj as usize]);
                    if search.low[adj as usize] >= search.num[hex as usize] && !root {
                        search.cuts.set(hex);
                    }
                    children += 1;
                }
            }
            if root && children > 1 {
                search.cuts.set(hex);
            }
        }

        let mut search = Search {
            state: self,
            visited: HexSet::new(),
            cuts: HexSet::new(),
            num: [0; GRID_SIZE],
            low: [0; GRID_SIZE],
            visit_num: 1,
        };
        dfs(&mut search, start, start);
        search.cuts
    }

    /// A bit per neighbor of `hex`, in `adjacent` order, set when it's occupied and isn't
    /// `origin`. `neighbors` is `adjacent(hex)`.
    fn occupied_neighbors(&self, origin: Hex, hex: Hex, neighbors: &[Hex; 6]) -> u32 {
        // `neighbors` is passed in because the search ran 10% slower when this called
        // `adjacent` again, even inlined.
        let mut occupied = u32::from(self.neighbors[hex as usize]);
        // Branch-free: an early-exit search for `origin` was slower than the loads it saves.
        for (i, &neighbor) in neighbors.iter().enumerate() {
            occupied &= !(u32::from(neighbor == origin) << i);
        }
        occupied
    }

    /// The neighbors of `hex` a crawler can slide to from it, staying against the hive.
    /// `origin` is where the crawler started; it's moving, so it's no wall to crawl along.
    fn slidable_adjacent(&self, origin: Hex, hex: Hex) -> impl Iterator<Item = Hex> {
        let neighbors = adjacent(hex);
        let mut occupied = self.occupied_neighbors(origin, hex, &neighbors);
        // Empty, with exactly one of the two hexes beside it occupied: both would be a gate,
        // neither would leave the hive. Three copies let the ring wrap around.
        occupied |= (occupied << 6) | (occupied << 12);
        let slidable = (!occupied & ((occupied << 1) ^ (occupied >> 1))) >> 6;
        neighbors
            .into_iter()
            .enumerate()
            .filter(move |&(i, _)| (slidable >> i) & 1 != 0)
            .map(|(_, neighbor)| neighbor)
    }

    /// The neighbors of `hex` a climber can step to, onto the hive or along it. `origin` is
    /// where the climber started.
    fn slidable_adjacent_beetle(&self, origin: Hex, hex: Hex) -> impl Iterator<Item = Hex> {
        // Not the original rule, where a beetle on the hive goes anywhere: it slides past
        // the higher of where it is and where it goes, as in
        // https://www.boardgamegeek.com/thread/332467
        let mut self_height = self.height(hex);
        if origin == hex {
            self_height -= 1;
        }
        let neighbors = adjacent(hex);
        let heights = neighbors.map(|neighbor| self.height(neighbor));
        (0..6)
            .filter(move |&i| {
                let barrier = self_height.max(heights[i]);
                // On the ground the sliding rule applies instead. Stacks higher than the
                // barrier on both sides make a gate.
                barrier != 0 && !(heights[(i + 1) % 6] > barrier && heights[(i + 5) % 6] > barrier)
            })
            .map(move |i| neighbors[i])
    }

    /// One step for the piece on top of the stack on `hex`.
    fn generate_stack_walking(&self, hex: Hex, turns: &mut Turns) {
        for adj in self.slidable_adjacent_beetle(hex, hex) {
            turns.push(Turn::Move(hex, adj));
        }
    }

    /// Jumps over each straight line of pieces next to `hex`.
    fn generate_jumps(&self, hex: Hex, turns: &mut Turns) {
        for dir in Direction::ALL {
            let mut jump = dir.apply(hex);
            let mut dist = 1;
            while self.occupied(jump) {
                jump = dir.apply(jump);
                dist += 1;
                if jump == hex {
                    // The line wrapped around the whole grid.
                    dist = 0;
                    break;
                }
            }
            if dist > 1 {
                turns.push(Turn::Move(hex, jump));
            }
        }
    }

    fn generate_walk1(&self, hex: Hex, turns: &mut Turns) {
        for adj in self.slidable_adjacent(hex, hex) {
            turns.push(Turn::Move(hex, adj));
        }
    }

    /// Exactly three steps, without going back.
    fn generate_walk3(&self, orig: Hex, turns: &mut Turns) {
        let mut visited = HexSet::new();
        visited.set(orig);
        for s1 in self.slidable_adjacent(orig, orig) {
            for s2 in self.slidable_adjacent(orig, s1) {
                if s2 == orig {
                    continue;
                }
                for s3 in self.slidable_adjacent(orig, s2) {
                    if s3 != s1 && !visited.get(s3) {
                        turns.push(Turn::Move(orig, s3));
                        visited.set(s3);
                    }
                }
            }
        }
    }

    /// Any number of steps.
    fn generate_walk_all(&self, orig: Hex, turns: &mut Turns) {
        let mut visited = HexSet::new();
        let mut stack = ArrayVec::<Hex, 64>::new();
        stack.push(orig);
        while let Some(hex) = stack.pop() {
            if visited.get(hex) {
                continue;
            }
            visited.set(hex);
            if hex != orig {
                turns.push(Turn::Move(orig, hex));
            }
            for adj in self.slidable_adjacent(orig, hex) {
                if !visited.get(adj) {
                    stack.push(adj);
                }
            }
        }
    }

    /// Two steps on top of the hive, then one down.
    fn generate_ladybug(&self, hex: Hex, turns: &mut Turns) {
        let mut step2 = HexSet::new();
        let mut step3 = HexSet::new();
        for s1 in self.slidable_adjacent_beetle(hex, hex) {
            if !self.occupied(s1) {
                continue;
            }
            for s2 in self.slidable_adjacent_beetle(hex, s1) {
                if !self.occupied(s2) || step2.get(s2) {
                    continue;
                }
                step2.set(s2);
                for s3 in self.slidable_adjacent_beetle(hex, s2) {
                    if !self.occupied(s3) && !step3.get(s3) {
                        step3.set(s3);
                        turns.push(Turn::Move(hex, s3));
                    }
                }
            }
        }
    }

    /// The pillbug on `hex` lifts a neighbor over itself onto an empty neighbor. Records
    /// where each throw starts and ends.
    fn generate_throws(
        &self,
        immovable: &HexSet,
        hex: Hex,
        turns: &mut Turns,
        throw_starts: &mut HexSet,
        throw_ends: &mut HexSet,
    ) {
        let mut starts = ArrayVec::<Hex, 6>::new();
        let mut ends = ArrayVec::<Hex, 6>::new();
        // Not next to `hex`, so no neighbor counts as moving.
        let origin = Direction::NW.apply(Direction::NW.apply(hex));
        for adj in self.slidable_adjacent_beetle(origin, hex) {
            match self.height(adj) {
                0 => ends.push(adj),
                1 if !immovable.get(adj) => starts.push(adj),
                _ => {}
            }
        }
        for &start in &starts {
            for &end in &ends {
                turns.push(Turn::Move(start, end));
                throw_starts.set(start);
                throw_ends.set(end);
            }
        }
    }

    /// The moves of each bug next to a mosquito on the ground.
    fn generate_mosquito(&self, hex: Hex, turns: &mut Turns) {
        let mut targets = [false; 8];
        for adj in adjacent(hex) {
            let node = self.node(adj);
            if node.occupied() {
                targets[node.bug() as usize] = true;
            }
        }

        let first = turns.len();
        if targets[Bug::Ant as usize] {
            self.generate_walk_all(hex, turns);
        } else {
            // The ant's moves already hold every walk.
            if targets[Bug::Queen as usize]
                || targets[Bug::Beetle as usize]
                || targets[Bug::Pillbug as usize]
            {
                self.generate_walk1(hex, turns);
            }
            if targets[Bug::Spider as usize] {
                self.generate_walk3(hex, turns);
            }
        }
        if targets[Bug::Grasshopper as usize] {
            self.generate_jumps(hex, turns);
        }
        if targets[Bug::Beetle as usize] {
            self.generate_stack_walking(hex, turns);
        }
        if targets[Bug::Ladybug as usize] {
            self.generate_ladybug(hex, turns);
        }

        // Two bugs can reach the same hex.
        let mut dests = HexSet::new();
        let mut i = first;
        while i < turns.len() {
            let Turn::Move(_, dest) = turns[i] else {
                unreachable!("a mosquito only moves");
            };
            if dests.get(dest) {
                turns.swap_remove(i);
            } else {
                dests.set(dest);
                i += 1;
            }
        }
    }

    /// The bugs the player to move has left to place.
    fn unplaced_bugs(&self) -> impl Iterator<Item = Bug> {
        let remaining = self.get_remaining();
        // `COUNTS` is known when compiling, so bugs outside the set are dropped there.
        Bug::ALL.into_iter().filter(move |&bug| {
            Self::COUNTS[bug as usize] > 0 && remaining[bug as usize] > 0
        })
    }

    /// Whether the player to move has to place their queen, from their fourth turn on.
    fn queen_required(&self) -> bool {
        self.turn_num > 5 && self.get_remaining()[Bug::Queen as usize] > 0
    }

    /// How many neighbors each color's queen has, indexed by `Color`.
    fn queens_surrounded(&self) -> [usize; 2] {
        self.queens
            .map(|queen| self.neighbors[queen as usize].count_ones() as usize)
    }
}

/// The key of a `bug` of `color` at `height` on `hex`.
fn zobrist_key(hex: Hex, bug: Bug, color: Color, height: u8) -> u64 {
    let idx = u64::from(hex) << 16 | u64::from(height) << 8 | (bug as u64) << 1 | color as u64;
    // Splitmix, not xorshift: a linear mixer lets keys XOR-cancel across hexes.
    idx.splitmix()
}

impl<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool> Default
    for HiveState<MOSQUITO, LADYBUG, PILLBUG>
{
    fn default() -> Self {
        Self {
            nodes: [Node::EMPTY; GRID_SIZE],
            neighbors: [0; GRID_SIZE],
            underworld: ArrayVec::new(),
            remaining: [Self::COUNTS; 2],
            queens: [START_HEX; 2],
            occupied_hexes: Default::default(),
            turn_num: 0,
            zobrist_hash: 0,
            zobrist_history: Vec::new(),
            turn_history: Vec::new(),
        }
    }
}

impl<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool> GameState
    for HiveState<MOSQUITO, LADYBUG, PILLBUG>
{
    type Choice = Turn;
    type Moves = Turns;
    const NUM_P: u8 = 2;
    const TT_BITS: u8 = 20;

    fn make_move(&mut self, turn: Turn) {
        match turn {
            Turn::Place(hex, bug) => {
                // Numbered from 1 in the order placed.
                let bug_num = Self::COUNTS[bug as usize] - self.get_remaining()[bug as usize] + 1;
                self.insert(hex, bug, bug_num, self.to_move());
                self.mut_remaining()[bug as usize] -= 1;
            }
            Turn::Move(start, end) => {
                let (bug, bug_num, color) = self.remove(start);
                self.insert(end, bug, bug_num, color);
            }
            Turn::Pass => {}
        }
        self.turn_num += 1;
        self.zobrist_hash ^= TO_MOVE_KEY;
        self.zobrist_history.push(self.zobrist_hash);
        self.turn_history.push(turn);
    }

    fn get_result(&self) -> Option<GameResult> {
        if self.is_repetition() {
            return Some(GameResult::Draw);
        }
        let me = self.to_move() as usize;
        let surrounded = self.queens_surrounded().map(|n| n == 6);
        match (surrounded[me], surrounded[1 - me]) {
            (true, true) => Some(GameResult::Draw),
            (true, false) => Some(GameResult::Player(1 - me as u8)),
            (false, true) => Some(GameResult::Player(me as u8)),
            (false, false) => None,
        }
    }

    fn candidate_moves(&self) -> Turns {
        let mut turns = Turns::new();
        if self.turn_num < 2 {
            self.generate_openings(&mut turns);
            return turns;
        }
        // Pieces may move once their queen is down.
        if self.get_remaining()[Bug::Queen as usize] == 0 {
            self.generate_movements(&mut turns);
        }
        if self.get_remaining().iter().any(|&n| n > 0) {
            self.generate_placements(&mut turns);
        }
        if turns.is_empty() {
            turns.push(Turn::Pass);
        }
        turns
    }

    fn is_valid(&self, turn: Turn) -> bool {
        self.candidate_moves().contains(&turn)
    }

    fn current_player(&self) -> u8 {
        self.to_move() as u8
    }

    fn hash(&self) -> u64 {
        // The piece moved last can't move or be thrown this turn, so where it stands is part
        // of the position. Mixed, so it reaches the top bits the table indexes with.
        match self.turn_history.last() {
            Some(&Turn::Move(_, end)) => self.zobrist_hash ^ u64::from(end).splitmix(),
            _ => self.zobrist_hash,
        }
    }

    fn ply(&self) -> u32 {
        self.turn_num
    }

    fn undo(&mut self) {
        let turn = self
            .turn_history
            .pop()
            .expect("undo called with no move to undo");
        self.zobrist_history.pop();
        self.zobrist_hash ^= TO_MOVE_KEY;
        self.turn_num -= 1;
        match turn {
            Turn::Place(hex, bug) => {
                self.remove(hex);
                self.mut_remaining()[bug as usize] += 1;
            }
            Turn::Move(start, end) => {
                let (bug, bug_num, color) = self.remove(end);
                self.insert(start, bug, bug_num, color);
            }
            Turn::Pass => {}
        }
    }
}
