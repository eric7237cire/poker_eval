use postflop_solver::Range;
use crate::{Agent, Round, GameState, Action, AgentRoundInfo, AgentState};

//#[derive(Copy, Clone)]
pub struct PassiveCallingStation {
    pub calling_range: Range
}

impl Agent for PassiveCallingStation {
    fn decide_round(&self, round_info: &AgentRoundInfo,
        agent_state: &AgentState,  game_state: &GameState) -> Action {


        match round_info.round {
            Round::Preflop => {

                //poker_eval as a different idea of card

                //not handling all ins
                if self.calling_range.data[agent_state.get_range_index_for_hole_cards()] > 0.0 {
                    Action::Call
                } else {
                    Action::Fold
                }
            },
            Round::Flop => {
                Action::Call
            },
            Round::Turn => {
                Action::Call
            },
            Round::River => {
                Action::Call
            },
        }
    }
    
}
