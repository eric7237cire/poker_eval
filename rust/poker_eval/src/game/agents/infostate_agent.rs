use std::cell::RefCell;
use std::rc::Rc;

use crate::board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval};
use crate::game::agents::info_state::{
    info_state_actions, InfoState, InfoStateDb, InfoStateDbTrait,
};
use crate::game::agents::Agent;
use crate::game::core::{ActionEnum, CommentedAction, GameState, PlayerState};
use crate::monte_carlo_equity::get_equivalent_hole_board;
use crate::HoleCards;

pub struct InfoStateAgent {
    pub hole_cards: Option<HoleCards>,
    pub name: String,

    info_state_db: Rc<RefCell<InfoStateDb>>,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
}

impl InfoStateAgent {
    pub fn new(
        name: &str,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
        info_state_db: Rc<RefCell<InfoStateDb>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            hole_cards: None,
            monte_carlo_db,
            info_state_db,
        }
    }
}

impl Agent for InfoStateAgent {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        let info_state = InfoState::from_game_state(
            game_state,
            player_state,
            self.hole_cards.as_ref().unwrap(),
            self.monte_carlo_db.clone(),
        );

        let info_state_db = self.info_state_db.borrow();

        let action_values = info_state_db.get(&info_state).unwrap();

        if action_values.is_none() {
            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some(format!("[{}]; did not exist, so folding", &info_state)),
            };
        }

        let action_values = action_values.unwrap();

        assert_eq!(action_values.len(), info_state_actions::NUM_ACTIONS);

        // action_values[info_state_actions::ALL_IN as usize] = InfoStateActionValueType::MIN;
        // action_values[info_state_actions::BET_POT as usize] = InfoStateActionValueType::MIN;

        // if game_state.current_to_call > 0 {
        //     action_values[info_state_actions::CHECK as usize] = InfoStateActionValueType::MIN;
        //     action_values[info_state_actions::BET_HALF as usize] = InfoStateActionValueType::MIN;
        // } else {
        //     action_values[info_state_actions::RAISE_3X as usize] = InfoStateActionValueType::MIN;
        //     action_values[info_state_actions::CALL as usize] = InfoStateActionValueType::MIN;
        // }

        let max_action_index = action_values
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0 as u8;

        let incoming_bet = game_state.current_to_call > 0;

        //let normalized = InfoStateDb::normalize_array(&action_values);
        //let common_comment = InfoStateDb::normalized_array_to_string(&normalized);

        let common_comment_is =
            InfoStateDb::normalized_array_to_string(&action_values, incoming_bet);

        let (eq_hole_cards, mut eq_board) =
            get_equivalent_hole_board(&self.hole_cards.unwrap(), game_state.board.as_slice_card());
        eq_board.get_index();

        let eq = self
            .monte_carlo_db
            .borrow_mut()
            .get_put(
                &eq_board,
                &eq_hole_cards,
                game_state.num_players_at_round_start(),
            )
            .unwrap();

        let common_comment = format!(
            "Eq {:.2}% with {} players in round;Non Folded Player Count: {} left to act: {};{}",
            eq * 100.0,
            game_state.num_players_at_round_start(),
            game_state.num_non_folded_players(),
            game_state.num_left_to_act,
            common_comment_is
        );

        let helpers = player_state.get_helpers(game_state);

        if incoming_bet {
            match max_action_index {
                info_state_actions::FOLD => {
                    //1 case, we can check big blind
                    if player_state.cur_round_putting_in_pot == game_state.current_to_call {
                        CommentedAction {
                            action: ActionEnum::Check,
                            comment: Some(format!(
                                "[{}]; checked big blind {}",
                                &info_state, &common_comment
                            )),
                        }
                    } else {
                        CommentedAction {
                            action: ActionEnum::Fold,
                            comment: Some(format!("[{}]; folded {}", &info_state, &common_comment)),
                        }
                    }
                }
                info_state_actions::CALL => CommentedAction {
                    action: ActionEnum::Call(helpers.call_amount),
                    comment: Some(format!("[{}]; called {}", &info_state, &common_comment)),
                },
                info_state_actions::RAISE_3X => helpers.build_raise_to(
                    game_state,
                    game_state.current_to_call * 3,
                    format!("[{}]; raised {}", &info_state, &common_comment),
                ),

                _ => {
                    panic!("Unknown action index {}", max_action_index);
                }
            }
        } else {
            match max_action_index {
                info_state_actions::CHECK => CommentedAction {
                    action: ActionEnum::Check,
                    comment: Some(format!("[{}]; checked {}", &info_state, &common_comment)),
                },
                info_state_actions::BET_HALF => helpers.build_bet(
                    game_state.pot() / 2,
                    format!("[{}]; bet {}", &info_state, &common_comment),
                ),
                info_state_actions::BET_POT => helpers.build_bet(
                    game_state.pot(),
                    format!("[{}]; bet {}", &info_state, &common_comment),
                ),
                _ => {
                    panic!("Unknown action index {}", max_action_index);
                }
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
