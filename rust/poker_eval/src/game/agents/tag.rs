use crate::{ActionEnum, GameState, HoleCards, PlayerState, Round};
use postflop_solver::Range;

use super::{Agent, AgentDecision};

#[derive(Default)]
pub struct Tag {
    pub three_bet_range: Range,
    pub pfr_range: Range,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
}

impl Agent for Tag {
    fn decide(&self, _player_state: &PlayerState, game_state: &GameState) -> AgentDecision {
        let action = match game_state.current_round {
            Round::Preflop => {
                let ri = self.hole_cards.unwrap().to_range_index();

                //Anyone bet so far?
                let any_raises = game_state.current_to_call > game_state.bb;

                if !any_raises {
                    if self.pfr_range.data[ri] > 0.0 {
                        ActionEnum::Raise(game_state.bb * 3)
                    } else {
                        if game_state.current_to_call == 0 {
                            ActionEnum::Check
                        } else {
                            ActionEnum::Fold
                        }
                    }
                } else {
                    if self.pfr_range.data[ri] > 0.0 {
                        ActionEnum::Raise(game_state.current_to_call * 3)
                    } else {                     
                        ActionEnum::Fold
                    }
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

        AgentDecision {
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
