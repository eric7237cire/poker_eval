

//Preflop goes left of buttonb, bb is last

//Then flop/turn/river order
//sb, bb, .... button

//button, sb, bb

//Any probabilities are handled in the agent, but they always do one thing

//Once we run it, we produce a probability distribution of the hands

#[cfg(test)]
mod tests {

    //use postflop_solver::Hand as Hand;

    //use poker_rs::core::Card as Card;
    //use poker_rs::{core::Deck as PsDeck, arena::HoldemSimulationBuilder, arena::game_state::Round as PsRound};
    //use poker_rs::arena::GameState as PsGameState;

    //use crate::{PassiveCallingStation, GameState, PreFrabRanges, build_range, Agent, Round, Action, AgentState, Position};

    // #[test]
    // fn test_run_game_with_split_pot() {

    //     // Start
    //     game_state.advance_round();
    //     // Preflop
    //     assert_eq!(Position::SmallBlind as usize, game_state.current_round_data().to_act_idx);
    //     game_state.do_bet(5, true).unwrap();

    //     assert_eq!(Position::BigBlind as usize, game_state.current_round_data().to_act_idx);
    //     game_state.do_bet(10, true).unwrap();

    //     //utg calls
    //     assert_eq!(Position::Utg as usize, game_state.current_round_data().to_act_idx);
    //     game_state.do_bet(10, false).unwrap();

    //     //hj raises
    //     assert_eq!(Position::HiJack as usize, game_state.current_round_data().to_act_idx);
    //     game_state.do_bet(20, false).unwrap();

    //     game_state.advance_round();
    //     assert_eq!(game_state.num_active_players(), 5);

    //     deal_community_card("6c", &mut deck, &mut game_state);
    //     deal_community_card("2d", &mut deck, &mut game_state);
    //     deal_community_card("3d", &mut deck, &mut game_state);
    //     // Flop
    //     game_state.do_bet(90, false).unwrap(); // idx 4
    //     game_state.do_bet(90, false).unwrap(); // idx 0
    //     game_state.advance_round();
    //     assert_eq!(game_state.num_active_players(), 5);

    //     deal_community_card("8h", &mut deck, &mut game_state);
    //     // Turn
    //     game_state.do_bet(0, false).unwrap(); // idx 4
    //     game_state.advance_round();
    //     assert_eq!(game_state.num_active_players(), 5);

    //     // River
    //     deal_community_card("8s", &mut deck, &mut game_state);
    //     game_state.do_bet(100, false).unwrap(); // idx 4
    //     game_state.advance_round();
    //     assert_eq!(game_state.num_active_players(), 5);

    //     let mut sim = HoldemSimulationBuilder::default()
    //         .game_state(game_state)
    //         .build()
    //         .unwrap();
    //     sim.run();

    //     assert_eq!(PsRound::Complete, sim.game_state.round);

    //     // assert_eq!(180, sim.game_state.player_winnings[0]);
    //     // assert_eq!(10, sim.game_state.player_winnings[1]);
    //     // assert_eq!(25, sim.game_state.player_winnings[2]);
    //     // assert_eq!(0, sim.game_state.player_winnings[3]);
    //     // assert_eq!(100, sim.game_state.player_winnings[4]);

    //     // assert_eq!(180, sim.game_state.stacks[0]);
    //     // assert_eq!(10, sim.game_state.stacks[1]);
    //     // assert_eq!(25, sim.game_state.stacks[2]);
    //     // assert_eq!(100, sim.game_state.stacks[3]);
    //     // assert_eq!(100, sim.game_state.stacks[4]);
    // }

    // fn deal_hand_card(idx: usize, card_str: &str, deck: &mut PsDeck, game_state: &mut PsGameState) {
    //     let c = Card::try_from(card_str).unwrap();
    //     assert!(deck.remove(&c));
    //     game_state.hands[idx].push(c);
    // }

    // fn deal_community_card(card_str: &str, deck: &mut PsDeck, game_state: &mut PsGameState) {
    //     let c = Card::try_from(card_str).unwrap();
    //     assert!(deck.remove(&c));
    //     for h in &mut game_state.hands {
    //         h.push(c);
    //     }

    //     game_state.board.push(c);
    // }
}
