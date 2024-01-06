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

    //this is before their raise or bet has been added
    pub pot : ChipType,
    //The total amount to call, so the amount they need to put in is this - amount_put_in_pot_this_round
    pub current_amt_to_call : ChipType,
    //this is the amount they put in the pot this round, before this action
    pub amount_put_in_pot_this_round : ChipType,
    //this is the amount they put in the pot the entire hand, before this action
    pub total_amount_put_in_pot : ChipType,

    //will be 0 when this player action could have closed the action for the round
    pub players_left_to_act: u8,

    //Should be if this action put the player all in, either with bet/call/raise
    pub is_all_in: bool,
}

impl Display for PlayerAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Player #{} {} in {} -- {}",
            self.player_index,
            self.action,
            self.round,
            self.player_comment.as_ref().unwrap_or(&"".to_string())
        )
    }
}
