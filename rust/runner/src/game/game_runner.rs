

use log::debug;

use crate::{GameState, ChipType, Agent, Position, AgentRoundInfo, Round, Action, AgentState};
use poker_rs::core::{Card as PsCard, Value as PsValue, Suit as PsSuit};

pub struct GameRunner<'a> {
    game_state: GameState,
    
    //Order is sb, bb, utg, hj, button
    pub agent_states: Vec<AgentState>,

    agents: Vec<Box<dyn Agent + 'a>>,
    small_blind: ChipType,
    big_blind: ChipType,
}

impl <'a> GameRunner<'a>  {

    fn new(agents: Vec<Box<dyn Agent + 'a>>, small_blind: ChipType) -> GameRunner {
        let mut agent_states = Vec::new();
        for pos_idx in 0..agents.len() {
            let mut agent_state = AgentState::default();
            agent_state.position = Position::from_usize(pos_idx);
            agent_states.push(agent_state);
        }

        GameRunner {
            game_state: GameState::new(agents.len() as u8),
            agent_states: agent_states,
            agents: agents,
            small_blind: small_blind,
            big_blind: small_blind*2,
        }
    }

    fn init_agent_state(&mut self, position: Position, hole_cards: &str, stack: ChipType) {
        let agent_state = &mut self.agent_states[position as usize];
        assert_eq!(    agent_state.position, position);

        agent_state.stack = stack;

        let str1: String = hole_cards.chars().take(2).collect();
        let str2: String = hole_cards.chars().skip(2).take(2).collect();

        agent_state.cards.push(
            PsCard::try_from(str1.as_str()).unwrap()
        );
        agent_state.cards.push(
            PsCard::try_from(str2.as_str()).unwrap()
        );
    }

    fn run_game(&mut self) {

        debug!("Running game");

        //Preflop
        self.game_state.current_pot += self.agent_states[Position::SmallBlind as usize].handle_put_money_in_pot(self.small_blind);
        self.game_state.current_pot += self.agent_states[Position::BigBlind as usize].handle_put_money_in_pot(self.big_blind);

        //Now loop until all players either folded, called what they need to, or are all in
        let mut last_to_act = Position::BigBlind;

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
            let agent = &mut self.agents[to_act as usize];
            let agent_state = &mut self.agent_states[to_act as usize];

            debug!("In round {:?}, position {:?}, stack: {}, already_bet: {}, folded: {} last to act: {:?}", 
                agent_round_info.round, agent_state.position, agent_state.stack, agent_state.already_bet, agent_state.folded,
                last_to_act
            );
            
            if !agent_state.folded && agent_state.stack > 0 {
                let did_raise = handle_player_action(agent, to_act, &mut agent_round_info, &mut self.game_state, agent_state, &mut num_effective_players);

                if did_raise {
                    debug!("Player {:?} raised to {}", to_act, agent_round_info.current_amt_to_call);
                    last_to_act = to_act;
                }

                if agent_state.folded || agent_state.stack <= 0 {
                    num_effective_players -= 1;
                }
            }

            //End condition is to_act==last_to_act and all players have called or folded
            let agent_state = &self.agent_states[to_act as usize];
            if to_act == last_to_act && (
                agent_state.folded || agent_state.stack <= 0 || 
                agent_state.already_bet == agent_round_info.current_amt_to_call
            ) {
            
                
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

//return true if raise
fn handle_player_action( 
    agent: &Box<dyn Agent + '_>, to_act: Position, 
    agent_round_info: &mut AgentRoundInfo, 
    game_state: &mut GameState,
    agent_state: &mut AgentState,
    num_effective_players: &mut u8) -> bool {
    let action = agent.decide_round(&agent_round_info,
        &agent_state, game_state);

    debug!("Position to act: {:?}, action: {:?} cur amt. to call: {}", to_act, action,
        agent_round_info.current_amt_to_call
    );

    match action {
        Action::Fold => {
            agent_state.folded = true;
            false
        },
        Action::Call => {
            let in_pot = agent_state.handle_put_money_in_pot(agent_round_info.current_amt_to_call);
            if agent_state.stack > 0 {
                assert_eq!(in_pot, agent_round_info.current_amt_to_call);        
                assert_eq!(agent_state.already_bet, agent_round_info.current_amt_to_call);
            } 
            game_state.current_pot+= in_pot;

            assert!(agent_state.already_bet > 0);
            false
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
            
            true
        }
        
    }

}


#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use postflop_solver::{card_pair_to_index, card_from_str, Hand};

    use crate::{PassiveCallingStation, build_range, PreFrabRanges, GameRunner};

    use poker_rs::core::{Value as PsValue, Card as PsCard};

    pub use super::*;
    
    //env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    fn init() {
        let _ = env_logger::builder().is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
    }

    struct TestPlayerAgent<'a> {
        action_counter: &'a RefCell<u8>,
    }
    
    impl <'a>  Agent for TestPlayerAgent<'a> {
        fn decide_round(&self, round_info: &AgentRoundInfo,
            agent_state: &AgentState,  game_state: &GameState) -> Action {
    
            debug!("TEST Action : {} for position {:?}", self.action_counter.borrow(), agent_state.position);

            //First to act should be utfg
            match *self.action_counter.borrow() {
                 0 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position, Position::Utg);

                    assert_eq!(agent_state.cards[0].value, PsValue::King);
                    assert_eq!(agent_state.cards[0].suit, PsSuit::Diamond);

                    let range_index = agent_state.get_range_index_for_hole_cards();
                    assert_eq!(range_index, card_pair_to_index(
                        card_from_str("Kd").unwrap(), card_from_str("Kh").unwrap()
                        ));
                 },
                 df => {
                    assert!(false, "Unexpected action counter: {}", df);
                 }
            }

            *self.action_counter.borrow_mut() += 1;
            Action::Fold
            //match round_info.round {
        }
    }


    #[test]
    fn test_split_pots() {
        init();

        let action_counter = RefCell::new(0u8);

        
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        let num_players = 5;

        

        for _ in 0..num_players {
            agents.push(Box::new(
                TestPlayerAgent {
                    action_counter: &action_counter,
                }
            ));
        }
        
        let mut game_runner = GameRunner::new(agents, 1);

        game_runner.init_agent_state(Position::SmallBlind, "TsTc", 70);
        game_runner.init_agent_state(Position::BigBlind, "AdAh", 20);
        game_runner.init_agent_state(Position::Utg, "KdKh", 40);
        game_runner.init_agent_state(Position::HiJack, "AsAc", 25);
        game_runner.init_agent_state(Position::Button, "KsKc", 30);
        

        game_runner.run_game();

    

        assert!(false);
    }

}