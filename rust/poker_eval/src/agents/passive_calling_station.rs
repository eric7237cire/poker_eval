use crate::{ActionEnum, Agent, AgentRoundInfo, AgentState, OldGameState, Round};
use postflop_solver::Range;

//#[derive(Copy, Clone)]
pub struct PassiveCallingStation {
    pub calling_range: Range,
}

impl Agent for PassiveCallingStation {
    fn decide_round(
        &self,
        round_info: &AgentRoundInfo,
        agent_state: &AgentState,
        _game_state: &OldGameState,
    ) -> ActionEnum {
        match round_info.round {
            Round::Preflop => {
                //poker_eval as a different idea of card

                //not handling all ins
                if self.calling_range.data[agent_state.get_range_index_for_hole_cards()] > 0.0 {
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
}
