use core::num;

use log::debug;
use poker_rs::arena::game_state;

use crate::{GameState, ChipType, Agent, Position, AgentRoundInfo, Round, Action, AgentState};

pub struct GameRunner {
    game_state: GameState,
    
    //Order is sb, bb, utg, hj, button
    pub agent_states: Vec<AgentState>,

    agents: Vec<Box<dyn Agent>>,
    small_blind: ChipType,
    big_blind: ChipType,
}

impl GameRunner {

    fn new(agents: Vec<Box<dyn Agent>>, small_blind: ChipType) -> GameRunner {
        let mut agent_states = Vec::new();
        for _ in 0..agents.len() {
            agent_states.push(AgentState::default());
        }

        GameRunner {
            game_state: GameState::new(agents.len() as u8),
            agent_states: agent_states,
            agents: agents,
            small_blind: small_blind,
            big_blind: small_blind*2,
        }
    }

    fn run_game(&mut self) {

        debug!("Running game");

        //Preflop
        self.game_state.current_pot += self.agent_states[Position::SmallBlind as usize].handle_put_money_in_pot(self.small_blind);
        self.game_state.current_pot += self.agent_states[Position::BigBlind as usize].handle_put_money_in_pot(self.big_blind);

        //Now loop until all players either folded, called what they need to, or are all in
        let last_to_act = Position::BigBlind;

        let mut to_act =Position::BigBlind.next();

        //Initialize round
        for agent_state in &mut self.agent_states {
            agent_state.already_bet = 0;
        }
       

        //minus folded and all_in
        let mut num_effective_players = self.agents.len() as u8;

        //Initialize for preflop round
        let mut agent_round_info = AgentRoundInfo {
            agents_already_acted: 0,
            agents_left_to_act: num_effective_players,
            current_amt_to_call: self.big_blind,
            prev_raise_amt: self.big_blind,
            round: Round::Preflop,
            bb_amt: self.big_blind,
        };

        let loop_check = 10;
        let mut loop_counter = 0;
        loop {

            loop_counter += 1;
            assert!(loop_counter < loop_check);

            //Ask agent what it wants to do 
            let agent = &self.agents[to_act as usize];
            let agent_state = &mut self.agent_states[to_act as usize];
            
            if !agent_state.folded && agent_state.stack > 0 {
                handle_player_action(agent, to_act, &mut agent_round_info, &mut self.game_state, agent_state, &mut num_effective_players);

                if agent_state.folded || agent_state.stack <= 0 {
                    num_effective_players -= 1;
                }
            }

            //End condition is to_act==last_to_act and all players have called or folded
            if to_act == last_to_act && agent_round_info.current_amt_to_call == self.agent_states[to_act as usize].already_bet {
                debug!("End of round");
                break;
            }

            if to_act == last_to_act {
                agent_round_info.agents_already_acted = 0;
                agent_round_info.agents_left_to_act = num_effective_players;
            }

            agent_round_info.agents_left_to_act -= 1;
            assert!(agent_round_info.agents_left_to_act <= num_effective_players);

            to_act = to_act.next();
        }


    }

    

    
}

fn handle_player_action( 
    agent: &Box<dyn Agent>, to_act: Position, 
    agent_round_info: &mut AgentRoundInfo, 
    game_state: &mut GameState,
    agent_state: &mut AgentState,
    num_effective_players: &mut u8)  {
    let action = agent.decide_round(&agent_round_info,
        &agent_state, game_state);

    debug!("Position to act: {:?}, action: {:?} cur amt. to call: {}", to_act, action,
        agent_round_info.current_amt_to_call
    );

    match action {
        Action::Fold => {
            agent_state.folded = true;

        },
        Action::Call => {
            let in_pot = agent_state.handle_put_money_in_pot(agent_round_info.current_amt_to_call);
            if agent_state.stack > 0 {
                assert_eq!(in_pot, agent_round_info.current_amt_to_call);                
            }
            game_state.current_pot+= in_pot;
            
        },
        Action::Raise(bet) => {
            
            game_state.current_pot+= agent_state.handle_put_money_in_pot(bet);

            //The bet has to be at least the size of the previous raise more than amt to call, unless it's an all in
            if agent_state.stack > 0 {
                assert!(bet >= agent_round_info.current_amt_to_call + agent_round_info.prev_raise_amt);

                //Divisible by # of big blinds
                assert!(bet % agent_round_info.bb_amt == 0);
            }
            
            //In all cases must be at least the size of the big blind
            agent_round_info.current_amt_to_call = bet;
            
            
        }
        
    }

}


#[cfg(test)]
mod tests {
    use crate::{PassiveCallingStation, build_range, PreFrabRanges, GameRunner};

    pub use super::*;
    
    //env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    fn init() {
        let _ = env_logger::builder().is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
    }

    struct TestPlayerAgent {
        round_counter: u8,
    }
    
    impl Agent for TestPlayerAgent {
        fn decide_round(&mut self, round_info: &AgentRoundInfo,
            agent_state: &AgentState,  game_state: &GameState) -> Action {
    
            self.round_counter += 1;
            Action::Fold
            //match round_info.round {
        }
    }


    #[test]
    fn test_split_pots() {
        init();

        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        let num_players = 5;

        for _ in 0..num_players {
            agents.push(Box::new(
                PassiveCallingStation {
                    calling_range: build_range(PreFrabRanges::Range75)
                }
            ));
        }
         
        let mut game_runner = GameRunner::new(agents, 1);

        game_runner.run_game();


        assert!(false);
    }

}