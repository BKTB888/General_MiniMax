use std::{
    fmt::{self, Display, Formatter, Write},
    str::FromStr,
};

use crossterm::style::{self, Stylize};

use super::{HiveState, Turn};
use crate::{
    bug::Bug,
    hex_grid::{Hex, Loc, ROW_SIZE, START_HEX, hex_to_loc, loc_to_hex},
    node::Color,
};

/// An empty hex, as wide as a bug.
const FREE_SPACE: char = '\u{ff0e}';
pub(crate) const MARK_BG: style::Color = style::Color::DarkGreen;
pub(crate) const SELECTED_BG: style::Color = style::Color::DarkYellow;
pub(crate) const LAST_MOVE_COLOR: style::Color = style::Color::DarkRed;

/// `A (1, -1)` places an ant, `(0, 0) -> (1, 0)` moves, and `pass` passes.
impl Display for Turn {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Turn::Place(hex, bug) => write!(f, "{} {}", bug.letter(), hex_to_loc(hex)),
            Turn::Move(start, end) => write!(f, "{} -> {}", hex_to_loc(start), hex_to_loc(end)),
            Turn::Pass => f.write_str("pass"),
        }
    }
}

#[derive(Debug)]
pub struct ParseTurnError;

impl FromStr for Turn {
    type Err = ParseTurnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("pass") {
            return Ok(Turn::Pass);
        }
        if let Some((start, end)) = s.split_once("->") {
            return Ok(Turn::Move(parse_hex(start)?, parse_hex(end)?));
        }
        let mut chars = s.chars();
        let bug = chars
            .next()
            .and_then(Bug::from_letter)
            .ok_or(ParseTurnError)?;
        Ok(Turn::Place(parse_hex(chars.as_str())?, bug))
    }
}

fn parse_hex(s: &str) -> Result<Hex, ParseTurnError> {
    let loc: Loc = s.parse().map_err(|_| ParseTurnError)?;
    Ok(loc_to_hex(loc))
}

/// The hive in staggered rows, White's bugs inverted, the last bug moved or placed on a red
/// background and the hex it left red too, as a red dot if empty, then a line each of the bugs
/// White and Black have left.
impl<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool> Display
    for HiveState<MOSQUITO, LADYBUG, PILLBUG>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.write_board(f, &[], None)?;
        for color in [Color::White, Color::Black] {
            self.write_remaining(f, color)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>
    HiveState<MOSQUITO, LADYBUG, PILLBUG>
{
    /// The board as `Display` draws it, with `marks` on a green background and `selected` on a
    /// yellow one, both over the last move's red.
    pub(crate) fn marked(&self, marks: &[Hex], selected: Option<Hex>) -> String {
        let mut board = String::new();
        self.write_board(&mut board, marks, selected)
            .expect("a String takes any write");
        board
    }

    /// The columns and rows the board takes in a terminal.
    pub(crate) fn board_size(&self) -> (u16, u16) {
        let (startr, endr, startc, endc) = self.bounding_box();
        let rows = endr.wrapping_sub(startr) % ROW_SIZE + 1;
        let cols = endc.wrapping_sub(startc) % ROW_SIZE + 1;
        // Each hex is 2 columns, and the rows are shifted by a column each.
        (2 * cols + rows - 1, rows)
    }

    /// The hex drawn at column `x` and row `y` of the board, counted from its top left.
    pub(crate) fn hex_at(&self, x: u16, y: u16) -> Option<Hex> {
        let (startr, endr, startc, endc) = self.bounding_box();
        let rows = endr.wrapping_sub(startr) % ROW_SIZE + 1;
        let cols = endc.wrapping_sub(startc) % ROW_SIZE + 1;
        if y >= rows {
            return None;
        }
        let r = (startr + y) % ROW_SIZE;
        let indent = endr.wrapping_sub(r) % ROW_SIZE;
        let col = x.checked_sub(indent)? / 2;
        if col >= cols {
            return None;
        }
        Some((startc + col) % ROW_SIZE + r * ROW_SIZE)
    }

    fn write_board(&self, f: &mut impl Write, marks: &[Hex], selected: Option<Hex>) -> fmt::Result {
        let (startr, endr, startc, endc) = self.bounding_box();
        let (from, to) = self.last_move();

        let mut r = startr;
        while r != (endr + 1) % ROW_SIZE {
            // Shifted half a hex per row, into a hex grid.
            let indent = endr.wrapping_sub(r) % ROW_SIZE;
            if indent % 2 == 1 {
                f.write_char(' ')?;
            }
            for _ in 0..indent / 2 {
                f.write_char(FREE_SPACE)?;
            }

            let mut c = startc;
            while c != (endc + 1) % ROW_SIZE {
                let hex = c + r * ROW_SIZE;
                let node = self.node(hex);
                let background = if selected == Some(hex) {
                    Some(SELECTED_BG)
                } else if marks.contains(&hex) {
                    Some(MARK_BG)
                } else if to == Some(hex) || (from == Some(hex) && node.occupied()) {
                    Some(LAST_MOVE_COLOR)
                } else {
                    None
                };
                if let Some(background) = background {
                    // Not inverted for White: that would swap the background in.
                    let cell = if node.occupied() {
                        node.bug().codepoint()
                    } else {
                        FREE_SPACE
                    };
                    write!(f, "{}", cell.on(background))?;
                } else if node.occupied() {
                    write_bug(f, node.bug(), node.color())?;
                } else if from == Some(hex) {
                    write!(f, "{}", FREE_SPACE.with(LAST_MOVE_COLOR))?;
                } else {
                    f.write_char(FREE_SPACE)?;
                }
                c = (c + 1) % ROW_SIZE;
            }

            f.write_char('\n')?;
            r = (r + 1) % ROW_SIZE;
        }
        Ok(())
    }

    /// Where the last turn's bug came from and went to. A placed bug comes from nowhere.
    fn last_move(&self) -> (Option<Hex>, Option<Hex>) {
        match self.turn_history.last() {
            Some(&Turn::Place(hex, _)) => (None, Some(hex)),
            Some(&Turn::Move(start, end)) => (Some(start), Some(end)),
            Some(&Turn::Pass) | None => (None, None),
        }
    }

    /// The first and last row, then the first and last column, of the hive with an empty one
    /// around it. The last can be less than the first, when the hive wraps around the grid.
    fn bounding_box(&self) -> (Hex, Hex, Hex, Hex) {
        let empty_rows: Vec<bool> = (0..ROW_SIZE)
            .map(|r| (0..ROW_SIZE).all(|c| !self.occupied(r * ROW_SIZE + c)))
            .collect();
        let empty_cols: Vec<bool> = (0..ROW_SIZE)
            .map(|c| (0..ROW_SIZE).all(|r| !self.occupied(r * ROW_SIZE + c)))
            .collect();
        if empty_rows.iter().all(|&r| r) {
            return (
                START_HEX / ROW_SIZE - 1,
                START_HEX / ROW_SIZE + 1,
                START_HEX % ROW_SIZE - 1,
                START_HEX % ROW_SIZE + 1,
            );
        }
        let (mut minr, mut maxr, mut minc, mut maxc) = (0, 0, 0, 0);
        for i in 0..ROW_SIZE as usize {
            let j = (i + 1) % ROW_SIZE as usize;
            if empty_rows[i] && !empty_rows[j] {
                minr = i as Hex;
            }
            if !empty_rows[i] && empty_rows[j] {
                maxr = j as Hex;
            }
            if empty_cols[i] && !empty_cols[j] {
                minc = i as Hex;
            }
            if !empty_cols[i] && empty_cols[j] {
                maxc = j as Hex;
            }
        }
        (minr, maxr, minc, maxc)
    }

    fn write_remaining(&self, f: &mut impl Write, color: Color) -> fmt::Result {
        for bug in Bug::ALL {
            let count = self.remaining[color as usize][bug as usize];
            if count == 0 {
                continue;
            }
            if count > 1 {
                write!(f, "{count}")?;
            } else {
                f.write_char(' ')?;
            }
            write_bug(f, bug, color)?;
            f.write_char(' ')?;
        }
        Ok(())
    }
}

fn write_bug(f: &mut impl Write, bug: Bug, color: Color) -> fmt::Result {
    if color == Color::White {
        write!(f, "{}", bug.codepoint().reverse())
    } else {
        f.write_char(bug.codepoint())
    }
}
