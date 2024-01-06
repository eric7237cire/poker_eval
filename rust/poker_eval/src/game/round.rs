use core::fmt;
use std::fmt::Display;

use serde::Serialize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Serialize)]
#[repr(u8)]
pub enum Round {
    Preflop = 0,
    Flop = 1,
    Turn = 2,
    River = 3,
}

impl Default for Round {
    fn default() -> Self {
        Round::Preflop
    }
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
