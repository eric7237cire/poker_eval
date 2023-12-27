use crate::{ActionEnum, CommentedAction, GameState, HoleCards, PlayerState};

use super::Agent;

#[derive(Default)]
pub struct JustFold {
    hole_cards: Option<HoleCards>,
    name: String,
}

impl Agent for JustFold {
    fn decide(&mut self, _player_state: &PlayerState, _game_state: &GameState) -> CommentedAction {
        CommentedAction {
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
