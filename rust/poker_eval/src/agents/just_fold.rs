use crate::{ActionEnum, Agent, AgentRoundInfo, AgentState, OldGameState};

pub struct JustFold {}

impl Agent for JustFold {
    fn decide_round(
        &self,
        _round_info: &AgentRoundInfo,
        _agent_state: &AgentState,
        _game_state: &OldGameState,
    ) -> ActionEnum {
        ActionEnum::Fold
    }
}
