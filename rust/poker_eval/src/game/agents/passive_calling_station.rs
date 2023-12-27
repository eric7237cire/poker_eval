use std::{cell::RefCell, rc::Rc};

use crate::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    ActionEnum, CommentedAction, FlushDrawType, GameState, HoleCards, PartialRankContainer,
    PlayerState, Round, StraightDrawType,
};
use postflop_solver::Range;

use super::Agent;

//#[derive(Default)]
pub struct PassiveCallingStation<'a> {
    pub calling_range: Option<Range>,
    pub hole_cards: Option<HoleCards>,
    pub name: String,

    partial_rank_db:
        Rc<RefCell<&'a mut EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>>,
}

impl<'a> PassiveCallingStation<'a> {
    pub fn new(
        calling_range_str: Option<&str>,
        name: &str,
        partial_rank_db: Rc<
            RefCell<&'a mut EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>,
        >,
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

        let mut likes_hand_comments: Vec<String> = Vec::new();

        if let Some(p) = prc.lo_pair {
            //if p.number_above == 0 {
            likes_hand_comments.push(format!("lo pair {}", hc.get_hi_card().value));
            //}
            if p.made_quads {
                likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
            }
            if p.made_set {
                likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
            }
        }
        if let Some(p) = prc.hi_pair {
            //if p.number_above == 0 {
            likes_hand_comments.push(format!("pair {}", hc.get_hi_card().value));
            //}
            if p.made_quads {
                likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
            }
            if p.made_set {
                likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
            }
        }
        if let Some(p) = prc.pocket_pair {
            //if p.number_above == 0 {
            likes_hand_comments.push(format!("pocket pair {}", hc.get_hi_card().value));
            //}
            if p.made_set {
                likes_hand_comments.push(format!("Pocket Pair Set {}", hc.get_hi_card().value));
            }
            if p.made_quads {
                likes_hand_comments.push(format!("Pocket Pair Quads {}", hc.get_hi_card().value));
            }
        }
        if game_state.current_round != Round::River {
            if let Some(p) = prc.flush_draw {
                if p.flush_draw_type == FlushDrawType::FlushDraw {
                    likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
                }
            }
            if let Some(p) = prc.straight_draw {
                if p.straight_draw_type == StraightDrawType::OpenEnded
                    || p.straight_draw_type == StraightDrawType::DoubleGutShot
                {
                    likes_hand_comments.push(format!("Straight draw"));
                }
                //likes_hand_comments.push( format!("Gutshot straight draw {}", p.) );
            }
        }

        if likes_hand_comments.len() > 0 {
            return CommentedAction {
                action: ActionEnum::Call,
                comment: Some(likes_hand_comments.join(", ")),
            };
        } else {
            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some("Folding, nothing interesting".to_string()),
            };
        }
    }
}

impl<'a> Agent for PassiveCallingStation<'a> {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        let action = match game_state.current_round {
            Round::Preflop => {
                let ri = self.hole_cards.unwrap().to_range_index();
                //not handling all ins
                if let Some(calling_range) = self.calling_range {
                    if calling_range.data[ri] > 0.0 {
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
