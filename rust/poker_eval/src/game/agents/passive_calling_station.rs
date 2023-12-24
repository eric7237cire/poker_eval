use crate::{ActionEnum, AgentState, OldGameState, Round, PlayerState, GameState, HoleCards};
use postflop_solver::Range;

use super::Agent;

//#[derive(Copy, Clone)]
pub struct PassiveCallingStation {
    pub calling_range: Range,
    pub hole_cards: HoleCards,
}

impl Agent for PassiveCallingStation {
    fn decide(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> ActionEnum {
        match game_state.current_round {
            Round::Preflop => {
                //poker_eval as a different idea of card

                let ri = self.hole_cards.to_range_index();
                //not handling all ins
                if self.calling_range.data[ri] > 0.0 {
                    ActionEnum::Call
                } else {
                    ActionEnum::Fold
                }
            }
            Round::Flop => ActionEnum::Call,
            Round::Turn => ActionEnum::Call,
            Round::River => ActionEnum::Call,
        }
    }

    fn get_hole_cards(&self) -> HoleCards {
        self.hole_cards
    }
    
}
