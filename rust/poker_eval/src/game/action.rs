
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

#[derive(Debug, PartialEq, Eq)]
pub struct PlayerAction {
    pub player_index: usize,
    pub action: ActionEnum,
    pub round: Round,
}
