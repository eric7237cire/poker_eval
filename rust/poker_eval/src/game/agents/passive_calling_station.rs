use std::{cell::RefCell, rc::Rc};

use boomphf::Mphf;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    ActionEnum, BoolRange, CardValue, CommentedAction, FlushDrawType, GameState, HoleCards,
    PlayerState, Round, StraightDrawType, pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash}, likes_hands::{likes_hand, LikesHandLevel},
};

use super::Agent;

//#[derive(Default)]
pub struct PassiveCallingStation {
    pub calling_range: Option<BoolRange>,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    hash_func: Mphf<u32>,
}

impl PassiveCallingStation {
    pub fn new(
        calling_range_str: Option<&str>,
        name: &str,
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    ) -> Self {
        let calling_range = match calling_range_str {
            Some(s) => Some(s.parse().unwrap()),
            None => None,
        };

        PassiveCallingStation {
            calling_range,
            hole_cards: None,
            name: name.to_string(),
            partial_rank_db,
            flop_texture_db,
            hash_func: load_boomperfect_hash(),
        }
    }

    fn decide_postflop(
        &mut self,
        _player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        if game_state.current_to_call == 0 {
            return CommentedAction {
                action: ActionEnum::Check,
                comment: Some("Checking".to_string()),
            };
        }

        //We'll call with literally anything interesting
        let hc = self.hole_cards.as_ref().unwrap();
        let mut pr_db = self.partial_rank_db.borrow_mut();
        let prc = pr_db.get_put(&game_state.board, hc).unwrap();
        let mut ft_db = self.flop_texture_db.borrow_mut();
        let ft = ft_db.get_put(&game_state.board).unwrap();

        let rank = fast_hand_eval(
            game_state.board.get_iter().chain(hc.get_iter()),
            &self.hash_func,
        );
        
        let likes_hand_response = likes_hand(&prc, &ft, &rank, &game_state.board, &hc).unwrap();

        let half_pot = game_state.pot() / 2;

        if game_state.current_to_call <= half_pot && likes_hand_response.likes_hand >= LikesHandLevel::CallSmallBet {
            return CommentedAction {
                action: ActionEnum::Call,
                comment: Some(format!("Calling <= half pot bet with {}.  +1 {} -1 {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),	
                    likes_hand_response.not_like_hand_comments.join(", ")))
            };
        } else if likes_hand_response.likes_hand >= LikesHandLevel::LargeBet {
            return CommentedAction {
                action: ActionEnum::Call,
                comment: Some(format!("Calling any pot with {}.  +1 {} -1 {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),	
                    likes_hand_response.not_like_hand_comments.join(", ")))
            };
        }
        else {
            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some(format!("Folding with {}.  +1 {} -1 {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),	
                    likes_hand_response.not_like_hand_comments.join(", ")))
            };
        }
    }
}

impl Agent for PassiveCallingStation {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        let action = match game_state.current_round {
            Round::Preflop => {
                let ri = self.hole_cards.unwrap().to_range_index();
                //not handling all ins
                if let Some(calling_range) = self.calling_range.as_ref() {
                    if calling_range.data[ri] {
                        ActionEnum::Call
                    } else {
                        ActionEnum::Fold
                    }
                } else {
                    ActionEnum::Call
                }
            }
            _ => {
                return self.decide_postflop(player_state, game_state);
            }
        };

        CommentedAction {
            action,
            comment: None,
        }
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
