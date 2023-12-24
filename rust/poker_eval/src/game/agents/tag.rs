use crate::{ActionEnum, GameState, HoleCards, PlayerState, Round};
use postflop_solver::Range;

use super::{Agent, AgentDecision};

#[derive(Default)]
pub struct Tag {
    pub three_bet_range: Option<Range>,
    pub pfr_range: Option<Range>,
    pub hole_cards: Option<HoleCards>,
    pub name: String,
}

impl Agent for Tag {
    fn decide(&self, _player_state: &PlayerState, game_state: &GameState) -> AgentDecision {
        let action = match game_state.current_round {
            Round::Preflop => {
                let ri = self.hole_cards.unwrap().to_range_index();

                //Anyone bet so far?
                let _any_raises = game_state.current_to_call > game_state.bb;

                if let Some(calling_range) = self.pfr_range {
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
