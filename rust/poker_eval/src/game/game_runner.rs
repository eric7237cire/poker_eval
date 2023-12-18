use log::debug;

use crate::{
    core::Card, rank_cards, Action, Agent, AgentRoundInfo, AgentState, ChipType, GameState,
    Position, Round, SMALL_BLIND, BIG_BLIND,
};


#[allow(dead_code)]
pub struct GameRunner<'a> {
    game_state: GameState,

    //Order is sb, bb, utg, hj, button
    pub agent_states: Vec<AgentState>,

    agents: Vec<Box<dyn Agent + 'a>>,
    small_blind: ChipType,
    big_blind: ChipType,
}

#[allow(dead_code)]
impl<'a> GameRunner<'a> {
    fn new(agents: Vec<Box<dyn Agent + 'a>>, small_blind: ChipType) -> GameRunner {
        let mut agent_states = Vec::new();
        for pos_idx in 0..agents.len() {
            let mut agent_state = AgentState::default();
            agent_state.position = pos_idx.try_into().unwrap();
            agent_states.push(agent_state);
        }

        GameRunner {
            game_state: GameState::new(agents.len() as u8),
            agent_states: agent_states,
            agents: agents,
            small_blind: small_blind,
            big_blind: small_blind * 2,
        }
    }

    fn init_agent_pos_and_stack(&mut self, position: Position, stack: ChipType) {
        let agent_state: &mut AgentState = &mut self.agent_states[usize::try_from(position).unwrap()];
        assert_eq!(agent_state.position,  position);

        agent_state.stack = stack;
        agent_state.initial_stack = stack;
    }

    fn run_round(&mut self, round: Round) {
        let mut to_act = Position::first_to_act(self.agents.len() as _, round);
        
        let current_amt_to_call = if round == Round::Preflop {
            self.big_blind
        } else {
            0
        };

        //Initialize round
        for agent_state in &mut self.agent_states {
            agent_state.already_bet = 0;
        }

        //Preflop blinds
        if round == Round::Preflop {
            self.game_state.current_pot += self.agent_states[usize::from(SMALL_BLIND)]
                .handle_put_money_in_pot(self.small_blind);
            self.game_state.current_pot += self.agent_states[usize::from(BIG_BLIND)]
                .handle_put_money_in_pot(self.big_blind);
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
            let agent = &mut self.agents[usize::from(to_act)];
            let agent_state = &mut self.agent_states[usize::from(to_act)];

            debug!(
                "In round {:?}, position {:?}, stack: {}, already_bet: {}, folded: {}",
                agent_round_info.round,
                agent_state.position,
                agent_state.stack,
                agent_state.already_bet,
                agent_state.folded,
            );

            if !agent_state.folded && agent_state.stack > 0 {
                let did_raise = handle_player_action(
                    agent,
                    to_act,
                    &mut agent_round_info,
                    &mut self.game_state,
                    agent_state,
                );

                if did_raise {
                    debug!(
                        "Player {:?} raised to {}",
                        to_act, agent_round_info.current_amt_to_call
                    );
                    agent_round_info.agents_left_to_act = self.agents.len() as u8;
                }
            }

            //End condition is to_act==last_to_act and all players have called or folded
            let _agent_state = &self.agent_states[usize::from(to_act)];
            agent_round_info.agents_left_to_act -= 1;

            if agent_round_info.agents_left_to_act == 0 {
                debug!("End of round");
                break;
            }

            to_act = to_act.next(self.agents.len() as _);
        }
    }

    //returns winnings for each player and chips put in pot, net winnings winning-chips put in pot'
    fn run_game(&mut self, cards: &[Card]) -> (Vec<ChipType>, Vec<ChipType>) {
        debug!("Running game");

        //need cards for all agents + 5 community cards
        assert_eq!(cards.len(), self.agents.len() * 2 + 5);

        //Deal hole cards to agents
        let mut card_index = 0;
        for agent_state in &mut self.agent_states {
            agent_state.cards.push(cards[card_index]);
            card_index += 1;
            agent_state.cards.push(cards[card_index]);
            card_index += 1;
        }

        //Now loop until all players either folded, called what they need to, or are all in

        self.run_round(Round::Preflop);

        //deal flop to all agents
        for agent_state in &mut self.agent_states {
            agent_state.cards.push(cards[card_index]);
            agent_state.cards.push(cards[card_index + 1]);
            agent_state.cards.push(cards[card_index + 2]);
        }
        card_index += 3;

        self.run_round(Round::Flop);

        //deal turn to all agents
        for agent_state in &mut self.agent_states {
            agent_state.cards.push(cards[card_index]);
        }
        self.run_round(Round::Turn);
        card_index += 1;

        //deal flop to all agents
        for agent_state in &mut self.agent_states {
            agent_state.cards.push(cards[card_index]);
        }
        self.run_round(Round::River);

        //Return how much each player won
        let hand_ranks = self
            .agent_states
            .iter()
            .map(|agent_state| rank_cards(&agent_state.cards))
            .collect::<Vec<_>>();

        let mut sorted_hand_ranks = self
            .agent_states
            .iter()
            .enumerate()
            .filter_map(|(agent_index, agent_state)| {
                if agent_state.folded {
                    return None;
                }
                Some((hand_ranks[agent_index], agent_index))
            })
            .collect::<Vec<_>>();

        let mut amount_bet_by_showdown_agent = self
            .agent_states
            .iter()
            .enumerate()
            .filter_map(|(agent_index, agent_state)| {
                if agent_state.folded {
                    return None;
                }
                Some((agent_state.initial_stack - agent_state.stack, agent_index))
            })
            .collect::<Vec<_>>();

        //Sort by rank, first one is highest rank
        sorted_hand_ranks.sort_by(|a, b| b.0.cmp(&a.0));

        //Sort by stack size, first one is smallest amount bet
        amount_bet_by_showdown_agent.sort_by(|a, b| a.0.cmp(&b.0));

        //assert!(hand_ranks[0].0 > hand_ranks[4].0);

        let mut winnings = vec![0; self.agents.len()];

        //sanity check amount bet total == pot
        let total_bet = self
            .agent_states
            .iter()
            .map(|agent_state| agent_state.initial_stack - agent_state.stack)
            .sum::<ChipType>();

        assert_eq!(total_bet, self.game_state.current_pot);

        //we are left with all the hands that are in the showdown

        //start with smallest stack, if it didn't win, then it's out
        while !sorted_hand_ranks.is_empty() {
            assert_eq!(sorted_hand_ranks.len(), amount_bet_by_showdown_agent.len());
            let small_stack_agent_index = amount_bet_by_showdown_agent[0].1;

            let small_stack_rank = hand_ranks[small_stack_agent_index];

            if small_stack_rank < sorted_hand_ranks[0].0 {
                //small stack didn't win, just remove them
                amount_bet_by_showdown_agent.remove(0);

                let shr_pos_to_remove = sorted_hand_ranks
                    .iter()
                    .position(|(_, ag_index)| *ag_index == small_stack_agent_index)
                    .unwrap();
                sorted_hand_ranks.remove(shr_pos_to_remove);
            } else {
                //small stack won, me take max amount bet (== the small stack) and subtract from remaining stacks
                //the pot gets reduced by the delta between the remaining winning stacks amount bet and the small stack amount bet
                let amount_put_in_pot_by_current_winner = amount_bet_by_showdown_agent[0].0;
                let remaining_agent_side_pot =
                    sorted_hand_ranks.len() as ChipType * amount_put_in_pot_by_current_winner;
                let remaining_agent_total_put_in_pot = amount_bet_by_showdown_agent
                    .iter()
                    .map(|(amt_bet, _)| *amt_bet)
                    .sum::<ChipType>();
                assert!(remaining_agent_total_put_in_pot >= remaining_agent_side_pot);

                let pot_to_split = self.game_state.current_pot - remaining_agent_total_put_in_pot
                    + remaining_agent_side_pot;

                debug!(
                    "Pot to split: {} of pot {}",
                    pot_to_split, self.game_state.current_pot
                );
                self.game_state.current_pot -= pot_to_split;

                let mut current_winning_hand_end = 0;

                //find last hand rank that is equal to the first one
                for (i, hand_rank) in sorted_hand_ranks.iter().enumerate() {
                    if hand_rank.0 == sorted_hand_ranks[0].0 {
                        current_winning_hand_end = i;
                    } else {
                        break;
                    }
                }

                //For everyone else, reduce the amount they put in the pot by the amount the small stack put in
                for ab_index in 1..sorted_hand_ranks.len() {
                    amount_bet_by_showdown_agent[ab_index].0 -= amount_put_in_pot_by_current_winner;
                }

                let num_tied = (current_winning_hand_end - 0 + 1) as ChipType;

                //Give side pot / num_tied to each winner
                for sh_index in (0..=current_winning_hand_end).rev() {
                    let agent_index = sorted_hand_ranks[sh_index].1;

                    //This is + since 2 players that tied will get processed once for the smallest stack, and twice
                    //for the bigger stack, so we will add the second one with num_tied=0
                    winnings[agent_index] += pot_to_split / num_tied;
                }

                //Remove the small stack we just processed
                amount_bet_by_showdown_agent.remove(0);

                let shr_pos_to_remove = sorted_hand_ranks
                    .iter()
                    .position(|(_, ag_index)| *ag_index == small_stack_agent_index)
                    .unwrap();
                sorted_hand_ranks.remove(shr_pos_to_remove);
            }
        }

        let losses = self
            .agent_states
            .iter()
            .map(|agent_state| agent_state.initial_stack - agent_state.stack)
            .collect::<Vec<_>>();

        (winnings, losses)
    }
}

//return true if raise
fn handle_player_action(
    agent: &Box<dyn Agent + '_>,
    to_act: Position,
    agent_round_info: &mut AgentRoundInfo,
    game_state: &mut GameState,
    agent_state: &mut AgentState,
) -> bool {
    let action = agent.decide_round(&agent_round_info, &agent_state, game_state);

    debug!(
        "Position to act: {:?}, action: {:?} cur amt. to call: {}",
        to_act, action, agent_round_info.current_amt_to_call
    );

    match action {
        Action::Fold => {
            agent_state.folded = true;
            false
        }
        Action::Check => {
            assert_eq!(agent_round_info.current_amt_to_call, 0);
            assert_eq!(agent_state.already_bet, 0);
            false
        }
        Action::Call => {
            assert!(
                agent_round_info.current_amt_to_call > 0,
                "Cannot call a 0 bet, that is a check"
            );

            let in_pot = agent_state.handle_put_money_in_pot(agent_round_info.current_amt_to_call);
            if agent_state.stack > 0 {
                //actually the agent can be calling a raise so the amount put in pot may be less than the amount to call
                //assert_eq!(in_pot, agent_round_info.current_amt_to_call);
                assert_eq!(
                    agent_state.already_bet,
                    agent_round_info.current_amt_to_call
                );
            }
            game_state.current_pot += in_pot;

            assert!(agent_state.already_bet > 0);
            false
        }
        Action::Raise(bet) => {
            debug!(
                "Player {:?} raised to {}.  Min raise {} cur to call {}",
                to_act, bet, agent_round_info.min_raise, agent_round_info.current_amt_to_call
            );

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

    use postflop_solver::{card_from_str, card_pair_to_index};

    use crate::{CardValue, Suit};

    pub use super::*;

    //env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();
    }

    struct TestPlayerAgent<'a> {
        action_counter: &'a RefCell<u8>,
    }

    impl<'a> Agent for TestPlayerAgent<'a> {
        fn decide_round(
            &self,
            round_info: &AgentRoundInfo,
            agent_state: &AgentState,
            game_state: &GameState,
        ) -> Action {
            debug!(
                "TEST Action : {} for position {:?}",
                self.action_counter.borrow(),
                agent_state.position
            );
            let action_counter = *self.action_counter.borrow();
            *self.action_counter.borrow_mut() += 1;

            //First to act should be utfg
            match action_counter {
                0 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position), 2usize);

                    assert_eq!(agent_state.cards[0].value, CardValue::King);
                    assert_eq!(agent_state.cards[0].suit, Suit::Diamond);

                    let range_index = agent_state.get_range_index_for_hole_cards();
                    assert_eq!(
                        range_index,
                        card_pair_to_index(
                            card_from_str("Kd").unwrap(),
                            card_from_str("Kh").unwrap()
                        )
                    );

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt);
                    assert_eq!(game_state.current_pot, 3);

                    return Action::Call;
                }
                1 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position),  3usize);

                    assert_eq!(agent_state.cards[0].value, CardValue::Ace);
                    assert_eq!(agent_state.cards[0].suit, Suit::Spade);

                    assert_eq!(game_state.current_pot, 5);

                    return Action::Raise(round_info.min_raise + round_info.bb_amt * 3);
                }
                2 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position),  4usize);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 4);
                    assert_eq!(8, round_info.current_amt_to_call);
                    assert_eq!(agent_state.cards[0].value, CardValue::King);
                    assert_eq!(agent_state.cards[0].suit, Suit::Spade);

                    assert_eq!(agent_state.already_bet, 0);

                    assert_eq!(game_state.current_pot, 5 + 8);

                    return Action::Call;
                }
                3 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 4);

                    //already posted big blind
                    assert_eq!(agent_state.stack, 69);
                    assert_eq!(agent_state.already_bet, 1);

                    assert_eq!(game_state.current_pot, 5 + 8 + 8);

                    assert_eq!(round_info.min_raise, 6);

                    return Action::Raise(16);
                }
                4 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position,  BIG_BLIND);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 8);

                    //already posted big blind
                    assert_eq!(agent_state.stack, 18);
                    assert_eq!(
                        agent_state.stack,
                        agent_state.initial_stack - round_info.bb_amt
                    );
                    assert_eq!(agent_state.already_bet, 2);

                    assert_eq!(game_state.current_pot, 36);

                    assert_eq!(round_info.min_raise, 8);

                    //min raise. effectively an all in
                    return Action::Raise(agent_state.initial_stack);
                }
                5 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position),  2usize);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);

                    //already posted big blind
                    assert_eq!(agent_state.stack, 38);
                    assert_eq!(
                        agent_state.stack,
                        agent_state.initial_stack - round_info.bb_amt
                    );
                    assert_eq!(agent_state.already_bet, round_info.bb_amt);

                    assert_eq!(game_state.current_pot, 54);

                    assert_eq!(round_info.min_raise, 8);

                    return Action::Call;
                }
                6 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position),  3usize);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);

                    //already posted big blind
                    assert_eq!(agent_state.stack, agent_state.initial_stack - 8);
                    assert_eq!(agent_state.stack, 17);

                    assert_eq!(agent_state.already_bet, round_info.bb_amt * 4);

                    assert_eq!(game_state.current_pot, 72);

                    return Action::Call;
                }
                7 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(usize::from(agent_state.position),  4usize);

                    assert_eq!(round_info.current_amt_to_call, round_info.bb_amt * 10);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 8);
                    assert_eq!(agent_state.stack, 21);

                    assert_eq!(agent_state.already_bet, round_info.bb_amt * 4);

                    assert_eq!(game_state.current_pot, 84);

                    return Action::Raise(agent_state.initial_stack);
                }
                8 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    assert_eq!(round_info.current_amt_to_call, 29);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 16);
                    assert_eq!(agent_state.stack, 54);

                    assert_eq!(agent_state.already_bet, 16);

                    assert_eq!(game_state.current_pot, 105);

                    return Action::Call;
                }
                9 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    //BB is all in
                    assert_eq!(usize::from(agent_state.position),  2usize);

                    assert_eq!(round_info.current_amt_to_call, 29);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 20);
                    assert_eq!(agent_state.stack, 20);

                    assert_eq!(agent_state.already_bet, 20);

                    assert_eq!(round_info.min_raise, 8);

                    assert_eq!(game_state.current_pot, 118);

                    //min raise 37 but we want it divisible by bb (2)
                    return Action::Raise(38);
                }
                10 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    //BB is all in
                    assert_eq!(usize::from(agent_state.position),  3usize);

                    assert_eq!(round_info.current_amt_to_call, 38);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 20);
                    assert_eq!(agent_state.stack, 5);

                    assert_eq!(agent_state.already_bet, 20);

                    //assert_eq!(round_info.min_raise, 8);

                    assert_eq!(game_state.current_pot, 136);

                    //puts us all in
                    return Action::Call;
                }
                11 => {
                    assert_eq!(round_info.round, Round::Preflop);
                    //BB is all in
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    assert_eq!(round_info.current_amt_to_call, 38);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 29);
                    assert_eq!(agent_state.stack, 41);

                    assert_eq!(agent_state.already_bet, 29);

                    //assert_eq!(round_info.min_raise, 8);

                    assert_eq!(game_state.current_pot, 141);

                    return Action::Call;
                }
                12 => {
                    assert_eq!(round_info.round, Round::Flop);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    assert_eq!(round_info.current_amt_to_call, 0);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 38);
                    assert_eq!(agent_state.stack, 32);

                    assert_eq!(agent_state.already_bet, 0);

                    //assert_eq!(round_info.min_raise, 8);

                    assert_eq!(game_state.current_pot, 150);

                    return Action::Check;
                }

                13 => {
                    assert_eq!(round_info.round, Round::Flop);
                    assert_eq!(usize::from(agent_state.position),  2usize);

                    assert_eq!(round_info.current_amt_to_call, 0);

                    assert_eq!(agent_state.stack, agent_state.initial_stack - 38);
                    assert_eq!(agent_state.stack, 2);

                    assert_eq!(agent_state.already_bet, 0);

                    //assert_eq!(round_info.min_raise, 8);

                    assert_eq!(game_state.current_pot, 150);

                    return Action::Check;
                }
                14 => {
                    assert_eq!(round_info.round, Round::Turn);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    return Action::Check;
                }
                15 => {
                    assert_eq!(round_info.round, Round::Turn);
                    assert_eq!(usize::from(agent_state.position),  2usize);

                    return Action::Raise(2);
                }
                16 => {
                    assert_eq!(round_info.round, Round::Turn);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    return Action::Call;
                }
                17 => {
                    assert_eq!(round_info.round, Round::River);
                    assert_eq!(agent_state.position,  SMALL_BLIND);

                    //No one left in hand?
                    return Action::Check;
                }

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
            agents.push(Box::new(TestPlayerAgent {
                action_counter: &action_counter,
            }));
        }

        let mut game_runner = GameRunner::new(agents, 1);

        game_runner.init_agent_pos_and_stack(SMALL_BLIND, 70);
        game_runner.init_agent_pos_and_stack(BIG_BLIND, 20);
        game_runner.init_agent_pos_and_stack(BIG_BLIND.next(5), 40);
        game_runner.init_agent_pos_and_stack(Position::try_from(3).unwrap(), 25);
        game_runner.init_agent_pos_and_stack(4usize.try_into().unwrap(), 29);

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

        let cards = vec![
            Card::try_from("Ts").unwrap(),
            Card::try_from("Tc").unwrap(),
            //BB cards
            Card::try_from("Ad").unwrap(),
            Card::try_from("Ah").unwrap(),
            Card::try_from("Kd").unwrap(),
            Card::try_from("Kh").unwrap(),
            //HJ cards
            Card::try_from("As").unwrap(),
            Card::try_from("Ac").unwrap(),
            Card::try_from("Ks").unwrap(),
            Card::try_from("Kc").unwrap(),
            Card::try_from("6c").unwrap(),
            Card::try_from("2d").unwrap(),
            Card::try_from("3d").unwrap(),
            Card::try_from("8h").unwrap(),
            Card::try_from("8s").unwrap(),
        ];

        let (winnings, losses) = game_runner.run_game(&cards);

        assert_eq!(winnings[usize::from(BIG_BLIND)], 50u16); //split 100 by 2
        assert_eq!(winnings[3usize as usize], 70); //split 100 by 2, then +5 from remaining players (4)

        //side pot is 4, 4, 4
        assert_eq!(winnings[4usize as usize], 6); //split side pot by 2, 29 stack size - 25 stack size
        assert_eq!(winnings[2usize as usize], 6 + 11 + 11); //side pot + remaining stack from self + loser

        assert_eq!(winnings[usize::from(SMALL_BLIND)], 0);

        assert_eq!(losses[usize::from(BIG_BLIND)], 20);
        assert_eq!(losses[3usize as usize], 25);

        assert_eq!(losses[4usize as usize], 29);
        assert_eq!(losses[2usize as usize], 40);

        assert_eq!(losses[usize::from(SMALL_BLIND)], 40);

        assert_eq!(18, *action_counter.borrow());

        //assert!(false);
    }
}
