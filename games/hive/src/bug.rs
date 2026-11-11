/// The kinds of piece. Mosquito, ladybug and pillbug are the expansions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bug {
    Queen = 0,
    Grasshopper = 1,
    Spider = 2,
    Ant = 3,
    Beetle = 4,
    Mosquito = 5,
    Ladybug = 6,
    Pillbug = 7,
}

impl Bug {
    /// In the order of their values.
    pub const ALL: [Bug; 8] = [
        Bug::Queen,
        Bug::Grasshopper,
        Bug::Spider,
        Bug::Ant,
        Bug::Beetle,
        Bug::Mosquito,
        Bug::Ladybug,
        Bug::Pillbug,
    ];

    /// The letter a move names this bug with.
    pub fn letter(self) -> char {
        match self {
            Bug::Queen => 'Q',
            Bug::Grasshopper => 'G',
            Bug::Spider => 'S',
            Bug::Ant => 'A',
            Bug::Beetle => 'B',
            Bug::Mosquito => 'M',
            Bug::Ladybug => 'L',
            Bug::Pillbug => 'P',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Bug::Queen => "queen",
            Bug::Grasshopper => "grasshopper",
            Bug::Spider => "spider",
            Bug::Ant => "ant",
            Bug::Beetle => "beetle",
            Bug::Mosquito => "mosquito",
            Bug::Ladybug => "ladybug",
            Bug::Pillbug => "pillbug",
        }
    }

    /// The bug named by `letter`, in either case.
    pub fn from_letter(letter: char) -> Option<Bug> {
        Bug::ALL
            .into_iter()
            .find(|bug| bug.letter() == letter.to_ascii_uppercase())
    }

    /// Whether the bug only moves itself by crawling.
    pub fn crawler(self) -> bool {
        matches!(self, Bug::Ant | Bug::Queen | Bug::Spider | Bug::Pillbug)
    }

    /// The emoji the board draws this bug with.
    pub fn codepoint(self) -> char {
        match self {
            Bug::Queen => '\u{1f41d}',       // HONEYBEE
            Bug::Grasshopper => '\u{1f997}', // CRICKET
            Bug::Spider => '\u{1f577}',      // SPIDER
            Bug::Ant => '\u{1f41c}',         // ANT
            Bug::Beetle => '\u{1fab2}',      // BEETLE
            Bug::Mosquito => '\u{1f99f}',    // MOSQUITO
            Bug::Ladybug => '\u{1f41e}',     // LADY BEETLE
            Bug::Pillbug => '\u{1f48a}',     // PILL
        }
    }
}
