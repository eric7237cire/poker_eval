use crate::{Agent, Action, GameState, AgentRoundInfo, AgentState};


pub struct JustFold {
    
}

impl Agent for JustFold {
    fn decide_round(&self, round_info: &AgentRoundInfo,
        agent_state: &AgentState,  game_state: &GameState) -> Action {

         Action::Fold
               
    }
    
}
