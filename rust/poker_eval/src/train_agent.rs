use std::{cell::RefCell, collections::HashMap, fs, rc::Rc};

use log::debug;

use poker_eval::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    game::{runner::GameRunnerSourceEnum, core::{CommentedAction, ActionEnum}},
    game::{agents::PanicAgent, core::InitialPlayerState},
    game::{
        agents::{
            build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource, EqAgent,
            EqAgentConfig, Tag,
        },
        runner::{GameRunner, GameRunnerSource},
    },
    init_logger,
    pre_calc::get_repo_root,
    Card, Deck,
};
use rand::seq::SliceRandom;

//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

fn build_agents(
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    monte_carlo_equity_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    num_total_players: usize,
) -> Vec<Box<dyn Agent>> {
    //let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";

    let mut agents: Vec<Box<dyn Agent>> = Vec::new();

    agents.push(Box::new(EqAgent::new(
        "EqAggroA",
        EqAgentConfig::get_aggressive(),
        flop_texture_db.clone(),
        partial_rank_db.clone(),
        monte_carlo_equity_db.clone(),
    )));

    agents.push(Box::new(EqAgent::new(
        "EqAggroB",
        EqAgentConfig::get_aggressive(),
        flop_texture_db.clone(),
        partial_rank_db.clone(),
        monte_carlo_equity_db.clone(),
    )));

    let tag = Tag::new(
        "JJ+,AJs+,AQo+,KQs",
        "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
        "Hero",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    );

    agents.push(Box::new(tag));

    let tag = Tag::new(
        "JJ+,AJs+,AQo+,KQs",
        "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
        "HeroDeux",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    );

    agents.push(Box::new(tag));

    agents.push(Box::new(PanicAgent::new("PanicAgent")));

    let mut i = 0;
    while agents.len() < num_total_players {
        i += 1;
        agents.push(Box::new(EqAgent::new(
            &format!("EqPsvAgent{}", i + 1),
            EqAgentConfig::get_passive(),
            flop_texture_db.clone(),
            partial_rank_db.clone(),
            monte_carlo_equity_db.clone(),
        )));
    }

    agents
}

pub fn main() {
    init_logger();

    //Building what the agents need
    let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
        EvalCacheWithHcReDb::new().unwrap();

    let rcref_pdb = Rc::new(RefCell::new(partial_rank_db));

    let flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

    let rcref_ftdb = Rc::new(RefCell::new(flop_texture_db));

    let monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
        EvalCacheWithHcReDb::new().unwrap();

    let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

    let mut agent_deck = Deck::new();

    //we want to track the worst loses
    //let mut heap: BinaryHeap<(i64, i32, GameLog)> = BinaryHeap::new();

    let num_total_iterations = 1;

    let num_players = 9;

    let repo_root = get_repo_root();

    let json_hh_path = repo_root.join("vue-poker/src/assets/hand_history");

    //let mut json_filenames = Vec::new();

    let hero_name = "PanicAgent";

    for it_num in 0..num_total_iterations {
        agent_deck.reset();

        let mut agents = build_agents(
            rcref_ftdb.clone(),
            rcref_pdb.clone(),
            rcref_mcedb.clone(),
            num_players,
        );
        agents.shuffle(&mut agent_deck.rng);

        let hero_index = agents
            .iter()
            .position(|a| a.get_name() == hero_name)
            .unwrap();
        set_agent_hole_cards(&mut agent_deck, &mut agents);

        let players: Vec<InitialPlayerState> = build_initial_players_from_agents(&agents);

        let board: Vec<Card> = agent_deck.choose_new_board();
        let agent_source = AgentSource {
            agents,
            players,
            sb: 2,
            bb: 5,
        };

        let mut game_source = GameRunnerSourceEnum::from(agent_source);

        let game_runner = GameRunner::new(
            game_source.get_initial_players(),
            game_source.get_small_blind(),
            game_source.get_big_blind(),
            &board,
        )
        .unwrap();
    
        let mut game_runner_queue = Vec::new();
        game_runner_queue.push(game_runner);

        while !game_runner_queue.is_empty() {

            let mut current_game_runner = game_runner_queue.pop().unwrap();

            //Need to check if this is already done
            if current_game_runner.game_state.player_states[0].final_state.is_some() {
                process_finished_gamestate(current_game_runner);
                continue;
            }

            //action loop
            for _ in 0..2000 {
                

                //If we have a choice, need to run with those choices, then actually take the best one to
                //evaluate the value of preceeding actions

                //In a game tree, if our agent being trained has 3 choices, to evaluate the 1st choice
                //We need the value of the best choice

                if current_game_runner.get_current_player_state().player_index() == hero_index {
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
                    if current_game_runner.game_state.current_to_call > 0 {
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
                            comment: None
                        };
                        call_game_runner.process_next_action(&call_action).unwrap();
                        game_runner_queue.push(call_game_runner);

                        if hero_helpers.can_raise {
                            let mut raise_game_runner = current_game_runner.clone();
                            let raise_action = hero_helpers.build_raise_to(&raise_game_runner.game_state, 
                                    raise_game_runner.game_state.current_to_call * 3, "".to_string());
                            
                            raise_game_runner
                                .process_next_action(&raise_action)
                                .unwrap();
                            game_runner_queue.push(raise_game_runner);
                        }

                        //out of action loop
                        break;


                    } else {
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
                        process_finished_gamestate(current_game_runner);
                        break;
                    }
                }
                
            }
        }

        // let _change = game_runner.game_state.player_states[hero_index].stack as i64
        //     - game_runner.game_state.player_states[hero_index].initial_stack as i64;

        // if it_num % 100 == 0 {
        //     debug!(
        //         "Iteration {}",
        //         it_num,
        //     );
        // }

        // let mut game_log = game_runner.to_game_log().unwrap();

        // game_log.calc_best_hands();
        // let json_str = serde_json::to_string_pretty(&game_log).unwrap();
        // let json_filename = format!("{}.json", it_num);
        // let file_path = json_hh_path.join(&json_filename);
        // json_filenames.push(json_filename);
        // fs::write(file_path, json_str).unwrap();
    }

    // let mut overview: HashMap<String, serde_json::Value> = HashMap::new();
    // overview.insert(
    //     "json_filenames".to_string(),
    //     serde_json::to_value(json_filenames).unwrap(),
    // );
    // let overview_filename = json_hh_path.join("overview.json");
    // fs::write(
    //     overview_filename,
    //     serde_json::to_string_pretty(&overview).unwrap(),
    // )
    // .unwrap();
}

fn process_finished_gamestate(game_runner: GameRunner) {

}