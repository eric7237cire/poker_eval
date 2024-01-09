use std::{
    cell::RefCell,
    cmp::{min},
    rc::Rc,
};

use boomphf::Mphf;


use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    likes_hands::{likes_hand, LikesHandLevel, LikesHandResponse},
    pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash},
    ActionEnum, BoolRange, CommentedAction, GameState, HoleCards, PlayerState, Round,
};

use super::Agent;

//#[derive(Default)]
pub struct Tag {
    pub three_bet_range: BoolRange,
    pub pfr_range: BoolRange,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    hash_func: Mphf<u32>,
}

impl Tag {
    pub fn new(
        three_bet_range_str: &str,
        pfr_range_str: &str,
        name: &str,
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    ) -> Self {
        Tag {
            three_bet_range: three_bet_range_str.parse().unwrap(),
            pfr_range: pfr_range_str.parse().unwrap(),
            hole_cards: None,
            name: name.to_string(),
            flop_texture_db,
            partial_rank_db,
            hash_func: load_boomperfect_hash(),
        }
    }

    fn decide_preflop(
        &self,
        _player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let ri = self.hole_cards.unwrap().to_range_index();

        //Anyone bet so far?
        let any_raises = game_state.current_to_call > game_state.bb;

        if !any_raises {
            if self.pfr_range.data[ri] {
                CommentedAction {
                    action: ActionEnum::Raise(
                        game_state.bb * 3 - game_state.current_to_call,
                        game_state.bb * 3,
                    ),
                    comment: Some("Opening raise".to_string()),
                }
            } else {
                if game_state.current_to_call == 0 {
                    CommentedAction {
                        action: ActionEnum::Check,
                        comment: Some("Checking the big blind".to_string()),
                    }
                } else {
                    CommentedAction {
                        action: ActionEnum::Fold,
                        comment: Some("Not in opening range".to_string()),
                    }
                }
            }
        } else {
            if self.pfr_range.data[ri] {
                CommentedAction {
                    action: ActionEnum::Raise(
                        game_state.current_to_call * 3 - game_state.current_to_call,
                        game_state.current_to_call * 3,
                    ),
                    comment: Some("3-betting".to_string()),
                }
            } else {
                CommentedAction {
                    action: ActionEnum::Fold,
                    comment: Some("Not in 3-bet range, folding to pfr".to_string()),
                }
            }
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

        let hc = self.hole_cards.as_ref().unwrap();
        let mut pdb = self.partial_rank_db.borrow_mut();
        let prc = pdb.get_put(&game_state.board, hc, 0).unwrap();

        let mut ft_db = self.flop_texture_db.borrow_mut();
        let ft = ft_db.get_put(&game_state.board).unwrap();

        let rank = fast_hand_eval(
            game_state.board.get_iter().chain(hc.get_iter()),
            &self.hash_func,
        );

        let likes_hand_response =
            likes_hand(&prc, &ft, &rank, &game_state.board, &hc, non_folded_players).unwrap();

        let current_pot = game_state.pot();

        //How much extra we need to put in to call the current bet, can be less than the total call required
        //if we are calling a raise to our bet
        let call_amt = min(
            game_state.current_to_call - player_state.cur_round_putting_in_pot.unwrap_or(0),
            player_state.stack,
        );

        //we can raise to a stack more that what we've already put in
        //Both these values are the total amount, not the increase
        let max_can_raise = player_state.stack + player_state.cur_round_putting_in_pot.unwrap_or(0);
        let _min_can_raise = min(
            game_state.min_raise + game_state.current_to_call,
            max_can_raise,
        );

        //let third_pot = max(min_can_raise, min(max_can_raise, current_pot / 3));

        let can_raise =
            max_can_raise > call_amt + player_state.cur_round_putting_in_pot.unwrap_or(0);

        if game_state.current_to_call == 0 {
            return self.decide_should_bet(&likes_hand_response, player_state, game_state);
        } else {
            let pot_eq = (100. * game_state.current_to_call as f64)
                / ((current_pot + game_state.pot()) as f64);

            if likes_hand_response.likes_hand <= LikesHandLevel::None {
                return CommentedAction {
                    action: ActionEnum::Fold,
                    comment: Some(
                        "Folding because does not like hand and facing bet/raise".to_string(),
                    ),
                };
            }
            if likes_hand_response.likes_hand >= LikesHandLevel::AllIn && can_raise {
                return CommentedAction {
                    action: ActionEnum::Raise(
                        max_can_raise - game_state.current_to_call,
                        max_can_raise,
                    ),
                    comment: Some(format!(
                        "Going all in because likes hand @ {};Positive {};Negative: {}",
                        likes_hand_response.likes_hand,
                        likes_hand_response.likes_hand_comments.join(", "),
                        likes_hand_response.not_like_hand_comments.join(", ")
                    )),
                };
            }
            // } else if game_state.current_to_call < game_state.pot() / 3
            //     && likes_hand_response.likes_hand >= LikesHandLevel::LargeBet
            //     && can_raise
            // {
            //     // debug!("hey1 min_raise: {} stack: {} already put in {} cur bet {}", game_state.min_raise,
            //     // player_state.stack, player_state.cur_round_putting_in_pot.unwrap_or(0), game_state.current_to_call);

            //     return CommentedAction {
            //         action: ActionEnum::Raise(third_pot - game_state.current_to_call, third_pot),
            //         comment: Some(format!(
            //             "Raising because wants 1/3 pot bet and likes hand: {}",
            //             likes_hand_response.likes_hand_comments.join(", ")
            //         )),
            //     };
            else if call_amt <= game_state.pot() / 2
                && likes_hand_response.likes_hand >= LikesHandLevel::LargeBet
            {
                return CommentedAction {
                    action: ActionEnum::Call(call_amt),
                    comment: Some(format!(
                        "Calling because likes hand: {};Willing to call a 1/2 pot bet;Positive: {};Negative: {}",
                        likes_hand_response.likes_hand,
                        likes_hand_response.likes_hand_comments.join(", "),
                        likes_hand_response.not_like_hand_comments.join(", ")
                    )),
                };
            } else if likes_hand_response.likes_hand >= LikesHandLevel::AllIn {
                return CommentedAction {
                    action: ActionEnum::Call(call_amt),
                    comment: Some(format!(
                        "Calling because can't raise any more;likes hand: {};Positive: {};Negative: {}",
                        likes_hand_response.likes_hand,
                        likes_hand_response.likes_hand_comments.join(", "),
                        likes_hand_response.not_like_hand_comments.join(", ")
                    )),
                };
            }

            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some(format!(
                    "Folding because likes hand: {};Pot Equity: {:.2};Positive: {};Negative: {}",
                    likes_hand_response.likes_hand,
                    pot_eq,
                    likes_hand_response.likes_hand_comments.join(", "),
                    likes_hand_response.not_like_hand_comments.join(", ")
                )),
            };
        }
    }

    fn decide_should_bet(
        &self,
        likes_hand_response: &LikesHandResponse,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let max_can_raise = player_state.stack + player_state.cur_round_putting_in_pot.unwrap_or(0);
        //let min_can_raise = min(game_state.min_raise, player_state.stack);

        let current_pot = game_state.pot();
        //let half_pot = min(max_can_raise, current_pot / 2);
        let third_pot = min(max_can_raise, current_pot / 3);

        if likes_hand_response.likes_hand >= LikesHandLevel::SmallBet
            && game_state.board.get_round().unwrap() < Round::River
        {
            return CommentedAction {
                action: ActionEnum::Bet(third_pot),
                comment: Some(format!(
                    "Bets 1/3 pot because likes hand @ {}: +1 {}; -1 {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),
                    likes_hand_response.not_like_hand_comments.join(", ")
                )),
            };
        } else if likes_hand_response.likes_hand >= LikesHandLevel::LargeBet {
            return CommentedAction {
                action: ActionEnum::Bet(third_pot),
                comment: Some(format!(
                    "Bets 1/3 pot on river because likes hand @ {}: +1 {}; -1 {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),
                    likes_hand_response.not_like_hand_comments.join(", ")
                )),
            };
        } else {
            return CommentedAction {
                action: ActionEnum::Check,
                comment: Some(format!(
                    "Checking because does not like hand enough @ {} comments -- {}; {}",
                    likes_hand_response.likes_hand,
                    likes_hand_response.likes_hand_comments.join(", "),
                    likes_hand_response.not_like_hand_comments.join(", ")
                )),
            };
        }
    }
}

impl Agent for Tag {
    fn decide(&mut self, player_state: &PlayerState, game_state: &GameState) -> CommentedAction {
        match game_state.current_round {
            Round::Preflop => {
                return self.decide_preflop(player_state, game_state);
            }
            _ => {
                return self.decide_postflop(player_state, game_state);
            }
        };
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use log::info;

    use super::*;
    use crate::{board_hc_eval_cache_redb::EvalCacheWithHcReDb, init_test_logger, Board};

    //#[test]
    //Need to move over partial rank to a no file system solution
    #[allow(dead_code)]
    fn test_doesnt_bet_river() {
        init_test_logger();

        let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
            EvalCacheWithHcReDb::new().unwrap();

        let rcref_pdb = Rc::new(RefCell::new(partial_rank_db));

        let flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

        let rcref_ftdb = Rc::new(RefCell::new(flop_texture_db));

        let mut tag = Tag::new(
            "JJ+,AJs+,AQo+,KQs",
            "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
            "Hero",
            rcref_ftdb.clone(),
            rcref_pdb.clone(),
        );

        let player_state = PlayerState {
            stack: 410,
            folded: false,
            position: 4.try_into().unwrap(),
            player_name: "Hero".to_string(),
            initial_stack: 500,
            cur_round_putting_in_pot: None,
            all_in: false,
            total_put_in_pot: 0,
        };

        let mut other_players: Vec<PlayerState> = Vec::with_capacity(5);

        for pos in 0..4 {
            other_players.push(PlayerState {
                stack: 500,
                folded: false,
                position: pos.try_into().unwrap(),
                player_name: "Villian".to_string(),
                initial_stack: 500,
                cur_round_putting_in_pot: None,
                all_in: false,
                total_put_in_pot: 0,
            });
        }

        let mut board = Board::try_from("2s 3c 8h 5d 6c").unwrap();
        board.get_index();

        let game_state: GameState = GameState {
            player_states: other_players,
            current_to_act: player_state.position,
            current_round: Round::River,
            prev_round_pot: 400,
            round_pot: 0,
            current_to_call: 0,
            min_raise: 5,
            board,
            sb: 2,
            bb: 5,
            actions: vec![],
            num_left_to_act: 2,
            total_active_players: 5,
            total_players_all_in: 0,
        };

        tag.set_hole_cards("8d 7d".parse().unwrap());

        let action = tag.decide(&player_state, &game_state);

        info!("Action: {}", action);
        assert_eq!(action.action, ActionEnum::Check);
    }
}
