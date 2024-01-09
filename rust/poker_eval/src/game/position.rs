use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::{PokerError, Round};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize)]
pub struct Position {
    pos: u8,
    //n_players: u8,
    // SmallBlind = 0,
    // BigBlind,
    // Utg,
    // HiJack,
    // Button,
}

pub const SMALL_BLIND: Position = Position { pos: 0 };
pub const BIG_BLIND: Position = Position { pos: 1 };
const UTG: Position = Position { pos: 2 };

const MAX_POSITION: u8 = 15;

pub enum PositionFamily {
    UTG,
    Middle,
    Late,
    Button,
    Blinds
}

impl Position {

    pub fn get_position_family(&self, num_players: u8) -> PositionFamily {
        
        if self.pos == 0 || self.pos == 1 {
            return PositionFamily::Blinds;
        }

        if self.pos == num_players - 1 {
            return PositionFamily::Button;
        }

        if self.pos == 2 {
            return PositionFamily::UTG;
        }

        assert!(num_players >= 5);

        if num_players == 5 {
            //sb bb utg mp button
            assert_eq!(self.pos,3);
            return PositionFamily::Middle;            
        }

        if num_players == 6 {
            //sb bb utg mp lp button
            if self.pos == 3 {
                return PositionFamily::Middle;
            } else {
                assert_eq!(self.pos,4);
                return PositionFamily::Late;
            } 
        }

        if num_players == 7 {
            //sb bb utg mp mp2 lp button
            if self.pos == 3 {
                return PositionFamily::Middle;
            } else if self.pos == 4 {
                return PositionFamily::Middle;
            } else {
                assert_eq!(self.pos,5);
                return PositionFamily::Late;
            } 
        }

        if num_players == 8 {
            //sb bb utg utg mp mp2 lp button
            if self.pos == 3 {
                return PositionFamily::UTG;
            } else if self.pos <=5 {
                return PositionFamily::Middle;
            } else {
                assert_eq!(self.pos,6);
                return PositionFamily::Late;
            } 
        }

        //sb bb utg utg (utg) mp mp2 lp lp button
        if self.pos >= num_players - 3 {
            return PositionFamily::Late;
        } else if self.pos >= num_players - 5 {
            return PositionFamily::Middle;
        } else {
            return PositionFamily::UTG;
        } 

    }

    pub fn first_to_act(n_players: u8, round: Round) -> Position {
        assert!(n_players >= 2);

        if n_players == 2 {
            if round == Round::Preflop {
                //the small blind is the 'button' in heads up, acting last all rounds except preflop
                return SMALL_BLIND;
            } else {
                return BIG_BLIND;
            }
        }

        if round == Round::Preflop {
            return UTG;
        } else {
            return SMALL_BLIND;
        }
    }

    pub fn next(&self, n_players: u8) -> Position {
        Position {
            pos: (self.pos + 1) % n_players,
        }
    }

    pub fn prev(&self, n_players: u8) -> Position {
        if self.pos == 0 {
            Position { pos: n_players - 1 }
        } else {
            Position { pos: self.pos - 1 }
        }
    }
}

impl TryFrom<usize> for Position {
    type Error = PokerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > MAX_POSITION as usize {
            Err(PokerError::from_string(format!(
                "Invalid position {}",
                value
            )))
        } else {
            Ok(Position { pos: value as u8 })
        }
    }
}

/*
impl Into<usize> for Position {
    fn into(self) -> usize {
        self.pos as usize
    }
}
 */

impl From<Position> for usize {
    fn from(value: Position) -> Self {
        value.pos as usize
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.pos {
            0 => write!(f, "SB (1st)"),
            1 => write!(f, "BB (2nd)"),
            2 => write!(f, "UTG (3rd)"),
            _ => write!(f, "{}th", self.pos),
        }
    }
}

impl Display for PositionFamily {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionFamily::UTG => write!(f, "UTG"),
            PositionFamily::Middle => write!(f, "Middle"),
            PositionFamily::Late => write!(f, "Late"),
            PositionFamily::Button => write!(f, "Button"),
            PositionFamily::Blinds => write!(f, "Blinds"),
        }
    }
}