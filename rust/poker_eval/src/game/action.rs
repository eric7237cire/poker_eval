use std::fmt::{Display, Formatter};

use crate::{ChipType, Round};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ActionEnum {
    Fold,
    Call,
    Check,
    Bet(ChipType),
    //Value is the new total, which may include what the player already bet
    Raise(ChipType),
}

impl Display for ActionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionEnum::Fold => write!(f, "Fold"),
            ActionEnum::Call => write!(f, "Call"),
            ActionEnum::Check => write!(f, "Check"),
            ActionEnum::Bet(amount) => write!(f, "Bet {}", amount),
            ActionEnum::Raise(amount) => write!(f, "Raise {}", amount),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PlayerAction {
    pub player_index: usize,
    pub action: ActionEnum,
    pub round: Round,

    pub comment: Option<String>,
}

impl Display for PlayerAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Player #{} {} in {} -- {}",
            self.player_index,
            self.action,
            self.round,
            self.comment.as_ref().unwrap_or(&"".to_string())
        )
    }
}
