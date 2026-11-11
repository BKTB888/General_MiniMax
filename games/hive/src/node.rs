use crate::{bug::Bug, hex_grid::Hex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// The other color, as an index.
    pub fn other(self) -> usize {
        1 - self as usize
    }
}

/// A piece, or nothing, in a byte: bit 7 is the color, bits 4-6 the bug, bits 2-3 its number,
/// and bits 0-1 its height, capped at 3. All zeros is empty.
#[derive(Clone, Copy)]
pub struct Node(u8);

impl Node {
    pub const EMPTY: Self = Node(0);

    /// A piece at `height`, which is stored capped at 3.
    pub fn new(bug: Bug, color: Color, bug_num: u8, height: u8) -> Self {
        Node((color as u8) << 7 | (bug as u8) << 4 | bug_num << 2 | height.min(3))
    }

    /// This piece at `height` instead, capped at 3.
    fn with_height(self, height: u8) -> Self {
        Node(self.0 & !3 | height.min(3))
    }

    pub fn color(self) -> Color {
        // SAFETY: `Color` has a variant for both values of the 1 bit.
        unsafe { std::mem::transmute::<u8, Color>(self.0 >> 7) }
    }

    pub fn bug(self) -> Bug {
        // SAFETY: `Bug` has a variant for all 8 values of the 3 bits.
        unsafe { std::mem::transmute::<u8, Bug>(self.0 >> 4 & 7) }
    }

    pub fn bug_num(self) -> u8 {
        self.0 >> 2 & 3
    }

    pub fn occupied(self) -> bool {
        self.0 != 0
    }

    pub fn is_stacked(self) -> bool {
        self.clipped_height() > 1
    }

    pub fn clipped_height(self) -> u8 {
        self.0 & 3
    }
}

/// A piece covered by another, with where it is and how high.
#[derive(Clone, Copy)]
pub struct UnderNode {
    pub node: Node,
    pub hex: Hex,
    pub height: u8,
}

impl UnderNode {
    pub fn new(node: Node, hex: Hex, height: u8) -> Self {
        Self {
            node: node.with_height(height),
            hex,
            height,
        }
    }
}
