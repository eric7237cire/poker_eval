//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Formatter},
    fs,
    rc::Rc,
};

use log::{debug, trace};

use crate::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::{
        agents::info_state::InfoStateDbTrait,
        core::{ActionEnum, CommentedAction, Round},
        runner::{GameRunner, GameRunnerSource, GameRunnerSourceEnum},
    },
    pre_calc::get_repo_root,
    Card, HoleCards, PokerError,
};

use crate::game::agents::info_state::{
    info_state_actions, InfoStateActionValueType, InfoStateDbEnum, InfoStateKey,
};

type UtilityHashMap = HashMap<InfoStateKey, UtilityHashValue>;

#[derive(Debug)]
pub struct UtilityHashValue {
    //action_utils
    pub action_utility: [Option<InfoStateActionValueType>; info_state_actions::NUM_ACTIONS],
    //reach_pr
    pub sum_probability: InfoStateActionValueType,
}

impl Display for UtilityHashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();
        for (action_idx, action_utility) in self.action_utility.iter().enumerate() {
            if let Some(action_utility) = action_utility {
                ret.push_str(&format!("{}: {} ", action_idx, action_utility));
            }
        }
        ret.push_str(format!("sum_prob: {}", self.sum_probability).as_str());
        write!(f, "{}", ret)
    }
}

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

    debug_json_writer: Option<&mut DebugJsonWriter>,
    info_state_db_enum: Rc<RefCell<InfoStateDbEnum>>,
) -> Result<UtilityHashMap, PokerError> {
    let mut ret: UtilityHashMap = HashMap::new();

    let game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )?;

    //debug!("Starting run_full_game_tree");

    let mut process_or_push_args = ProcessOrPushArgs {
        hero_index,
        player_hole_cards: game_source.get_hole_cards(hero_index)?,
        monte_carlo_db: monte_carlo_db.clone(),
        debug_json_writer,
        info_state_db_enum: info_state_db_enum.clone(),
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
                        &mut process_or_push_args,
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
                        &mut process_or_push_args,
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
                            &mut process_or_push_args,
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
                        &mut process_or_push_args,
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
                            &mut process_or_push_args,
                        );
                    } else {
                        //bet half pot

                        let bet_action = hero_helpers
                            .build_bet(current_game_runner.game_state.pot() / 2, "1".to_string());
                        process_or_push(
                            current_game_runner.clone(),
                            bet_action,
                            &mut game_runner_queue,
                            &mut ret,
                            &mut process_or_push_args,
                        );

                        //bet pot
                        let bet_action = hero_helpers
                            .build_bet(current_game_runner.game_state.pot(), "2".to_string());
                        process_or_push(
                            current_game_runner.clone(),
                            bet_action,
                            &mut game_runner_queue,
                            &mut ret,
                            &mut process_or_push_args,
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
                if process_or_push_args.debug_json_writer.is_none() {
                    current_game_runner.game_state.actions.pop().unwrap();
                }

                if r {
                    process_finished_gamestate(
                        current_game_runner,
                        &mut ret,
                        &mut process_or_push_args,
                    );
                    break;
                }
            }
        }
    }

    Ok(ret)
}

struct ProcessOrPushArgs<'a> {
    hero_index: usize,
    player_hole_cards: HoleCards,
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    debug_json_writer: Option<&'a mut DebugJsonWriter>,
    info_state_db_enum: Rc<RefCell<InfoStateDbEnum>>,
}

fn process_or_push(
    mut game_runner: GameRunner,
    action: CommentedAction,
    game_runner_queue: &mut Vec<GameRunner>,
    info_state_value: &mut UtilityHashMap,
    proc_or_push_args: &mut ProcessOrPushArgs,
) {
    let r = game_runner.process_next_action(&action).unwrap();
    if r {
        process_finished_gamestate(game_runner, info_state_value, proc_or_push_args);
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
    // Computing action_utils[act] from kuhn_cfr.py
    action_utils: &mut UtilityHashMap,
    proc_or_push_args: &mut ProcessOrPushArgs,
) {
    trace!("{}\nProcessing finished gamestate", "*".repeat(80));
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

    For the leaf nodes,
    action_utils[act] is the chips/bb won / lost *
    the current strategy of the info set

    For a parent node,
    it's the strategy[action] * total utility of the child node

    This is updated piece meal, so can add

    One problem is arriving at river infosets via different paths
        maybe it's ok?
     */

    let mut current_strategy_probability = 1.0;

    //assign the max ev for each infostate in the game
    for action in game_runner.game_state.actions.iter().rev() {
        trace!(
            "Action {}, info_state_value len {} value {}",
            action,
            action_utils.len(),
            value
        );
        //For debugging we might leave in actions that aren't the hero's
        if action.player_index != hero_index {
            continue;
        }
        assert_eq!(hero_index, action.player_index);

        let (info_state_key, action_id) = InfoStateKey::from_player_action(
            &action,
            &game_runner.game_state,
            &player_hole_cards,
            monte_carlo_db.clone(),
        );

        let prob_played_action = if let Some(iv) = proc_or_push_args
            .info_state_db_enum
            .borrow()
            .get(&info_state_key)
            .unwrap()
        {
            iv.strategy[action_id as usize]
        } else {
            1.0 / info_state_actions::NUM_ACTIONS as InfoStateActionValueType
        };

        current_strategy_probability *= prob_played_action;

        let adjusted_value = current_strategy_probability * value;

        let hv = action_utils
            .entry(info_state_key.clone())
            .or_insert_with(|| UtilityHashValue {
                action_utility: [None; info_state_actions::NUM_ACTIONS],
                sum_probability: 0.0,
            });

        let cv_action = hv.action_utility[action_id as usize].unwrap_or(0.0);

        trace!("Prob played action: {}", prob_played_action);
        trace!("Current strategy prob: {}", current_strategy_probability);
        trace!(
            "Prob sum: {}, now: {}",
            hv.sum_probability,
            hv.sum_probability + current_strategy_probability
        );
        trace!(
            "Value {} * Cur prob {} == Adjusted value: {}",
            value,
            current_strategy_probability,
            adjusted_value
        );
        trace!(
            "Utility Action: {}, now {}",
            cv_action,
            adjusted_value + cv_action
        );

        if let Some(debug_json_writer) = debug_json_writer.as_mut() {
            /*
            InfoState: middle Num Players: 4 Hole Card Cat: 3 < 10% facing raise preflop

            InfoState: middle Num Players: 4 Hole Card Cat: 1 10 - 30% unbet flop
             */
            if info_state_key.position == 1
                && info_state_key.num_players == 4
                && info_state_key.hole_card_category == 1
                && info_state_key.equity == 1
                && info_state_key.bet_situation == 0   
                && info_state_key.round == 1             
            {
                debug!(
                    "Action {}, value {}",
                    action,
                    value
                );
                debug!("Prob played action: {}", prob_played_action);
                debug!("Current strategy prob: {}", current_strategy_probability);
                debug!(
                    "Prob sum: {}, now: {}",
                    hv.sum_probability,
                    hv.sum_probability + current_strategy_probability
                );
                debug!(
                    "Value {} * Cur prob {} == Adjusted value: {}",
                    value,
                    current_strategy_probability,
                    adjusted_value
                );
                debug!(
                    "Utility Action id={} is {}, now {}",
                    action_id,
                    cv_action,
                    adjusted_value + cv_action
                );
                debug_json_writer.write_json(&game_runner);


            }
        }

        hv.action_utility[action_id as usize] = Some(adjusted_value + cv_action);

        hv.sum_probability += current_strategy_probability;
    }

    //info!("{}", game_runner.to_game_log().unwrap().to_game_log_string(false, true, 1));
}


//Recursive version
pub fn run_full_game_tree_recursive(
    game_source: &mut GameRunnerSourceEnum,
    board: Vec<Card>,
    //The agent being trained's index
    hero_index: usize,
    //Used to calculate equity vs random hole cards
    monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    debug_json_writer: Option<&mut DebugJsonWriter>,
    info_state_db_enum: Rc<RefCell<InfoStateDbEnum>>,
) -> Result<InfoStateActionValueType, PokerError> {

    let mut  game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )?;

    //debug!("Starting run_full_game_tree");

    let mut process_or_push_args = ProcessOrPushArgs {
        hero_index,
        player_hole_cards: game_source.get_hole_cards(hero_index)?,
        monte_carlo_db: monte_carlo_db.clone(),
        debug_json_writer,
        info_state_db_enum: info_state_db_enum.clone(),
    };

    run_full_game_tree_recursive_helper(game_source, &mut game_runner, &mut process_or_push_args, 1.0)
}

fn run_full_game_tree_recursive_helper(
    game_source: &mut GameRunnerSourceEnum,
    game_runner: &mut GameRunner,
    proc_or_push_args: &mut ProcessOrPushArgs,
    //Probability agent being trained took the actions to get to this point in game runner
    hero_prob: InfoStateActionValueType,
) -> Result<InfoStateActionValueType, PokerError> {

    //Continue until it is our agent's turn or until the game is over
    for _ in 0..2000 {
        if game_runner
                .get_current_player_state()
                .player_index()
                == proc_or_push_args.hero_index {
            break;
        }

        let action = game_source
                .get_action(
                    game_runner.get_current_player_state(),
                    &game_runner.game_state,
                )
                .unwrap();
        let r = game_runner.process_next_action(&action).unwrap();

        //No need to save history unless we're debugging
        if proc_or_push_args.debug_json_writer.is_none() {
            game_runner.game_state.actions.pop().unwrap();
        }

        if r {
            //game is done, return how much hero won or lost
            let player_state = &game_runner.game_state.player_states[proc_or_push_args.hero_index];
            let value = (player_state.stack as f64 / game_runner.game_state.bb as f64
                - player_state.initial_stack as f64 / game_runner.game_state.bb as f64)
                as InfoStateActionValueType;
            return Ok(value);
        }
    }

    //At this point it's heros turn and the game is not over

    let hero_helpers = game_runner
        .get_current_player_state()
        .get_helpers(&game_runner.game_state);

    let mut action_utils: [Option<InfoStateActionValueType>; info_state_actions::NUM_ACTIONS] = [None; info_state_actions::NUM_ACTIONS];

    let info_state_key = InfoStateKey::from_game_state(
        &game_runner.game_state,
        game_runner.get_current_player_state(),
        &proc_or_push_args.player_hole_cards,
        proc_or_push_args.monte_carlo_db.clone(),
    );

    let mut info_state_value = proc_or_push_args
        .info_state_db_enum
        .borrow()
        .get(&info_state_key)
        .unwrap()
        .unwrap_or_default();

    {
        let prob_played_action = &info_state_value.strategy;

        //Are we facing a bet?
        if hero_helpers.call_amount > 0 {
            //Fold, call, raise

            let fold_action = CommentedAction {
                action: ActionEnum::Fold,
                comment: None,
            };

            let fold_value = play_action(
                game_source,
                game_runner,
                fold_action,
                proc_or_push_args,
                prob_played_action[0] * hero_prob
            )?;

            action_utils[0] = Some(fold_value);

            let call_action = CommentedAction {
                action: ActionEnum::Call(hero_helpers.call_amount),
                comment: None,
            };
            let call_value = play_action(
                game_source,
                game_runner,
                call_action,
                proc_or_push_args,
                prob_played_action[1] * hero_prob
            )?;

            action_utils[1] = Some(call_value);

            if hero_helpers.can_raise {
                let raise_action = hero_helpers.build_raise_to(
                    &game_runner.game_state,
                    game_runner.game_state.current_to_call * 3,
                    "2".to_string(),
                );

                let raise_value = play_action(
                    game_source,
                    game_runner,
                    raise_action,
                    proc_or_push_args,
                    prob_played_action[2] * hero_prob
                )?;

                action_utils[2] = Some(raise_value);
            }
        } else {
            //no bet (more correctly, we alreeday put in call amount, so bb would be here)
            let check_action = CommentedAction {
                action: ActionEnum::Check,
                comment: None,
            };

            let check_value = play_action(
                game_source,
                game_runner,
                check_action,
                proc_or_push_args,
                prob_played_action[0] * hero_prob
            )?;

            action_utils[0] = Some(check_value);

            if game_runner.game_state.current_round == Round::Preflop {
                //3 bet in bb; 1 is always the big blind player index
                assert_eq!(proc_or_push_args.hero_index, 1);
                let bet_action = hero_helpers.build_raise_to(
                    &game_runner.game_state,
                    game_runner.game_state.current_to_call * 3,
                    "1".to_string(),
                );
                
                let bet_value = play_action(
                    game_source,
                    game_runner,
                    bet_action,
                    proc_or_push_args,
                    prob_played_action[1] * hero_prob
                )?;

                action_utils[1] = Some(bet_value);

            } else {
                //bet half pot

                let bet_action = hero_helpers
                    .build_bet(game_runner.game_state.pot() / 2, "1".to_string());
                
                let bet_value = play_action(
                    game_source,
                    game_runner,
                    bet_action,
                    proc_or_push_args,
                    prob_played_action[1] * hero_prob
                )?;

                action_utils[1] = Some(bet_value);

                if hero_helpers.max_can_raise > game_runner.game_state.pot() / 2 {
                    //bet pot
                    let bet_action = hero_helpers
                        .build_bet(game_runner.game_state.pot(), "2".to_string());

                
                    
                    let bet_value = play_action(
                        game_source,
                        game_runner,
                        bet_action,
                        proc_or_push_args,
                        prob_played_action[2] * hero_prob
                    )?;

                    action_utils[2] = Some(bet_value);
                }
            }
        }

    }
    //Here we have gone down each path and have the value of each action (in terms of bb won/lost)

    let util = action_utils.iter().enumerate()
    .map(|(i, au)| au.unwrap_or(0.0) * info_state_value.strategy[i]).sum::<InfoStateActionValueType>();

    
    let mut normalizing_sum = 0.0;
    for action_id in 0..info_state_actions::NUM_ACTIONS {
        if action_utils[action_id].is_none() {
            continue;
        }
        let regret = action_utils[action_id].unwrap() - util;

        //Normally this is multplied by the probability of chance / other players, but
        //as we're using deterministic bots, it's * 1.0 
        info_state_value.regret_sum[action_id] += regret;

        if info_state_value.regret_sum[action_id] < 0.0 {
            info_state_value.regret_sum[action_id] = 0.0;
        }

        normalizing_sum += info_state_value.regret_sum[action_id];
    }

    //Update strategy
    for action_id in 0..info_state_actions::NUM_ACTIONS {
        if normalizing_sum > 0.0 {
            info_state_value.strategy[action_id] = info_state_value.regret_sum[action_id] / normalizing_sum;
        } else {
            //Lots of negative regret can make this happen
            info_state_value.strategy[action_id] = 1.0 / info_state_actions::NUM_ACTIONS as InfoStateActionValueType;
        }

        info_state_value.strategy_sum[action_id] +=
            hero_prob * info_state_value.strategy[action_id];
    }

    //Update fields for average strategy calculation
    info_state_value.reach_pr_sum += hero_prob;

    //Save the updated info state
    proc_or_push_args
        .info_state_db_enum
        .borrow_mut()
        .put(&info_state_key, &info_state_value)
        .unwrap();

    Ok(util)
}

fn play_action(
    game_source: &mut GameRunnerSourceEnum,
    game_runner: &mut GameRunner,
    action: CommentedAction,        
    proc_or_push_args: &mut ProcessOrPushArgs,
    hero_prob: InfoStateActionValueType,
) -> Result<InfoStateActionValueType, PokerError> {

    

    let cloned_game_state = game_runner.game_state.clone();
    let cloned_board = game_runner.board_cards.clone();
    let cloned_used_cards = game_runner.used_cards;
    
    assert_eq!(game_runner
        .get_current_player_state()
        .player_index(), proc_or_push_args.hero_index);

    let r = game_runner.process_next_action(&action).unwrap();

    //No need to save history unless we're debugging
    if proc_or_push_args.debug_json_writer.is_none() {
        game_runner.game_state.actions.pop().unwrap();
    }

    let value = if r {
        let player_state = &game_runner.game_state.player_states[proc_or_push_args.hero_index];
        (player_state.stack as f64 / game_runner.game_state.bb as f64
                - player_state.initial_stack as f64 / game_runner.game_state.bb as f64)
                as InfoStateActionValueType
    } else {        
        run_full_game_tree_recursive_helper(
            game_source,
            game_runner,
            proc_or_push_args,
            hero_prob
        )?
    };

    //Put state back
    game_runner.game_state = cloned_game_state;
    game_runner.board_cards = cloned_board;
    game_runner.used_cards = cloned_used_cards;

    Ok(value)
}


#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use log::info;

    use crate::{
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
        game::{
            agents::info_state::{
                info_state_actions::{self, InfoStateActionValueType},
                InfoStateDbEnum, InfoStateKey, InfoStateMemory,
            },
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

        let info_state_flop: InfoStateKey = InfoStateKey::new(
            3,
            1,
            &game_source.get_hole_cards(1).unwrap(),
            rcref_mcedb.clone(),
            0,
            0,
            &board.as_slice_card()[0..3],
            Round::Flop,
        );

        let info_state_turn: InfoStateKey = InfoStateKey::new(
            3,
            1,
            &game_source.get_hole_cards(1).unwrap(),
            rcref_mcedb.clone(),
            0,
            0,
            &board.as_slice_card()[0..4],
            Round::Turn,
        );

        //info_state_db_enum.put(&info_state_flop, [0.9, 0.05, 0.05]).unwrap();
        //info_state_db_enum.put(&info_state_turn, [0.2, 0.7, 0.1]).unwrap();

        let rcref_is_db = Rc::new(RefCell::new(info_state_db_enum));

        let best_values = run_full_game_tree(
            &mut game_source,
            board.as_slice_card().to_vec(),
            1,
            rcref_mcedb.clone(),
            None,
            rcref_is_db.clone(),
        )
        .unwrap();

        for (is_idx, (info_state, value)) in best_values.iter().enumerate() {
            info!("#{}: {} {:?}", is_idx, info_state, value);
        }

        let info_state_river: InfoStateKey = InfoStateKey::new(
            3,
            1,
            &game_source.get_hole_cards(1).unwrap(),
            rcref_mcedb.clone(),
            0,
            0,
            board.as_slice_card(),
            Round::River,
        );

        assert!(best_values.contains_key(&info_state_river));

        //In this test, the other 2 players, despite having the best hands will fold only to a river bet
        let values = best_values.get(&info_state_river).unwrap().action_utility;
        assert_approx_eq_opt(values[info_state_actions::CHECK as usize], Some(-0.33333));
        assert_approx_eq_opt(
            values[info_state_actions::BET_HALF as usize],
            Some(0.666667),
        );
        //assert_approx_eq_opt(values[info_state_actions::BET_POT as usize], Some(2.0));

        //In the turn, we need to modify the above values by the current weight/probabilities;
        //lets say checking is currently @ 0.2, bet half @ 0.7 bet pot @ .1
        //But question, do we need to discount possibility that we are in an unbet pot to begin with?  Let's not for now...

        //What if those probabilities are 0, those values would get lost, so maybe have the min be at least 5% or 10% or something
        let values = best_values.get(&info_state_turn).unwrap().action_utility;
        assert_approx_eq_opt(
            values[info_state_actions::CHECK as usize],
            Some(1.0 / 3.0 * 1.0 / 3.0),
        );
        assert_approx_eq_opt(
            values[info_state_actions::BET_HALF as usize],
            Some(-1.0 / 3.0 * 35.0 / 19.0),
        );
        //assert_eq!(values[info_state_actions::BET_POT as usize], Some(0.0));

        let values = best_values.get(&info_state_flop).unwrap().action_utility;
        assert_approx_eq_opt(
            values[info_state_actions::CHECK as usize],
            Some(1.0 / 3.0 * 1.0 / 9.0 + 1.0 / 3.0 * -1.0 / 3.0 * 35.0 / 19.0),
        );
        assert_approx_eq_opt(
            values[info_state_actions::BET_HALF as usize],
            Some(-0.614035),
        );
        //assert_approx_eq_opt(values[info_state_actions::BET_POT as usize], Some(0.0));
    }

    fn assert_approx_eq(a: InfoStateActionValueType, b: InfoStateActionValueType) {
        assert!((a - b).abs() < 0.0001, "{} != {}", a, b);
    }
    fn assert_approx_eq_opt(
        a: Option<InfoStateActionValueType>,
        b: Option<InfoStateActionValueType>,
    ) {
        assert_approx_eq(a.unwrap(), b.unwrap());
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

        let _best_values = run_full_game_tree(
            &mut game_source,
            board.as_slice_card().to_vec(),
            1,
            rcref_mcedb,
            None,
            rcref_is_db.clone(),
        )
        .unwrap();
    }
}
