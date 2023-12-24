use crate::{ActionEnum, GameState, HoleCards, PlayerState};

use super::Agent;

#[derive(Default)]
pub struct JustFold {
    hole_cards: Option<HoleCards>,
}

impl Agent for JustFold {
    fn decide(&self, _player_state: &PlayerState, _game_state: &GameState) -> ActionEnum {
        ActionEnum::Fold
    }

    fn get_hole_cards(&self) -> crate::HoleCards {
        self.hole_cards.unwrap()
    }

    fn set_hole_cards(&mut self, hole_cards: HoleCards) {
        self.hole_cards = Some(hole_cards);
    }
}
