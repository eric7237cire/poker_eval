

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
        agent_state.initial_stack = stack;

        let str1: String = hole_cards.chars().take(2).collect();
        let str2: String = hole_cards.chars().skip(2).take(2).collect();

        agent_state.cards.push(
            PsCard::try_from(str1.as_str()).unwrap()
        );
        agent_state.cards.push(
            PsCard::try_from(str2.as_str()).unwrap()
        );
    }

    fn run_round(&mut self, round: Round) {

        let mut to_act =
        if round == Round::Preflop {
            Position::Utg
        } else {
            Position::SmallBlind
        };

        let current_amt_to_call = if round == Round::Preflop {
             self.big_blind } else {0};

         //Initialize round
        for agent_state in &mut self.agent_states {
            agent_state.already_bet = 0;
        }

        //Preflop blinds
        if round == Round::Preflop {
            self.game_state.current_pot += self.agent_states[Position::SmallBlind as usize].handle_put_money_in_pot(self.small_blind);
            self.game_state.current_pot += self.agent_states[Position::BigBlind as usize].handle_put_money_in_pot(self.big_blind);
        }

        //Initialize for preflop round
        let mut agent_round_info = AgentRoundInfo {
            //agents_already_acted: 0,
            agents_left_to_act: self.agents.len() as u8,
            current_amt_to_call,
            min_raise: self.big_blind,
            round,
            bb_amt: self.big_blind,
        };

        let loop_check = 20;
        let mut loop_counter = 0;
        loop {

            loop_counter += 1;
            assert!(loop_counter < loop_check);

            //Ask agent what it wants to do 
            let agent = &mut self.agents[to_act as usize];
            let agent_state = &mut self.agent_states[to_act as usize];

            debug!("In round {:?}, position {:?}, stack: {}, already_bet: {}, folded: {}", 
                agent_round_info.round, agent_state.position, agent_state.stack, agent_state.already_bet, agent_state.folded,
                
            );
            
            if !agent_state.folded && agent_state.stack > 0 {
                let did_raise = handle_player_action(agent, to_act, &mut agent_round_info, &mut self.game_state, agent_state);

                if did_raise {
                    debug!("Player {:?} raised to {}", to_act, agent_round_info.current_amt_to_call);
                    agent_round_info.agents_left_to_act = self.agents.len() as u8;
                }
            }

            //End condition is to_act==last_to_act and all players have called or folded
            let agent_state = &self.agent_states[to_act as usize];
            agent_round_info.agents_left_to_act -= 1;

            if agent_round_info.agents_left_to_act == 0 { 
                debug!("End of round");
                break;
            }

            to_act = to_act.next();
        }
    }

    fn run_game(&mut self) {

        debug!("Running game");

        
        //Now loop until all players either folded, called what they need to, or are all in
        
        self.run_round(Round::Preflop);
        self.run_round(Round::Flop);
        self.run_round(Round::Turn);
        self.run_round(Round::River);


        
        


    }

    

    
}

//return true if raise
fn handle_player_action( 
    agent: &Box<dyn Agent + '_>, to_act: Position, 
    agent_round_info: &mut AgentRoundInfo, 
    game_state: &mut GameState,
    agent_state: &mut AgentState) -> bool {
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
        Action::Check => {
            assert_eq!(agent_round_info.current_amt_to_call, 0);
            assert_eq!(agent_state.already_bet, 0);
            false
        }
        Action::Call => {

            assert!(agent_round_info.current_amt_to_call > 0, "Cannot call a 0 bet, that is a check");

            let in_pot = agent_state.handle_put_money_in_pot(agent_round_info.current_amt_to_call);
            if agent_state.stack > 0 {
                //actually the agent can be calling a raise so the amount put in pot may be less than the amount to call
                //assert_eq!(in_pot, agent_round_info.current_amt_to_call);        
                assert_eq!(agent_state.already_bet, agent_round_info.current_amt_to_call);
            } 
            game_state.current_pot+= in_pot;

            assert!(agent_state.already_bet > 0);
            false
        },
        Action::Raise(bet) => {
            
            debug!("Player {:?} raised to {}.  Min raise {} cur to call {}", 
                to_act, bet,
                agent_round_info.min_raise, agent_round_info.current_amt_to_call);

            game_state.current_pot += agent_state.handle_put_money_in_pot(bet);

            //If the player raised to X, then that's exactly what should have been bet
            assert_eq!(agent_state.already_bet, bet);

            //The bet has to be at least the size of the previous raise more than amt to call, unless it's an all in
            if agent_state.stack > 0 {

                assert!(bet >= agent_round_info.current_amt_to_call + agent_round_info.min_raise);

                agent_round_info.min_raise = bet - agent_round_info.current_amt_to_call;

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
            let action_counter = *self.action_counter.borrow();
            *self.action_counter.borrow_mut() += 1;

            
            //First to act should be utfg
            match action_counter {
                 0 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position, Position::Utg);

                    assert_eq!(agent_state.cards[0].value, PsValue::King);
                    assert_eq!(agent_state.cards[0].suit, PsSuit::Diamond);

                    let range_index = agent_state.get_range_index_for_hole_cards();
                    assert_eq!(range_index, card_pair_to_index(
                        card_from_str("Kd").unwrap(), card_from_str("Kh").unwrap()
                        ));

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt);
                    assert_eq!(game_state.current_pot, 3);

                    return Action::Call;
                 },
                 1 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position, Position::HiJack);

                    assert_eq!(agent_state.cards[0].value, PsValue::Ace);
                    assert_eq!(agent_state.cards[0].suit, PsSuit::Spade);

                    assert_eq!(game_state.current_pot, 5);
                    
                    return Action::Raise(round_info.min_raise + round_info.bb_amt * 3);
                 },
                    2 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::Button);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 4);
                        assert_eq!(8, round_info.current_amt_to_call);
                        assert_eq!(agent_state.cards[0].value, PsValue::King);
                        assert_eq!(agent_state.cards[0].suit, PsSuit::Spade);

                        assert_eq!(agent_state.already_bet, 0);

                        assert_eq!(game_state.current_pot, 5+8);
    
                        return Action::Call;
                    },
                    3 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::SmallBlind);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 4);
                        
                        //already posted big blind
                        assert_eq!(agent_state.stack, 69);
                        assert_eq!(agent_state.already_bet, 1);

                        assert_eq!(game_state.current_pot, 5+8+8);

                        assert_eq!(round_info.min_raise, 6);
    
                        return Action::Raise(16);
                    },
                    4 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::BigBlind);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 8);
                        
                        //already posted big blind
                        assert_eq!(agent_state.stack, 18);
                        assert_eq!(agent_state.stack, agent_state.initial_stack - round_info.bb_amt);
                        assert_eq!(agent_state.already_bet, 2);

                        assert_eq!(game_state.current_pot, 36); 


                        assert_eq!(round_info.min_raise, 8);
    
                        //min raise. effectively an all in
                        return Action::Raise(agent_state.initial_stack);
                    },
                    5 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::Utg);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);
                        
                        //already posted big blind
                        assert_eq!(agent_state.stack, 38);
                        assert_eq!(agent_state.stack, agent_state.initial_stack - round_info.bb_amt);
                        assert_eq!(agent_state.already_bet, round_info.bb_amt);

                        assert_eq!(game_state.current_pot, 54);

                        assert_eq!(round_info.min_raise, 8);
    
                        return Action::Call;
                    },
                    6 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::HiJack);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);
                        
                        //already posted big blind
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 8);
                        assert_eq!(agent_state.stack, 17);
                        
                        assert_eq!(agent_state.already_bet, round_info.bb_amt*4);

                        assert_eq!(game_state.current_pot, 72);
    
                        return Action::Call;
                    },
                    7 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::Button);
    
                        assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 8);
                        assert_eq!(agent_state.stack, 21);
                        
                        assert_eq!(agent_state.already_bet, round_info.bb_amt*4);

                        assert_eq!(game_state.current_pot, 84);
    
                        return Action::Raise(agent_state.initial_stack);
                    },
                    8 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        assert_eq!(agent_state.position, Position::SmallBlind);
    
                        assert_eq!(round_info.current_amt_to_call, 29);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 16);
                        assert_eq!(agent_state.stack, 54);
                        
                        assert_eq!(agent_state.already_bet, 16);

                        assert_eq!(game_state.current_pot, 105);
    
                        return Action::Call;
                    },
                    9 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        //BB is all in
                        assert_eq!(agent_state.position, Position::Utg);
    
                        assert_eq!(round_info.current_amt_to_call, 29);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 20);
                        assert_eq!(agent_state.stack, 20);
                        
                        assert_eq!(agent_state.already_bet, 20);

                        assert_eq!(round_info.min_raise, 8);

                        assert_eq!(game_state.current_pot, 118);
    
                        //min raise 37 but we want it divisible by bb (2)
                        return Action::Raise(38);
                    },
                    10 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        //BB is all in
                        assert_eq!(agent_state.position, Position::HiJack);
    
                        assert_eq!(round_info.current_amt_to_call, 38);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 20);
                        assert_eq!(agent_state.stack, 5);
                        
                        assert_eq!(agent_state.already_bet, 20);

                        //assert_eq!(round_info.min_raise, 8);

                        assert_eq!(game_state.current_pot, 136);
    
                        //puts us all in
                        return Action::Call;
                    },
                    11 => {
                        assert_eq!(round_info.round, Round::Preflop);
                        //BB is all in
                        assert_eq!(agent_state.position, Position::SmallBlind);
    
                        assert_eq!(round_info.current_amt_to_call, 38);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 29);
                        assert_eq!(agent_state.stack, 41);
                        
                        assert_eq!(agent_state.already_bet, 29);

                        //assert_eq!(round_info.min_raise, 8);

                        assert_eq!(game_state.current_pot, 141);
    
                        return Action::Call;
                    },
                    12 => {
                        assert_eq!(round_info.round, Round::Flop);
                        assert_eq!(agent_state.position, Position::SmallBlind);
    
                        assert_eq!(round_info.current_amt_to_call, 0);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 38);
                        assert_eq!(agent_state.stack, 32);
                        
                        assert_eq!(agent_state.already_bet, 0);

                        //assert_eq!(round_info.min_raise, 8);

                        assert_eq!(game_state.current_pot, 150);
    
                        return Action::Check;
                    },

                    13 => {
                        assert_eq!(round_info.round, Round::Flop);
                        assert_eq!(agent_state.position, Position::Utg);
    
                        assert_eq!(round_info.current_amt_to_call, 0);
                        
                        assert_eq!(agent_state.stack, agent_state.initial_stack - 38);
                        assert_eq!(agent_state.stack, 2);
                        
                        assert_eq!(agent_state.already_bet, 0);

                        //assert_eq!(round_info.min_raise, 8);

                        assert_eq!(game_state.current_pot, 150);
    
                        return Action::Check;
                    },
                    14 => {
                        assert_eq!(round_info.round, Round::Turn);
                        assert_eq!(agent_state.position, Position::SmallBlind);

                        return Action:: Check;
                    },
                    15 => {
                        assert_eq!(round_info.round, Round::Turn);
                        assert_eq!(agent_state.position, Position::Utg);

                        return Action:: Raise(2);
                    },
                    16 => {
                        assert_eq!(round_info.round, Round::Turn);
                        assert_eq!(agent_state.position, Position::SmallBlind);

                        return Action:: Call;
                    },
                    17 => {
                        assert_eq!(round_info.round, Round::River);
                        assert_eq!(agent_state.position, Position::SmallBlind);

                        //No one left in hand?
                        return Action::Check;
                    },


                 df => {
                    assert!(false, "Unexpected action counter: {}", df);
                 }
            }

            
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
        game_runner.init_agent_state(Position::Button, "KsKc", 29);
        
        //0. utg limps in for 2; 
        //1. hj raises to 8; pot = 5 (raise amount 6)
        //2. button calls 8; pot = 13
        //3. sb raises to 16; pot = 21 (adds 15 more), raise amount 8
        //4. bb goes all in for 20; pot = 36 (adds 18 more)
        //5. utg calls; pot = 54 (adds 18 more)
        //6. hj calls 20; pot = 72 (adds 20-8 more)
        //7. button goes all in, 21 more
        //8. sb calls 29
        //9. bb is already all in, utg raises to 38
        //10. hj calls (all in)
        //11. button already all in, sb calls action stops

        //12 flop utg checks
        //13 sb checks
        //14 turn utg all in
        //15 sb calls

        //https://poker.stackexchange.com/questions/158/minimum-re-raise-in-hold-em
        //https://www.reddit.com/r/poker/comments/g4n0oc/minimum_reraise_in_texas_holdem/
        game_runner.run_game();


        assert_eq!(18, *action_counter.borrow());

        //assert!(false);
    }

}