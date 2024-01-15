use std::{cell::RefCell, cmp::min, rc::Rc};

use boomphf::Mphf;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    likes_hands::likes_hand,
    monte_carlo_equity::get_equivalent_hole_board,
    pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash},
    ActionEnum, BoolRange, CommentedAction, GameState, HoleCards, PlayerState, Round,
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
    pub early_position_range: BoolRange,
    pub mid_position_range: BoolRange,
    pub late_position_range: BoolRange,
    pub button_range: BoolRange,
    pub three_bet_range: BoolRange,
}

impl EqAgentConfig {
    pub fn get_aggressive() -> Self {
        Self {
            flop_min_eq_to_bet: vec![0.5, 0.4, 0.4, 0.3],
            turn_min_eq_to_bet: vec![0.5, 0.55, 0.60],
            river_min_eq_to_bet: vec![0.55, 0.7, 0.8],
            early_position_range: "77+,A4s+,AJo+,K9s+,K5s,KQo,QTs+,JTs".parse().unwrap(),
            mid_position_range: "55+,A3s+,ATo+,K8s+,K6s-K5s,KJo+,Q9s+,J9s+,T9s,76s".parse().unwrap(),
            late_position_range: "33+,A2s+,A9o+,K4s+,KTo+,Q6s+,QTo+,J8s+,JTo,T8s+,97s+,87s".parse().unwrap(),
            button_range: "22+,A2+,K2s+,K7o+,Q2s+,Q8o+,J3s+,J8o+,T5s+,T8o+,96s+,98o,85s+,87o,75s+,76o,64s+,53s+".parse().unwrap(),
            three_bet_range: "JJ+,AQs+,AKo".parse().unwrap(),
        }
    }

    pub fn get_passive() -> Self {
        Self {
            flop_min_eq_to_bet: vec![0.7, 0.8],
            turn_min_eq_to_bet: vec![0.8, 0.9],
            river_min_eq_to_bet: vec![0.8, 0.9],
            early_position_range: "22+,A2+,K2s+,K7o+,Q2s+,Q8o+,J3s+,J8o+,T5s+,T8o+,96s+,98o,85s+,87o,75s+,76o,64s+,53s+".parse().unwrap(),
            mid_position_range: "22+,A2+,K2+,Q2+,J2+,T2s+,T6o+,92s+,95o+,82s+,84o+,72s+,74o+,62s+,65o,53s+".parse().unwrap(),
            late_position_range: "22+,A2+,K2+,Q2+,J2+,T2+,92+,82s+,84o+,72s+,74o+,62s+,64o+,52s+,54o,42s+,32s".parse().unwrap(),
            button_range: "22+,A2+,K2+,Q2+,J2+,T2+,92+,82s+,84o+,72s+,74o+,62s+,64o+,52s+,54o,42s+,32s".parse().unwrap(),
            three_bet_range: "AA".parse().unwrap()
        }
    }
}

//#[derive(Default)]
pub struct EqAgent {
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
        name: &str,
        agent_config: EqAgentConfig,
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    ) -> Self {
        

        EqAgent {
           // calling_range,
            hole_cards: None,
            name: name.to_string(),
            partial_rank_db,
            flop_texture_db,
            monte_carlo_db,
            hash_func: load_boomperfect_hash(),
            agent_config,
        }
    }

    fn decide_postflop(
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let non_folded_players = game_state.num_non_folded_players();
        let players_at_round_start = game_state.num_players_at_round_start();

        let hole_cards = self.hole_cards.as_ref().unwrap();
        let mut pr_db = self.partial_rank_db.borrow_mut();
        let prc = pr_db.get_put(&game_state.board, hole_cards, 0).unwrap();
        let mut ft_db = self.flop_texture_db.borrow_mut();
        let ft = ft_db.get_put(&game_state.board).unwrap();

        let rank = fast_hand_eval(
            game_state.board.get_iter().chain(hole_cards.iter()),
            &self.hash_func,
        );

        let likes_hand_response = likes_hand(
            &prc,
            &ft,
            &rank,
            &game_state.board,
            &hole_cards,
            players_at_round_start,
        )
        .unwrap();

        let (eq_hole_cards, mut eq_board) =
            get_equivalent_hole_board(&hole_cards, game_state.board.as_slice_card());
        eq_board.get_index();

        //Issue is we have Raise, fold, fold, Us to act
        //If we calculate eq with vs 2, it will be quite high

        let eq = self
            .monte_carlo_db
            .borrow_mut()
            .get_put(&eq_board, &eq_hole_cards, players_at_round_start)
            .unwrap();

        let call_amt = min(
            game_state.current_to_call - player_state.cur_round_putting_in_pot.unwrap_or(0),
            player_state.stack,
        );

        //max is always just the remaining stack

        let mut comment_common = format!(
            "Eq {:.2}% with {} players in round;Non Folded Player Count: {};Likes Hand Level: {};Positive {};Negative {}",
            eq * 100.0,
            players_at_round_start,
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
        let mut bet_threshold = eq + 1.1;

        //1st index is 2 players
        let threshold_index = (non_folded_players - 2) as usize;
        if game_state.current_round == Round::Flop
            && !self.agent_config.flop_min_eq_to_bet.is_empty()
        {
            if threshold_index >= self.agent_config.flop_min_eq_to_bet.len() {
                bet_threshold = self.agent_config.flop_min_eq_to_bet
                    [self.agent_config.flop_min_eq_to_bet.len() - 1];
            } else {
                bet_threshold = self.agent_config.flop_min_eq_to_bet[threshold_index];
            }
        } else if game_state.current_round == Round::Turn
            && !self.agent_config.turn_min_eq_to_bet.is_empty()
        {
            if threshold_index >= self.agent_config.turn_min_eq_to_bet.len() {
                bet_threshold = self.agent_config.turn_min_eq_to_bet
                    [self.agent_config.turn_min_eq_to_bet.len() - 1];
            } else {
                bet_threshold = self.agent_config.turn_min_eq_to_bet[threshold_index];
            }
        } else if game_state.current_round == Round::River
            && !self.agent_config.river_min_eq_to_bet.is_empty()
        {
            if threshold_index >= self.agent_config.river_min_eq_to_bet.len() {
                bet_threshold = self.agent_config.river_min_eq_to_bet
                    [self.agent_config.river_min_eq_to_bet.len() - 1];
            } else {
                bet_threshold = self.agent_config.river_min_eq_to_bet[threshold_index];
            }
        }

        let half_pot_bet = min(game_state.pot() / 2, player_state.stack);

        if eq > bet_threshold {
            return CommentedAction {
                action: ActionEnum::Bet(half_pot_bet),
                comment: Some(format!(
                    "Eq is at least {:.2}%;{}",
                    bet_threshold * 100.0,
                    comment_common
                )),
            };
        } else {
            return CommentedAction {
                action: ActionEnum::Check,
                comment: Some(format!(
                    "Eq is less than {:.2}%;{}",
                    bet_threshold * 100.0,
                    comment_common
                )),
            };
        }
    }

    fn decide_preflop(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let ri = self.hole_cards.unwrap().to_range_index();

        //Anyone bet so far?
        let any_raises = game_state.current_to_call > game_state.bb;

        let num_players = game_state.player_states.len() as u8;
        let position_family = player_state.position.get_position_family(num_players);

        let range_to_use = match position_family {
            crate::PositionFamily::UTG => &self.agent_config.early_position_range,
            crate::PositionFamily::Middle => &self.agent_config.mid_position_range,
            crate::PositionFamily::Late => &self.agent_config.late_position_range,
            crate::PositionFamily::Button => &self.agent_config.button_range,
            crate::PositionFamily::Blinds => &self.agent_config.button_range,
        };

        let helpers = player_state.get_helpers(game_state);

        let common_comment = format!(
            "Position {} Family {};Range {:.1}%",
            player_state.position,
            position_family,
            range_to_use.get_perc_enabled() * 100.0
        );

        if !any_raises {
            if range_to_use.data[ri] {
                helpers.build_raise_to(
                    game_state,
                    game_state.current_to_call * 3,
                    format!("Opening raise;{}", common_comment),
                )
            } else {
                if game_state.current_to_call == 0 {
                    CommentedAction {
                        action: ActionEnum::Check,
                        comment: Some("Checking the big blind".to_string()),
                    }
                } else {
                    CommentedAction {
                        action: ActionEnum::Fold,
                        comment: Some(format!("Not in opening range;{}", common_comment)),
                    }
                }
            }
        } else {
            let bb_amt = game_state.current_to_call as f64 / game_state.bb as f64;

            if bb_amt >= 10.0 {
                if self.agent_config.three_bet_range.data[ri] {
                    CommentedAction {
                        action: ActionEnum::Call(helpers.call_amount),
                        comment: Some(format!(
                            "Calling >3-bet;3bet Range: {:.1}%;{}",
                            self.agent_config.three_bet_range.get_perc_enabled() * 100.0,
                            common_comment
                        )),
                    }
                } else {
                    CommentedAction {
                        action: ActionEnum::Fold,
                        comment: Some(format!(
                            "Not in >3-bet range;3bet range: {:.1}%;{}",
                            self.agent_config.three_bet_range.get_perc_enabled() * 100.0,
                            common_comment
                        )),
                    }
                }
            } else if bb_amt >= 4.5 {
                if range_to_use.data[ri] {
                    CommentedAction {
                        action: ActionEnum::Call(helpers.call_amount),
                        comment: Some(format!(
                            "Calling a 3 bet in opening range;{}",
                            common_comment
                        )),
                    }
                } else {
                    CommentedAction {
                        action: ActionEnum::Fold,
                        comment: Some(format!(
                            "Not in opening range, folding to 3-bet;{}",
                            common_comment
                        )),
                    }
                }
            } else {
                if self.agent_config.three_bet_range.data[ri] {
                    helpers.build_raise_to(
                        game_state,
                        game_state.current_to_call * 3,
                        format!(
                            "3-betting with range: {:.1}%;{}",
                            &self.agent_config.three_bet_range.get_perc_enabled() * 100.0,
                            common_comment
                        ),
                    )
                } else if range_to_use.data[ri] {
                    CommentedAction {
                        action: ActionEnum::Call(helpers.call_amount),
                        comment: Some(format!("Calling a pre flop raise;{}", common_comment)),
                    }
                } else {
                    CommentedAction {
                        action: ActionEnum::Fold,
                        comment: Some(format!(
                            "Not in opening range, folding to PFR;{}",
                            common_comment
                        )),
                    }
                }
            }
        }
    }
}

impl Agent for EqAgent {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        match game_state.current_round {
            Round::Preflop => {
                return self.decide_preflop(player_state, game_state);
            }
            _ => {
                return self.decide_postflop(player_state, game_state);
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
