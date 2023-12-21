use crate::{ActionEnum, Agent, AgentRoundInfo, AgentState, GameState};

pub struct JustFold {}

impl Agent for JustFold {
    fn decide_round(
        &self,
        _round_info: &AgentRoundInfo,
        _agent_state: &AgentState,
        _game_state: &GameState,
    ) -> ActionEnum {
        ActionEnum::Fold
    }
}
