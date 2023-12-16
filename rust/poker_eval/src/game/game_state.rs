use crate::ChipType;

pub struct GameState {
    //pot from prev. betting rounds
    pub current_pot: ChipType,
}

impl Default for GameState {
    fn default() -> Self {
        GameState { current_pot: 0 }
    }
}

impl GameState {
    pub fn new(_num_players: u8) -> Self {
        GameState { current_pot: 0 }
    }
}
