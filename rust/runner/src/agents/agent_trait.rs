use crate::{Action, AgentState, ChipType, GameState, Round};

//For convenience, also build a struct that gives agent relavant info

pub struct AgentRoundInfo {
    //pub agents_already_acted: u8,
    pub agents_left_to_act: u8,

    //They need to call this - already_bet
    pub current_amt_to_call: ChipType,

    //https://www.reddit.com/r/poker/comments/oqrmyk/minimal_raise/
    pub min_raise: ChipType,

    pub bb_amt: ChipType,

    pub round: Round,
}

pub trait Agent {
    //Get hand cards with index_to_card_pair
    fn decide_round(
        //To be able to mutate internal state
        &self,
        round_info: &AgentRoundInfo,
        agent_state: &AgentState,
        game_state: &GameState,
    ) -> Action;
}
