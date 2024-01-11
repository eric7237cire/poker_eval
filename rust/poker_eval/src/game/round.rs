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

    pub fn get_num_board_cards(&self) -> usize {
        match self {
            Round::Preflop => 0,
            Round::Flop => 3,
            Round::Turn => 4,
            Round::River => 5,
        }
    }
}

impl Into<usize> for Round {
    fn into(self) -> usize {
        self as usize
    }
}

impl TryFrom<u8> for Round {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Round::Preflop),
            1 => Ok(Round::Flop),
            2 => Ok(Round::Turn),
            3 => Ok(Round::River),
            _ => Err(format!("Invalid round {}", value)),
        }
    }
}