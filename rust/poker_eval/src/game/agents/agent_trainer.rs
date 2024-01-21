//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use log::info;

use crate::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::{
        core::{ActionEnum, CommentedAction, Position, Round, BIG_BLIND},
        runner::{GameRunner, GameRunnerSource},
    },
    Card, HoleCards, PokerError,
};

use super::{info_state_actions, InfoState, InfoStateActionValueType};

pub struct AgentTrainer {}

pub fn run_full_game_tree<T: GameRunnerSource>(
    game_source: &mut T,
    board: Vec<Card>,
    hero_index: usize,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
) -> Result<
    HashMap<InfoState, [InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>,
    PokerError,
> {
    let mut ret: HashMap<InfoState, [InfoStateActionValueType; info_state_actions::NUM_ACTIONS]> =
        HashMap::new();

    let game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )?;

    let mut game_runner_queue = Vec::new();
    game_runner_queue.push(game_runner);

    while !game_runner_queue.is_empty() {
        let mut current_game_runner = game_runner_queue.pop().unwrap();

        //Need to check if this is already done
        if current_game_runner.game_state.player_states[0]
            .final_state
            .is_some()
        {
            process_finished_gamestate(
                current_game_runner,
                hero_index,
                &game_source.get_hole_cards(hero_index)?,
                &mut ret,
                monte_carlo_db.clone(),
            );
            continue;
        }

        //action loop
        for _ in 0..2000 {
            //If we have a choice, need to run with those choices, then actually take the best one to
            //evaluate the value of preceeding actions

            //In a game tree, if our agent being trained has 3 choices, to evaluate the 1st choice
            //We need the value of the best choice

            if current_game_runner
                .get_current_player_state()
                .player_index()
                == hero_index
            {
                //let action_choices = [...]

                //If we have a value already evaluated for those action choices, we take it and
                //move on

                //Otherwise we queue all the possible choices, and also this game state
                //We don't need to run the choice again, we should have its expected value

                //Action number is not perfect since rounds can have varaiable # of actions

                /*
                OO A1
                OO A2
                OO A3

                OO A3 MM A4
                OO A3 MM A5
                OO A3 MM A6

                OO A3 MM A4 NN A7
                OO A3 MM A4 NN A8
                OO A3 MM A4 NN A9

                OO A3 MM A4 NN A9 P [45]
                OO A3 MM A4 NN A8 Q [99]

                give each action an integer, maybe action number will work

                clear every action in gamestate that is not heros

                Then at end, get info states for each action, update equity for that infostate

                InfoState can be derived from PlayerAction
                */

                let hero_helpers = current_game_runner
                    .get_current_player_state()
                    .get_helpers(&current_game_runner.game_state);
                //Are we facing a bet?
                if hero_helpers.call_amount > 0 {
                    //Fold, call, raise
                    let mut fold_game_runner = current_game_runner.clone();
                    let fold_action = CommentedAction {
                        action: ActionEnum::Fold,
                        comment: None,
                    };
                    fold_game_runner.process_next_action(&fold_action).unwrap();
                    game_runner_queue.push(fold_game_runner);

                    let mut call_game_runner = current_game_runner.clone();
                    let call_action = CommentedAction {
                        action: ActionEnum::Call(hero_helpers.call_amount),
                        comment: None,
                    };
                    call_game_runner.process_next_action(&call_action).unwrap();
                    game_runner_queue.push(call_game_runner);

                    if hero_helpers.can_raise {
                        let mut raise_game_runner = current_game_runner.clone();
                        let raise_action = hero_helpers.build_raise_to(
                            &raise_game_runner.game_state,
                            raise_game_runner.game_state.current_to_call * 3,
                            "".to_string(),
                        );

                        raise_game_runner
                            .process_next_action(&raise_action)
                            .unwrap();
                        game_runner_queue.push(raise_game_runner);
                    }

                    //out of action loop
                    break;
                } else {
                    //no bet
                    let mut check_game_runner = current_game_runner.clone();
                    let check_action = CommentedAction {
                        action: ActionEnum::Check,
                        comment: None,
                    };
                    check_game_runner
                        .process_next_action(&check_action)
                        .unwrap();
                    game_runner_queue.push(check_game_runner);

                    if current_game_runner.game_state.current_round == Round::Preflop {
                        //3 bet in bb; 1 is always the big blind player index
                        assert_eq!(hero_index, 1);
                        let mut bet_game_runner = current_game_runner.clone();
                        let bet_action = hero_helpers.build_raise_to(
                            &current_game_runner.game_state,
                            current_game_runner.game_state.current_to_call * 3,
                            "".to_string(),
                        );
                        bet_game_runner.process_next_action(&bet_action).unwrap();
                        game_runner_queue.push(bet_game_runner);
                    } else {
                        //bet half pot
                        let mut bet_game_runner = current_game_runner.clone();
                        let bet_action = hero_helpers
                            .build_bet(bet_game_runner.game_state.pot() / 2, "".to_string());
                        bet_game_runner.process_next_action(&bet_action).unwrap();
                        game_runner_queue.push(bet_game_runner);
                    }

                    break;
                }
            } else {
                let action = game_source
                    .get_action(
                        current_game_runner.get_current_player_state(),
                        &current_game_runner.game_state,
                    )
                    .unwrap();
                let r = current_game_runner.process_next_action(&action).unwrap();

                //We don't keep the action if it's not the hero's
                current_game_runner.game_state.actions.pop().unwrap();

                if r {
                    process_finished_gamestate(
                        current_game_runner,
                        hero_index,
                        &game_source.get_hole_cards(hero_index)?,
                        &mut ret,
                        monte_carlo_db.clone(),
                    );
                    break;
                }
            }
        }
    }

    Ok(ret)
}

fn process_finished_gamestate(
    game_runner: GameRunner,
    hero_index: usize,
    player_hole_cards: &HoleCards,
    info_state_value: &mut HashMap<
        InfoState,
        [InfoStateActionValueType; info_state_actions::NUM_ACTIONS],
    >,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
) {
    info!("Processing finished gamestate");
    assert!(game_runner.game_state.player_states[hero_index]
        .final_state
        .is_some());
    let player_state = &game_runner.game_state.player_states[hero_index];
    let value = (player_state.stack as f64 / game_runner.game_state.bb as f64
        - player_state.initial_stack as f64 / game_runner.game_state.bb as f64)
        as InfoStateActionValueType;

    //assign the max ev for each infostate in the game
    for action in game_runner.game_state.actions.iter() {
        info!(
            "Action {}, info_state_value len {} value {}",
            action,
            info_state_value.len(),
            value
        );
        assert_eq!(hero_index, action.player_index);

        let (info_state, action_id) = InfoState::from(
            &action,
            &game_runner.game_state,
            &player_hole_cards,
            monte_carlo_db.clone(),
        );
        info_state_value
            .entry(info_state)
            .and_modify(|cv| {
                if cv[action_id as usize] < value {
                    cv[action_id as usize] = value;
                }
            })
            .or_insert_with(|| {
                let mut ret = [InfoStateActionValueType::MIN; info_state_actions::NUM_ACTIONS];
                ret[action_id as usize] = value as InfoStateActionValueType;
                ret
            });
    }

    //info!("{}", game_runner.to_game_log().unwrap().to_game_log_string(false, true, 1));
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use log::info;

    use crate::{
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
        game::{
            agents::info_state_actions::BET_HALF,
            core::{
                ActionEnum, ChipType, CommentedAction, GameState, InitialPlayerState, PlayerState,
                Round,
            },
            runner::GameRunnerSource,
        },
        init_test_logger, Board, HoleCards, PokerError,
    };

    use super::run_full_game_tree;

    struct TestGameSource {
        //trained agent is position 1, everyone will fold only if he bets on the river
        //the other 2 players have better cards and will go all in on any bet on preflop/flop/turn
        //The test is the best ev for this setup is call preflop, check check, then bet only the river
        init_player_state: Vec<InitialPlayerState>,
    }

    impl TestGameSource {
        fn new() -> Self {
            let mut ret = Vec::with_capacity(3);
            ret.push(InitialPlayerState {
                stack: 45,
                player_name: "P1".to_string(),
                position: 0.try_into().unwrap(),
                cards: Some("Kh Ks".parse().unwrap()),
            });
            ret.push(InitialPlayerState {
                stack: 35,
                player_name: "Trainee".to_string(),
                position: 1.try_into().unwrap(),
                cards: Some("2h 2s".parse().unwrap()),
            });
            ret.push(InitialPlayerState {
                stack: 55,
                player_name: "P2".to_string(),
                position: 2.try_into().unwrap(),
                cards: Some("Ah As".parse().unwrap()),
            });
            Self {
                init_player_state: ret,
            }
        }
    }

    impl GameRunnerSource for TestGameSource {
        fn get_initial_players(&self) -> &[InitialPlayerState] {
            &self.init_player_state
        }

        fn get_small_blind(&self) -> ChipType {
            3
        }

        fn get_big_blind(&self) -> ChipType {
            19
        }

        fn get_action(
            &mut self,
            player_state: &PlayerState,
            game_state: &GameState,
        ) -> Result<CommentedAction, PokerError> {
            //The agent being trained should not have an action
            assert_ne!(player_state.player_index(), 1);

            let helper = player_state.get_helpers(game_state);

            if game_state.current_round == Round::River && game_state.current_to_call > 0 {
                return Ok(CommentedAction {
                    action: ActionEnum::Fold,
                    comment: None,
                });
            }

            if game_state.current_round == Round::Preflop
                && game_state.current_to_call <= 19
                && game_state.current_to_call > 0
            {
                return Ok(CommentedAction {
                    action: ActionEnum::Call(helper.call_amount),
                    comment: None,
                });
            }

            Ok(if game_state.current_to_call > 0 {
                helper.build_raise_to(game_state, helper.max_can_raise, "".to_string())
            } else {
                CommentedAction {
                    action: ActionEnum::Check,
                    comment: None,
                }
            })
        }

        fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
            Ok(self.init_player_state[player_index].cards.unwrap())
        }
    }

    #[test]
    fn test_run_full_game_tree() {
        init_test_logger();

        // cargo test --lib test_run_full_game_tree -- --nocapture

        let monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
            EvalCacheWithHcReDb::new().unwrap();
        let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

        let mut game_source = TestGameSource::new();
        let board: Board = "3s 4c 5h 7d 8h".parse().unwrap();
        let best_values = run_full_game_tree(
            &mut game_source,
            board.as_slice_card().to_vec(),
            1,
            rcref_mcedb,
        )
        .unwrap();

        for (is_idx, (info_state, value)) in best_values.iter().enumerate() {
            info!("#{}: {} {:?}", is_idx, info_state, value);
        }

        //find info state for betting preflop,flop,turn
        for round in &[Round::Flop, Round::Turn] {
            let mut found = false;
            let round_u8 = (*round) as usize as u8;
            for (info_state, value) in best_values.iter() {
                if info_state.round == round_u8 {
                    found = true;
                    info!("{} {:?}", info_state, value);
                    assert_eq!(value[BET_HALF as usize], -35.0 / 19.0);
                    break;
                }
            }
            assert!(found);
        }

        let mut found = false;
        let round_u8 = Round::River as usize as u8;
        for (info_state, value) in best_values.iter() {
            if info_state.round == round_u8 {
                assert!(!found);
                found = true;
                info!("{} {:?}", info_state, value);

                assert_eq!(value[BET_HALF as usize], 2.0);
            }
        }
        assert!(found);
    }
}
