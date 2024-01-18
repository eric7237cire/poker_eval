use std::{cell::RefCell, cmp::min, rc::Rc};

use boomphf::Mphf;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    likes_hands::{likes_hand, LikesHandLevel, LikesHandResponse},
    pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash},
     HoleCards, 
};
use crate::game::core::{PlayerState, GameState, CommentedAction, ActionEnum, Round, PositionFamily};
use super::Agent;

pub struct PanicAgent {
    pub hole_cards: Option<HoleCards>,
    pub name: String,
}

impl PanicAgent {
    pub fn new(
        name: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            hole_cards: None,
        }
    }

    fn decide_preflop(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        panic!("Panic Agent");
    }

    
}

impl Agent for PanicAgent {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
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
