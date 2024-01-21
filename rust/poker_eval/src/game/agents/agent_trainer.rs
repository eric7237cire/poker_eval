//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

use std::{cell::RefCell, collections::HashMap, rc::Rc, fs};

use log::{trace, debug};

use crate::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::{
        core::{ActionEnum, CommentedAction, Round},
        runner::{GameRunner, GameRunnerSource},
    },
    Card, HoleCards, PokerError, pre_calc::get_repo_root,
};

use super::{info_state_actions, InfoState, InfoStateActionValueType, InfoStateDbEnum};

type ActionHashMap =
    HashMap<InfoState, [Option<InfoStateActionValueType>; info_state_actions::NUM_ACTIONS]>;

pub struct AgentTrainer {}

/*
Returns best value for each infostate

*/
pub fn run_full_game_tree<T: GameRunnerSource>(
    //Contains other agents as well as hole cards
    game_source: &mut T,
    board: Vec<Card>,
    hero_index: usize,
    //Used to calculate equity vs random hole cards
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    
    debug_json_writer: Option<DebugJsonWriter>,
    info_state_db_enum: Rc<RefCell<InfoStateDbEnum>>
) -> Result<ActionHashMap, PokerError> {
    let mut ret: ActionHashMap = HashMap::new();

    let game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )?;

    let mut process_or_push_args = ProcessOrPushArgs {
        hero_index,
        player_hole_cards: game_source.get_hole_cards(hero_index)?,
        monte_carlo_db: monte_carlo_db.clone(),
        debug_json_writer,
        info_state_db_enum: info_state_db_enum.clone()
    };

    let mut game_runner_queue = Vec::new();
    game_runner_queue.push(game_runner);

    while !game_runner_queue.is_empty() {
        let mut current_game_runner = game_runner_queue.pop().unwrap();

        //action loop
        //We go until we get to a decision the agent being trained needs to make
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

                    let fold_action = CommentedAction {
                        action: ActionEnum::Fold,
                        comment: None,
                    };

                    process_or_push(
                        current_game_runner.clone(),
                        fold_action,
                        &mut game_runner_queue,
                        &mut ret,
                        &mut process_or_push_args
                    );

                    let call_action = CommentedAction {
                        action: ActionEnum::Call(hero_helpers.call_amount),
                        comment: None,
                    };
                    process_or_push(
                        current_game_runner.clone(),
                        call_action,
                        &mut game_runner_queue,                        
                        &mut ret,
                        &mut process_or_push_args
                    );

                    if hero_helpers.can_raise {
                        let raise_action = hero_helpers.build_raise_to(
                            &current_game_runner.game_state,
                            current_game_runner.game_state.current_to_call * 3,
                            "".to_string(),
                        );

                        process_or_push(
                            current_game_runner.clone(),
                            raise_action,
                            &mut game_runner_queue,                            
                            &mut ret,
                            &mut process_or_push_args
                        );
                    }
                } else {
                    //no bet
                    let check_action = CommentedAction {
                        action: ActionEnum::Check,
                        comment: None,
                    };
                    process_or_push(
                        current_game_runner.clone(),
                        check_action,
                        &mut game_runner_queue,                        
                        &mut ret,
                        &mut process_or_push_args
                    );

                    if current_game_runner.game_state.current_round == Round::Preflop {
                        //3 bet in bb; 1 is always the big blind player index
                        assert_eq!(hero_index, 1);
                        let bet_action = hero_helpers.build_raise_to(
                            &current_game_runner.game_state,
                            current_game_runner.game_state.current_to_call * 3,
                            "".to_string(),
                        );
                        process_or_push(
                            current_game_runner.clone(),
                            bet_action,
                            &mut game_runner_queue,
                            &mut ret,
                            &mut process_or_push_args
                        );
                    } else {
                        //bet half pot

                        let bet_action = hero_helpers
                            .build_bet(current_game_runner.game_state.pot() / 2, "".to_string());
                        process_or_push(
                            current_game_runner.clone(),
                            bet_action,
                            &mut game_runner_queue,
                            &mut ret,
                            &mut process_or_push_args
                        );
                    }
                }

                //We added all the new game runners from the different choices the agent being trained could make
                //so we process the queue from the top again
                //This breaks the action loop
                break;
            } else {
                let action = game_source
                    .get_action(
                        current_game_runner.get_current_player_state(),
                        &current_game_runner.game_state,
                    )
                    .unwrap();
                let r = current_game_runner.process_next_action(&action).unwrap();

                //We don't keep the action if it's not the hero's
                if process_or_push_args.debug_json_writer.is_some() {
                    current_game_runner.game_state.actions.pop().unwrap();
                }

                if r {
                    process_finished_gamestate(
                        current_game_runner,
                        &mut ret,
                        &mut process_or_push_args
                    );
                    break;
                }
            }
        }
    }

    Ok(ret)
}

struct ProcessOrPushArgs {
    hero_index: usize,
    player_hole_cards: HoleCards,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    debug_json_writer: Option<DebugJsonWriter>,
    info_state_db_enum: Rc<RefCell<InfoStateDbEnum>>
}


fn process_or_push(
    mut game_runner: GameRunner,
    action: CommentedAction,
    game_runner_queue: &mut Vec<GameRunner>,    
    info_state_value: &mut ActionHashMap,
    proc_or_push_args: &mut ProcessOrPushArgs
) {
    let r = game_runner.process_next_action(&action).unwrap();
    if r {
        process_finished_gamestate(
            game_runner,
            info_state_value,
            proc_or_push_args,
        );
    } else {
        game_runner_queue.push(game_runner);
    }
}

pub struct DebugJsonWriter {
    json_filenames: Vec<String>,
    current_num: usize,
}

impl DebugJsonWriter {
    pub fn new() -> Self {
        Self {
            json_filenames: Vec::new(),
            current_num: 0,
        }
    }

    pub fn write_json(&mut self, game_runner: &GameRunner) {
        let repo_root = get_repo_root();
        let json_hh_path = repo_root.join("vue-poker/src/assets/hand_history");

        let mut game_log = game_runner.to_game_log().unwrap();

        game_log.calc_best_hands();
        let json_str = serde_json::to_string_pretty(&game_log).unwrap();
        let json_filename = format!("{}.json", self.current_num);
        let file_path = json_hh_path.join(&json_filename);

        debug!("Writing json to {}", file_path.to_str().unwrap());
        self.json_filenames.push(json_filename);
        fs::write(file_path, json_str).unwrap();

        self.current_num += 1;
    }

    pub fn write_overview(&self) {
        let mut overview: HashMap<String, serde_json::Value> = HashMap::new();
        overview.insert(
            "json_filenames".to_string(),
            serde_json::to_value(&self.json_filenames).unwrap(),
        );
        let repo_root = get_repo_root();
        let json_hh_path = repo_root.join("vue-poker/src/assets/hand_history");
        let overview_filename = json_hh_path.join("overview.json");
        fs::write(
            overview_filename,
            serde_json::to_string_pretty(&overview).unwrap(),
        )
        .unwrap();
    }
}

fn process_finished_gamestate(
    game_runner: GameRunner,
    info_state_value: &mut ActionHashMap,
    proc_or_push_args: &mut ProcessOrPushArgs
) {
    trace!("Processing finished gamestate");
    let hero_index = proc_or_push_args.hero_index;
    let player_hole_cards = proc_or_push_args.player_hole_cards;
    let monte_carlo_db = proc_or_push_args.monte_carlo_db.clone();
    let debug_json_writer = &mut proc_or_push_args.debug_json_writer;

    assert!(game_runner.game_state.player_states[hero_index]
        .final_state
        .is_some());
    let player_state = &game_runner.game_state.player_states[hero_index];
    let value = (player_state.stack as f64 / game_runner.game_state.bb as f64
        - player_state.initial_stack as f64 / game_runner.game_state.bb as f64)
        as InfoStateActionValueType;

        /*
        Actions at the leaves have 'full value' but the parents need to multiply by the 
        normalized weight of taking that action / probability
        
         */

    //assign the max ev for each infostate in the game
    for action in game_runner.game_state.actions.iter() {
        trace!(
            "Action {}, info_state_value len {} value {}",
            action,
            info_state_value.len(),
            value
        );
        //For debugging we might leave in actions that aren't the hero's
        if action.player_index != hero_index {
            continue;
        }
        assert_eq!(hero_index, action.player_index);

        let (info_state, action_id) = InfoState::from_player_action(
            &action,
            &game_runner.game_state,
            &player_hole_cards,
            monte_carlo_db.clone(),
        );

        if let Some(debug_json_writer) = debug_json_writer.as_mut() {
            /*
            InfoState: middle Num Players: 4 Hole Card Cat: 3 < 10% facing raise preflop
             */
            if info_state.position == 1 && info_state.num_players == 4 && 
            info_state.hole_card_category == 3 && info_state.round == 0 && action_id > 0 && value > 5.0 {
                debug_json_writer.write_json(&game_runner);
            }
        }

        info_state_value
            .entry(info_state)
            .and_modify(|cv| {
                let cv_action = cv[action_id as usize];
                if cv_action.is_none() || cv_action.unwrap() < value {
                    cv[action_id as usize] = Some(value);
                }
            })
            .or_insert_with(|| {
                let mut ret = [None; info_state_actions::NUM_ACTIONS];
                ret[action_id as usize] = Some(value as InfoStateActionValueType);
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
            agents::{info_state_actions::{self, BET_HALF}, InfoState, InfoStateMemory, InfoStateDbEnum},
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

        test_number: usize,
    }

    impl TestGameSource {
        fn new(test_number: usize) -> Self {
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
                test_number,
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

            if self.test_number == 1 {
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
            } else if self.test_number == 2 {
                if game_state.current_round == Round::Preflop && game_state.current_to_call == 19 {
                    return Ok(helper.build_raise_to(
                        game_state,
                        game_state.current_to_call * 2,
                        "".to_string(),
                    ));
                } else if game_state.current_to_call > 0 {
                    return Ok(CommentedAction {
                        action: ActionEnum::Call(helper.call_amount),
                        comment: None,
                    });
                } else {
                    return Ok(CommentedAction {
                        action: ActionEnum::Check,
                        comment: None,
                    });
                }
            } else {
                panic!("Invalid test number");
            }
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

        let mut game_source = TestGameSource::new(1);
        let board: Board = "3s 4c 5h 7d 8h".parse().unwrap();



        let info_state_db_enum = InfoStateDbEnum::from(InfoStateMemory::new());

        let rcref_is_db = Rc::new(RefCell::new(info_state_db_enum));


        let best_values = run_full_game_tree(
            &mut game_source,
            board.as_slice_card().to_vec(),
            1,
            rcref_mcedb.clone(),
            None,
            rcref_is_db.clone()
        )
        .unwrap();

        for (is_idx, (info_state, value)) in best_values.iter().enumerate() {
            info!("#{}: {} {:?}", is_idx, info_state, value);
        }

        let info_state_river: InfoState = InfoState::new(
            3, 1, &game_source.get_hole_cards(1).unwrap(),
            rcref_mcedb.clone(), 0, 0, board.as_slice_card(),
            Round::River
        );

        //find info state for betting preflop,flop,turn
        for round in &[Round::Flop, Round::Turn] {
            let mut found = false;
            let round_u8 = (*round) as usize as u8;
            for (info_state, value) in best_values.iter() {
                if info_state.round == round_u8 {
                    found = true;
                    info!("{} {:?}", info_state, value);
                    assert_eq!(value[BET_HALF as usize], Some(-35.0 / 19.0));
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

                assert_eq!(value[info_state_actions::BET_HALF as usize], Some(2.0));
            }
        }
        assert!(found);
    }

    #[test]
    fn test_run_best_move_folding() {
        init_test_logger();

        // cargo test --lib test_run_full_game_tree -- --nocapture

        let monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
            EvalCacheWithHcReDb::new().unwrap();
        let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

        let mut game_source = TestGameSource::new(2);
        let board: Board = "3s 4c 5h 7d 8h".parse().unwrap();

        let info_state_db_enum = InfoStateDbEnum::from(InfoStateMemory::new());

        let rcref_is_db = Rc::new(RefCell::new(info_state_db_enum));

        let best_values = run_full_game_tree(
            &mut game_source,
            board.as_slice_card().to_vec(),
            1,
            rcref_mcedb,
            None,
            rcref_is_db.clone()
        )
        .unwrap();

        for (is_idx, (info_state, value)) in best_values.iter().enumerate() {
            info!("#{}: {} {:?}", is_idx, info_state, value);
        }

        let mut found = false;
        let round_u8 = Round::Preflop as usize as u8;
        for (info_state, value) in best_values.iter() {
            if info_state.round == round_u8 {
                assert!(!found);
                found = true;
                info!("{} {:?}", info_state, value);

                assert_eq!(value[info_state_actions::FOLD as usize], Some(-1.0));
                assert_eq!(value[info_state_actions::CALL as usize], Some(-35.0 / 19.0));
            }
        }
        assert!(found);
    }
}
