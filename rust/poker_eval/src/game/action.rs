use crate::ChipType;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Action {
    Fold,
    Call,
    Check,
    //Value is the new total, which may include what the player already bet
    Raise(ChipType),
}
