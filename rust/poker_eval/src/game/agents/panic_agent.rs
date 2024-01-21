use super::Agent;
use crate::game::core::{CommentedAction, GameState, PlayerState};
use crate::HoleCards;

pub struct PanicAgent {
    pub hole_cards: Option<HoleCards>,
    pub name: String,
}

impl PanicAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hole_cards: None,
        }
    }

}

impl Agent for PanicAgent {
    fn decide(&mut self, _player_state: &PlayerState, _game_state: &GameState) -> CommentedAction {
        panic!("Panic Agent");
    }

    fn get_hole_cards(&self) -> HoleCards {
        self.hole_cards.unwrap()
    }

    fn set_hole_cards(&mut self, hole_cards: HoleCards) {
        self.hole_cards = Some(hole_cards);
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
