use crate::{ActionEnum, GameState, HoleCards, PlayerState};

use super::Agent;

pub struct JustFold {
    hole_cards: HoleCards,
}

impl Agent for JustFold {
    fn decide(&self, _player_state: &PlayerState, _game_state: &GameState) -> ActionEnum {
        ActionEnum::Fold
    }

    fn get_hole_cards(&self) -> crate::HoleCards {
        self.hole_cards
    }
}
