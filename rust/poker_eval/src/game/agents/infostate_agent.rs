use std::cell::RefCell;
use std::rc::Rc;

use boomphf::Mphf;

use crate::game::agents::{Agent, InfoStateDb, InfoState, info_state_actions};
use crate::board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval};
use crate::game::core::{CommentedAction, GameState, PlayerState, ActionEnum};
use crate::HoleCards;
use crate::pre_calc::perfect_hash::load_boomperfect_hash;


pub struct InfoStateAgent {
    pub hole_cards: Option<HoleCards>,
    pub name: String,

    info_state_db: Rc<RefCell<InfoStateDb>>,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    hash_func: Mphf<u32>,
}

impl InfoStateAgent {
    pub fn new(name: &str,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
        info_state_db: Rc<RefCell<InfoStateDb>>
    ) -> Self {
        Self {
            name: name.to_string(),
            hole_cards: None,
            monte_carlo_db,
            info_state_db,
            hash_func: load_boomperfect_hash(),
        }
    }

}

impl Agent for InfoStateAgent {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        let info_state = 
        InfoState::from_game_state(game_state, player_state,self.hole_cards.as_ref().unwrap(), self.monte_carlo_db.clone());

        let info_state_db = self.info_state_db.borrow();

        let action_values = info_state_db.get(&info_state).unwrap();

        if action_values.is_none() {
            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some(format!("Infostate {} did not exist, so folding", &info_state))
            };
        }

        let action_values = action_values.unwrap();

        assert_eq!(action_values.len(), info_state_actions::NUM_ACTIONS);

        let max_action_index = action_values.iter()
        .enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0 as u8;

        let normalized = InfoStateDb::normalize_array(&action_values);
        let common_comment = InfoStateDb::normalized_array_to_string(&normalized);

        let helpers = player_state.get_helpers(game_state);

        match max_action_index {
            info_state_actions::FOLD => {
                CommentedAction {
                    action: ActionEnum::Fold,
                    comment: Some(format!("Infostate {} folded {}", &info_state, &common_comment))
                }
            },
            info_state_actions::CHECK => {
                CommentedAction {
                    action: ActionEnum::Check,
                    comment: Some(format!("Infostate {} checked {}", &info_state, &common_comment))
                }
            },
            info_state_actions::CALL => {
                CommentedAction {
                    action: ActionEnum::Call(helpers.call_amount),
                    comment: Some(format!("Infostate {} called {}", &info_state, &common_comment))
                }
            },
            info_state_actions::BET_HALF => {
                helpers.build_bet(game_state.pot() / 2, 
                    format!("Infostate {} bet {}", &info_state, &common_comment))
                
            },
            info_state_actions::BET_POT => {
                helpers.build_bet(game_state.pot(), 
                    format!("Infostate {} bet {}", &info_state, &common_comment))
            },
            info_state_actions::RAISE_3X => {
                helpers.build_raise_to(game_state, game_state.current_to_call * 3, 
                    format!("Infostate {} raised {}", &info_state, &common_comment))
                
            },
            info_state_actions::ALL_IN => {
                if game_state.current_to_call == 0 {
                    helpers.build_bet( helpers.max_can_raise, 
                        format!("Infostate {} Bet All In {}", &info_state, &common_comment))
                } else {
                    helpers.build_raise_to(game_state, helpers.max_can_raise, 
                        format!("Infostate {} Raise All In {}", &info_state, &common_comment))
                }
            },
            _ => {
                panic!("Unknown action index {}", max_action_index);
            }
        
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
