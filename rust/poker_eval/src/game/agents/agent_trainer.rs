use std::{
    cell::RefCell,
    collections::{HashMap},
    fs,
    rc::Rc,
};

use log::debug;

use crate::{
    game::agents::{
        build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource, EqAgent,
        EqAgentConfig, Tag,
    },
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    init_logger, Card, Deck,  pre_calc::{get_repo_root}, game::core::InitialPlayerState, game::runner::{GameRunner, GameRunnerSourceEnum},
};
use rand::seq::SliceRandom;

use super::PanicAgent;

//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

pub struct AgentTrainer {
    
}


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

pub fn try_trainer() {
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

    let num_total_iterations = 20_000;
    
    let num_players = 9;
    
    let repo_root = get_repo_root();
    
    let json_hh_path = repo_root.join("vue-poker/src/assets/hand_history");
    
    let mut json_filenames = Vec::new();

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

        let hero_index = agents.iter().position(|a| a.get_name() == hero_name).unwrap();
        set_agent_hole_cards(&mut agent_deck, &mut agents);

        let players: Vec<InitialPlayerState> = build_initial_players_from_agents(&agents);

        let board: Vec<Card> = agent_deck.choose_new_board();
        let agent_source = AgentSource {
            agents,
            players,
            sb: 2,
            bb: 5,
            board,
        };

        let mut game_runner = GameRunner::new(GameRunnerSourceEnum::from(agent_source)).unwrap();

        for _ in 0..2000 {
            let action_count_before = game_runner.game_state.actions.len();

            //If the game runner is on the 'panic agent', we need to provide it a different action based on what is possible
            let r = game_runner.process_next_action().unwrap();
            if r {
                break;
            }
            let action_count_after = game_runner.game_state.actions.len();
            // debug!(
            //     "Last action: {}",
            //     &game_runner.game_state.actions.last().as_ref().unwrap()
            // );
            assert_eq!(action_count_before + 1, action_count_after);
        }

        let _change = game_runner.game_state.player_states[hero_index].stack as i64
            - game_runner.game_state.player_states[hero_index].initial_stack as i64;

        

        if it_num % 100 == 0 {
            debug!(
                "Iteration {}",
                it_num,
            );
        }

        let mut game_log = game_runner.to_game_log().unwrap();

        game_log.calc_best_hands();
        let json_str = serde_json::to_string_pretty(&game_log).unwrap();
        let json_filename = format!("{}.json", it_num);
        let file_path = json_hh_path.join(&json_filename);
        json_filenames.push(json_filename);
        fs::write(file_path, json_str).unwrap();

    }

    

    let mut overview: HashMap<String, serde_json::Value> = HashMap::new();
    overview.insert(
        "json_filenames".to_string(),
        serde_json::to_value(json_filenames).unwrap(),
    );
    let overview_filename = json_hh_path.join("overview.json");
    fs::write(
        overview_filename,
        serde_json::to_string_pretty(&overview).unwrap(),
    )
    .unwrap();

    

}