use crate::{Round, PokerError};


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Position {
    pos: u8,
    //n_players: u8,
    // SmallBlind = 0,
    // BigBlind,
    // Utg,
    // HiJack,
    // Button,
}

pub  const SMALL_BLIND: Position = Position { pos: 0 };
pub const BIG_BLIND: Position = Position { pos: 1 };
const UTG: Position = Position { pos: 2 };

const MAX_POSITION: u8 = 15;            

impl Position {

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
            Position {
                pos: n_players - 1,
            }
        } else {
            Position {
                pos: self.pos - 1,
            }
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