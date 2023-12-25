use crate::{ActionEnum, GameState, HoleCards, PlayerState, Round, partial_rank_cards, StraightDrawType, FlushDrawType};
use bitvec::vec;
use postflop_solver::Range;

use super::{Agent, AgentDecision};

#[derive(Default)]
pub struct Tag {
    pub three_bet_range: Range,
    pub pfr_range: Range,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
}

impl Tag {
    fn decide_preflop(&self, player_state: &PlayerState, game_state: &GameState) -> AgentDecision {
        let ri = self.hole_cards.unwrap().to_range_index();

        //Anyone bet so far?
        let any_raises = game_state.current_to_call > game_state.bb;

        if !any_raises {
            if self.pfr_range.data[ri] > 0.0 {
                AgentDecision {
                    action: ActionEnum::Raise(game_state.bb * 3),
                    comment: Some("Opening raise".to_string())
                }
            } else {
                if game_state.current_to_call == 0 {
                    AgentDecision {
                        action: ActionEnum::Check,
                        comment: Some("Checking the big blind".to_string())
                    }
                } else {
                    AgentDecision {
                        action: ActionEnum::Fold,
                        comment: Some("Not in opening range".to_string())
                    }
                }
            }
        } else {
            if self.pfr_range.data[ri] > 0.0 {
                

                AgentDecision {
                    action: ActionEnum::Raise(game_state.current_to_call * 3),
                    comment: Some("3-betting".to_string())
                }
            } else {                     
                AgentDecision {
                    action: ActionEnum::Fold,
                    comment: Some("Not in 3-bet range, folding to pfr".to_string())
                }
            }
        }
    }

    fn decide_postflop(&self, player_state: &PlayerState, game_state: &GameState) -> AgentDecision {

        let hc = self.hole_cards.as_ref().unwrap();
        let prc = partial_rank_cards(hc, &game_state.board);

        let mut likes_hand_comments: Vec<String> = Vec::new();
        
        if let Some(p) = prc.hi_pair {
            if p.number_above == 0 {
                likes_hand_comments.push( format!("Top pair {}", hc.get_hi_card().value) );
            }
            if p.made_quads {
                likes_hand_comments.push( format!("Quads {}", hc.get_hi_card().value) );
            }
            if p.made_set {
                likes_hand_comments.push( format!("Set {}", hc.get_hi_card().value) );
            }
        }
        if let Some(p) = prc.pocket_pair {
            if p.number_above == 0 {
                likes_hand_comments.push( format!("Overpair {}", hc.get_hi_card().value) );
            }
            if p.made_set {
                likes_hand_comments.push( format!("Pocket Pair Set {}", hc.get_hi_card().value) );
            }
            if p.made_quads {
                likes_hand_comments.push( format!("Pocket Pair Quads {}", hc.get_hi_card().value) );
            }
        }
        if game_state.current_round != Round::River {
            if let Some(p) = prc.flush_draw {
                if p.flush_draw_type == FlushDrawType::FlushDraw {
                    likes_hand_comments.push( format!("Flush draw {}", p.hole_card_value) );
                }                
            }
            if let Some(p) = prc.straight_draw {
                if p.straight_draw_type == StraightDrawType::OpenEnded || p.straight_draw_type == StraightDrawType::DoubleGutShot {
                    likes_hand_comments.push( format!("Straight draw") );
                }
                //likes_hand_comments.push( format!("Gutshot straight draw {}", p.) );
            }
        }

        let current_pot = game_state.pot();

        let half_pot = current_pot / 2;
        let third_pot = current_pot / 3;

        if game_state.current_to_call == 0 {
            if likes_hand_comments.len() > 0 {
                return AgentDecision {
                    action: ActionEnum::Bet(third_pot),
                    comment: Some(format!("Bets 1/3 pot because likes hand: {}", likes_hand_comments.join(", ")))
                }
            } else {
                return AgentDecision {
                    action: ActionEnum::Check,
                    comment: Some("Checking because does not like hand".to_string())
                }
            }
        } else {

            let pot_eq = (100. * game_state.current_to_call as f64) / ( (current_pot + game_state.pot()) as f64);

            if likes_hand_comments.is_empty() {
                return AgentDecision {
                    action: ActionEnum::Fold,
                    comment: Some("Folding because does not like hand and facing bet/raise".to_string())
                }
            }
            if game_state.current_to_call < third_pot {
                return AgentDecision {
                    action: ActionEnum::Raise(third_pot),
                    comment: Some(format!("Raising because wants 1/3 pot bet and likes hand: {}", likes_hand_comments.join(", ")))
                }
            }
            if game_state.current_to_call < half_pot {
                return AgentDecision {
                    action: ActionEnum::Call,
                    comment: Some(format!("Calling because likes hand and willing to call a 1/2 pot bet: {}", likes_hand_comments.join(", ")))
                }
            }
            return AgentDecision {
                action: ActionEnum::Fold,
                comment: Some(format!("Folding because likes hand but bet is too high. {:.2}", pot_eq))
            }
        }
    }

}




impl Agent for Tag {
    
    fn decide(&self, player_state: &PlayerState, game_state: &GameState) -> AgentDecision {
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
