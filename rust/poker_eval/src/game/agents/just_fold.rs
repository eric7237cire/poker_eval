use crate::{ActionEnum, PlayerState, GameState, HoleCards};

use super::Agent;

pub struct JustFold {
    hole_cards: HoleCards,
}

impl Agent for JustFold {
    fn decide(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> ActionEnum {
        ActionEnum::Fold
    }

    fn get_hole_cards(&self) -> crate::HoleCards {
        self.hole_cards
    }
}
