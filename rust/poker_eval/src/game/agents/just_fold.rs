use crate::{ActionEnum, GameState, HoleCards, PlayerState};

use super::{Agent, AgentDecision};

#[derive(Default)]
pub struct JustFold {
    hole_cards: Option<HoleCards>,
    name: String,
}

impl Agent for JustFold {
    fn decide(&self, _player_state: &PlayerState, _game_state: &GameState) -> AgentDecision {
        AgentDecision {
            action: ActionEnum::Fold,
            comment: None,
        }
    }

    fn get_hole_cards(&self) -> crate::HoleCards {
        self.hole_cards.unwrap()
    }

    fn set_hole_cards(&mut self, hole_cards: HoleCards) {
        self.hole_cards = Some(hole_cards);
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}