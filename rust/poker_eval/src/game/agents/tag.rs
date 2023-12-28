use std::{cell::RefCell, rc::Rc};

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    ActionEnum, CommentedAction, FlushDrawType, GameState, HoleCards,
    PartialRankContainer, PlayerState, Round, StraightDrawType,
};

use postflop_solver::Range;

use super::Agent;

//#[derive(Default)]
pub struct Tag {
    pub three_bet_range: Range,
    pub pfr_range: Range,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db:
        Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>>,
}

impl Tag {
    pub fn new(
        three_bet_range_str: &str,
        pfr_range_str: &str,
        name: &str,
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<
            RefCell<EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>,
        >,
    ) -> Self {
        Tag {
            three_bet_range: three_bet_range_str.parse().unwrap(),
            pfr_range: pfr_range_str.parse().unwrap(),
            hole_cards: None,
            name: name.to_string(),
            flop_texture_db,
            partial_rank_db,
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
            if self.pfr_range.data[ri] > 0.0 {
                CommentedAction {
                    action: ActionEnum::Raise(game_state.bb * 3),
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
            if self.pfr_range.data[ri] > 0.0 {
                CommentedAction {
                    action: ActionEnum::Raise(game_state.current_to_call * 3),
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
        _player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction {
        let non_folded_players = game_state
            .player_states
            .iter()
            .filter(|ps| !ps.folded)
            .count();

        let hc = self.hole_cards.as_ref().unwrap();
        let mut pdb = self.partial_rank_db.borrow_mut();
        let prc = pdb.get_put(&game_state.board, hc).unwrap();

        let mut ft_db = self.flop_texture_db.borrow_mut();
        let ft = ft_db.get_put(&game_state.board).unwrap();

        let mut likes_hand_comments: Vec<String> = Vec::new();

        if let Some(p) = prc.lo_pair {
            if p.made_quads {
                likes_hand_comments.push(format!("lo pair Quads {}", hc.get_hi_card().value));
            }
            if p.made_set {
                likes_hand_comments.push(format!("lo pair Set {}", hc.get_hi_card().value));
            }
        }
        if let Some(p) = prc.hi_pair {
            if p.number_above == 0 {
                likes_hand_comments.push(format!("Top pair {}", hc.get_hi_card().value));
            }
            if p.made_quads {
                likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
            }
            if p.made_set {
                likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
            }
        }
        if let Some(p) = prc.pocket_pair {
            if p.number_above == 0 {
                likes_hand_comments.push(format!("Overpair {}", hc.get_hi_card().value));
            }
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

        let current_pot = game_state.pot();

        let half_pot = current_pot / 2;
        let third_pot = current_pot / 3;

        if game_state.current_to_call == 0 {
            if non_folded_players >= 4 && ft.num_with_str8 > 150 {
                return CommentedAction {
                    action: ActionEnum::Check,
                    comment: Some(format!(
                        "Worried someone ({} players) has a straight, {} / {} not betting",
                        non_folded_players, ft.num_with_str8, ft.num_hole_cards
                    )),
                };
            }

            if likes_hand_comments.len() > 0 {
                return CommentedAction {
                    action: ActionEnum::Bet(third_pot),
                    comment: Some(format!(
                        "Bets 1/3 pot because likes hand: {}",
                        likes_hand_comments.join(", ")
                    )),
                };
            } else {
                return CommentedAction {
                    action: ActionEnum::Check,
                    comment: Some("Checking because does not like hand".to_string()),
                };
            }
        } else {
            let pot_eq = (100. * game_state.current_to_call as f64)
                / ((current_pot + game_state.pot()) as f64);

            if likes_hand_comments.is_empty() {
                return CommentedAction {
                    action: ActionEnum::Fold,
                    comment: Some(
                        "Folding because does not like hand and facing bet/raise".to_string(),
                    ),
                };
            }
            if game_state.current_to_call < third_pot {
                return CommentedAction {
                    action: ActionEnum::Raise(third_pot),
                    comment: Some(format!(
                        "Raising because wants 1/3 pot bet and likes hand: {}",
                        likes_hand_comments.join(", ")
                    )),
                };
            }
            if game_state.current_to_call < half_pot {
                return CommentedAction {
                    action: ActionEnum::Call,
                    comment: Some(format!(
                        "Calling because likes hand and willing to call a 1/2 pot bet: {}",
                        likes_hand_comments.join(", ")
                    )),
                };
            }
            return CommentedAction {
                action: ActionEnum::Fold,
                comment: Some(format!(
                    "Folding because likes hand but bet is too high. {:.2}",
                    pot_eq
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

    #[test]
    fn test_doesnt_bet_river() {
        init_test_logger();

        let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards, _> =
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
            final_eval_comment: None,
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
                final_eval_comment: None,
            });
        }

        let game_state: GameState = GameState {
            player_states: other_players,
            current_to_act: player_state.position,
            current_round: Round::River,
            prev_round_pot: 400,
            round_pot: 0,
            current_to_call: 0,
            min_raise: 5,
            board: Board::try_from("2s 3c 8h 5d 6c").unwrap(),
            sb: 2,
            bb: 5,
            actions: vec![],
        };

        tag.set_hole_cards("8d 7d".parse().unwrap());

        let action = tag.decide(&player_state, &game_state);

        info!("Action: {}", action);
        assert_eq!(action.action, ActionEnum::Check);

        drop(rcref_pdb.borrow_mut());
    }
}
