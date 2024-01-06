use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::{ChipType, Round};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize)]
pub enum ActionEnum {
    Fold,
    //This is the difference between what they already put in the pot and what they need to put in the pot
    Call(ChipType),
    Check,
    Bet(ChipType),
    //1st value is the increase, 2nd Value is the new total
    Raise(ChipType, ChipType),
}

impl Default for ActionEnum {
    fn default() -> Self {
        ActionEnum::Fold
    }
}

impl Display for ActionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionEnum::Fold => write!(f, "folds"),
            ActionEnum::Call(amount) => write!(f, "calls {}", amount),
            ActionEnum::Check => write!(f, "checks"),
            ActionEnum::Bet(amount) => write!(f, "bets {}", amount),
            ActionEnum::Raise(increase, amount) => write!(f, "raises {} to {}", increase, amount),
        }
    }
}

pub struct CommentedAction {
    pub action: ActionEnum,
    pub comment: Option<String>,
}

impl Display for CommentedAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} # {}",
            self.action,
            self.comment.as_ref().unwrap_or(&"".to_string())
        )
    }
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Clone)]
pub struct PlayerAction {
    pub player_index: usize,
    pub action: ActionEnum,
    pub round: Round,
    pub player_comment: Option<String>,
    pub system_comment: Option<String>,
}

impl Display for PlayerAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Player #{} {} in {} -- {} ; {}",
            self.player_index,
            self.action,
            self.round,
            self.system_comment.as_ref().unwrap_or(&"".to_string()),
            self.player_comment.as_ref().unwrap_or(&"".to_string())
        )
    }
}
