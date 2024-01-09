use std::{cell::RefCell, cmp::min, rc::Rc};

use bitvec::vec;
use boomphf::Mphf;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    likes_hands::likes_hand,
    pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash},
    ActionEnum, BoolRange, CommentedAction, GameState, HoleCards, PlayerState, Round, monte_carlo_equity::get_equivalent_hole_board,
};

use super::Agent;

//Need some config struct, for like eq to raise, call
//For now just constants
const EQ_TO_ALL_IN: f64 = 0.75;
use num_format::{Locale, ToFormattedString};

pub struct NumPlayers {
    pub from_num_players: usize,
    pub to_num_players: usize,
}
pub struct EqAgentConfig {
    //round, num players, min eq to bet
    //num players starts at 2
    pub flop_min_eq_to_bet: Vec<f64>,
    pub turn_min_eq_to_bet: Vec<f64>,
    pub river_min_eq_to_bet: Vec<f64>,
}

impl EqAgentConfig {
    pub fn get_aggressive(&self) -> Self {
        Self {
            flop_min_eq_to_bet: vec![0.5, 0.4, 0.4, 0.3],
            turn_min_eq_to_bet: vec![0.5, 0.55, 0.60],
            river_min_eq_to_bet: vec![0.55, 0.7, 0.8]
        }
    }

    pub fn get_passive(&self) -> Self {
        Self {
            flop_min_eq_to_bet: vec![0.7, 0.8],
            turn_min_eq_to_bet: vec![0.8, 0.9],
            river_min_eq_to_bet: vec![0.8, 0.9]
        }
    }
}

//#[derive(Default)]
pub struct EqAgent {
    pub calling_range: Option<BoolRange>,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    hash_func: Mphf<u32>,
    agent_config: EqAgentConfig,
}

impl EqAgent {
    pub fn new(
        calling_range_str: Option<&str>,
        name: &str,
        agent_config: EqAgentConfig,
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    ) -> Self {
        let calling_range = match calling_range_str {
            Some(s) => Some(s.parse().unwrap()),
            None => None,
        };

        EqAgent {
            calling_range,
            hole_cards: None,
            name: name.to_string(),
            partial_rank_db,
            flop_texture_db,
            monte_carlo_db,
            hash_func: load_boomperfect_hash(),
            agent_config
        }
    }

    fn decide_postflop(
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let non_folded_players = game_state
            .player_states
            .iter()
            .filter(|ps| !ps.folded)
            .count() as u8;

        let hole_cards = self.hole_cards.as_ref().unwrap();
        let mut pr_db = self.partial_rank_db.borrow_mut();
        let prc = pr_db.get_put(&game_state.board, hole_cards, 0).unwrap();
        let mut ft_db = self.flop_texture_db.borrow_mut();
        let ft = ft_db.get_put(&game_state.board).unwrap();

        let rank = fast_hand_eval(
            game_state.board.get_iter().chain(hole_cards.get_iter()),
            &self.hash_func,
        );

        let likes_hand_response =
            likes_hand(&prc, &ft, &rank, &game_state.board, &hole_cards, non_folded_players).unwrap();

        let (eq_hole_cards, mut eq_board) = get_equivalent_hole_board(&hole_cards, &game_state.board);
        eq_board.get_index();

        let eq = self
            .monte_carlo_db
            .borrow_mut()
            .get_put(&eq_board, &eq_hole_cards, non_folded_players)
            .unwrap();

        let call_amt = min(
            game_state.current_to_call - player_state.cur_round_putting_in_pot.unwrap_or(0),
            player_state.stack,
        );

        //max is always just the remaining stack

        let mut comment_common = format!(
            "Eq {:.2}% with {} other players;Likes Hand Level {};Positive {};Negative {}",
            eq * 100.0,
            non_folded_players,
            likes_hand_response.likes_hand,
            likes_hand_response.likes_hand_comments.join(", "),
            likes_hand_response.not_like_hand_comments.join(", ")
        );

        //are we facing a bet?
        if call_amt > 0 {
            let pot_eq = call_amt as f64 / (call_amt as f64 + game_state.pot() as f64);
            comment_common.push_str(&format!(
                ";Pot Eq {:.2}% calling {} into {} pot",
                pot_eq * 100.0,
                call_amt,
                game_state.pot().to_formatted_string(&Locale::en)
            ));

            if eq >= EQ_TO_ALL_IN && call_amt < player_state.stack {
                let max_can_raise =
                    player_state.stack + player_state.cur_round_putting_in_pot.unwrap_or(0);
                //let min_can_raise = min(game_state.min_raise + game_state.current_to_call, max_can_raise);

                return CommentedAction {
                    action: ActionEnum::Raise(
                        max_can_raise - game_state.current_to_call,
                        max_can_raise,
                    ),
                    comment: Some(format!(
                        "Raising all in, equity at least {:.2}%;{}",
                        EQ_TO_ALL_IN * 100.0,
                        comment_common
                    )),
                };
            } else if eq >= pot_eq {
                return CommentedAction {
                    action: ActionEnum::Call(call_amt),
                    comment: Some(format!("Enough to call;{}", comment_common)),
                };
            } else {
                return CommentedAction {
                    action: ActionEnum::Fold,
                    comment: Some(format!("Not enough eq to call;{}", comment_common)),
                };
            }
        }

        //here not facing a bet

        let half_pot_bet = min(game_state.pot() / 2, player_state.stack);

        if eq > 0.5 {
            return CommentedAction {
                action: ActionEnum::Bet(half_pot_bet),
                comment: Some(format!(
                    "Eq is at least {:.2}%;{}",
                    0.5 * 100.0,
                    comment_common
                )),
            };
        } else {
            return CommentedAction {
                action: ActionEnum::Check,
                comment: Some(format!(
                    "Eq is less than {:.2}%;{}",
                    0.5 * 100.0,
                    comment_common
                )),
            };
        }
    }
}

impl Agent for EqAgent {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        let action = match game_state.current_round {
            Round::Preflop => {
                let call_amt =
                    game_state.current_to_call - player_state.cur_round_putting_in_pot.unwrap_or(0);
                let ri = self.hole_cards.unwrap().to_range_index();
                //not handling all ins
                if let Some(calling_range) = self.calling_range.as_ref() {
                    if calling_range.data[ri] {
                        ActionEnum::Call(call_amt)
                    } else {
                        ActionEnum::Fold
                    }
                } else {
                    ActionEnum::Call(call_amt)
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
