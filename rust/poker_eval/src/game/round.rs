use core::fmt;
use std::fmt::Display;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Round {
    Preflop,
    Flop,
    Turn,
    River,
}

impl Display for Round {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Round::Preflop => write!(f, "Preflop"),
            Round::Flop => write!(f, "Flop"),
            Round::Turn => write!(f, "Turn"),
            Round::River => write!(f, "River"),
        }
    }
}

impl Round {
    pub fn next(&self) -> Option<Round> {
        match self {
            Round::Preflop => Some(Round::Flop),
            Round::Flop => Some(Round::Turn),
            Round::Turn => Some(Round::River),
            Round::River => None,
        }
    }
}

impl Into<usize> for Round {
    fn into(self) -> usize {
        self as usize
    }
}
