use crate::{ActionEnum, GameState, HoleCards, PlayerState, Round, board_hc_eval_cache_redb::{EvalCacheWithHcReDb, PARTIAL_RANK_PATH, ProducePartialRankCards}, PartialRankContainer, CommentedAction};
use postflop_solver::Range;

use super::{Agent};

//#[derive(Default)]
pub struct PassiveCallingStation {
    pub calling_range: Option<Range>,
    pub hole_cards: Option<HoleCards>,
    pub name: String,

    partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>,
}

impl PassiveCallingStation {
    pub fn new(
        calling_range_str: Option<&str>,
        name: &str,
    ) -> Self {
        let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards, _> =
        EvalCacheWithHcReDb::new(PARTIAL_RANK_PATH).unwrap();
        
        let calling_range = match calling_range_str {
            Some(s) => Some(s.parse().unwrap()),
            None => None
        };
        
        PassiveCallingStation {
            calling_range,  
            hole_cards: None,
            name: name.to_string(),
            partial_rank_db
        }
    }
}

impl Agent for PassiveCallingStation {
    fn decide(&mut self, _player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
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
                if game_state.current_to_call == 0 {
                    ActionEnum::Check
                } else {
                    ActionEnum::Call
                }
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
