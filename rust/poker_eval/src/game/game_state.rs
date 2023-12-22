use crate::Position;

use crate::ChipType;

pub struct PlayerState {
    pub stack: ChipType,
    pub folded: bool,

    //Not yet taken from stack
    pub cur_round_put_in_pot: ChipType,

    //In current betting round, so == remaining stack
    pub all_in_for : ChipType,

    //Used in all in, to see how much they can win
    pub max_pot: ChipType,
}

pub struct GameState {
    //sb first order
    pub player_states: Vec<PlayerState>,

    pub current_to_act: Position,

    pub pot: ChipType,

    //Until current rounds are finished, is not added to pot
    pub round_pot: ChipType,

    pub current_to_call: ChipType,
}

pub struct OldGameState {
    //pot from prev. betting rounds
    pub current_pot: ChipType,
}

impl Default for OldGameState {
    fn default() -> Self {
        OldGameState { current_pot: 0 }
    }
}

impl OldGameState {
    pub fn new(_num_players: u8) -> Self {
        OldGameState { current_pot: 0 }
    }
}
