
use postflop_solver::{Range, Card, Hand};

enum Round {
    Preflop,
    Flop,
    Turn,
    River,
}

struct GameState {
    //Big blind is assumed to be 1 
    num_callers: u8,
    //In terms of # of big blinds
    current_pot: u16,

    to_call: u16,

    agents: Vec<AgentState>,

    //index 0,1,2 for flop 3 and 4 for river
    common_cards: Vec<Card>,
    common_hand: Hand
}

struct AgentState {
    //In terms of # of big blinds
    stack: u16,
    position: Position,
    //Index into range
    hole_cards: usize
}

enum Position {    
    SmallBlind,
    BigBlind,
    Utg,
    HiJack,
    Button,
}

//Preflop goes left of bb, bb is last

//Then flop/turn/river order
//sb, bb, .... button

//button, sb, bb 



#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Action {
    Fold,
    Call,
    Raise(u16),
}

//Any probabilities are handled in the agent, but they always do one thing

trait Agent {
    //Get hand cards with index_to_card_pair
    fn decide_round(&self, agent_index: usize, round: &Round, game_state: &GameState) -> Action;

    
}

enum PreFrabRanges {
    RANGE_ALL,
    RANGE_75,
    RANGE_50,
    RANGE_25,
}


fn build_range(range: PreFrabRanges) -> Range {
    match range {
        PreFrabRanges::RANGE_ALL => "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap(),
        PreFrabRanges::RANGE_75 => "22+, A2s+, K2s+, Q2s+, J2s+, T2s+, 92s+, 82s+, 72s+, 62s+, 52s+, 42s+, A2o+, K2o+, Q2o+, J4o+, T6o+, 96o+, 86o+, 76o".parse().unwrap(),
        PreFrabRanges::RANGE_50 => "22+, A2s+, K2s+, Q2s+, J2s+, T5s+, 96s+, 86s+, 75s+, A2o+, K5o+, Q7o+, J8o+, T8o+".parse().unwrap(),
        PreFrabRanges::RANGE_25 => "55+, A2s+, K5s+, Q8s+, J8s+, T9s, A8o+, K9o+, QTo+, JTo".parse().unwrap(),
    }
}


//Once we run it, we produce a probability distribution of the hands


struct PassiveCallingStation {
    calling_range: Range
}

impl Agent for PassiveCallingStation {
    fn decide_round(&self, agent_index: usize, round: &Round, game_state: &GameState) -> Action {

        let this_agent = &game_state.agents[agent_index];

        match round {
            Round::Preflop => {
                //not handling all ins
                if self.calling_range.data[this_agent.hole_cards] > 0.0 {
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




#[cfg(test)]
mod tests {
    use postflop_solver::Hand;

    use crate::agent::{PassiveCallingStation, GameState, PreFrabRanges, build_range, Agent, Round, Action};

    #[test]
    fn test_passive_agent() {
        let agent = PassiveCallingStation {
            calling_range: build_range(PreFrabRanges::RANGE_75)
        };

        let game_state = GameState {
            num_callers: 0,
            current_pot: 0,
            to_call: 0,
            agents: vec![],
            common_hand: Hand::new(),
            common_cards: vec![]
        };

        let action = agent.decide_round(0, &Round::Preflop, &game_state);
        assert_eq!(action, Action::Call);
    }
}