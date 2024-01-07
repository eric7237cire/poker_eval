use std::{cell::RefCell, collections::{BinaryHeap, HashMap}, rc::Rc, path::PathBuf, fs};

use log::{debug};
use num_format::{ToFormattedString, Locale};
use poker_eval::{
    agents::{
        build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource,
        PassiveCallingStation, Tag, EqAgent,
    },
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards, ProduceMonteCarloEval},
    game_runner_source::GameRunnerSourceEnum,
    init_logger, Card, Deck, GameRunner, InitialPlayerState, GameLog,
};

fn build_agents(
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    monte_carlo_equity_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    hero_position: usize,
) -> Vec<Box<dyn Agent>> {
    let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";

    let mut agents: Vec<Box<dyn Agent>> = Vec::new();

    agents.push(Box::new(PassiveCallingStation::new(
        None,
        "CallAllA",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    )));
    agents.push(Box::new(EqAgent::new(
        None,
        "EqAgent1",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
        monte_carlo_equity_db.clone(),
    )));

    for i in 0..5 {
        let agent = PassiveCallingStation::new(
            Some(calling_75),
            &format!("CalStn75_{}", i + 1),
            flop_texture_db.clone(),
            partial_rank_db.clone(),
        );
        agents.push(Box::new(agent));
    }

    agents.push(
        Box::new(EqAgent::new(
            Some("22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+"),
            "EqAgent2",
            flop_texture_db.clone(),
            partial_rank_db.clone(),
            monte_carlo_equity_db.clone(),
        ))
    );

    let tag = Tag::new(
        "JJ+,AJs+,AQo+,KQs",
        "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
        "Hero",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    );
    
    agents.insert(hero_position, Box::new(tag));

    //info!("Built {} agents", agents.len());

    agents
}

fn main() {
    /*
    cargo run --release --bin try_agent
    */
    init_logger();

    let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
        EvalCacheWithHcReDb::new().unwrap();

    let rcref_pdb = Rc::new(RefCell::new(partial_rank_db));

    let flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

    let rcref_ftdb = Rc::new(RefCell::new(flop_texture_db));

    let monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
        EvalCacheWithHcReDb::new().unwrap();

    let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

    let mut agent_deck = Deck::new();

    let mut hero_winnings: i64 = 0;

    //we want to track the worst loses
    let mut heap: BinaryHeap<(i64, i32, GameLog)> = BinaryHeap::new();

    let num_total_iterations = 20;
    let num_worst_hands_to_keep = 5;
    let mut hero_position = 0;

    let hh_path = PathBuf::from("/home/eric/git/poker_eval/rust/hand_history");
    let ps_hh_path = PathBuf::from("/home/eric/git/poker_eval/rust/ps_hand_history");
    let json_hh_path = PathBuf::from("/home/eric/git/poker_eval/vue-poker/src/assets/hand_history");


    //delete tree hh_path
    for path in [hh_path.clone(), ps_hh_path.clone(), json_hh_path.clone()].iter() {
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
        fs::create_dir_all(path).unwrap();
    }
    
    for it_num in 0..num_total_iterations {
        agent_deck.reset();

        let mut agents = build_agents(rcref_ftdb.clone(), rcref_pdb.clone(), rcref_mcedb.clone(), 
        hero_position);
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

        for _ in 0..200 {
            let action_count_before = game_runner.game_state.actions.len();
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
    

        let change = game_runner.game_state.player_states[hero_position].stack as i64
            - game_runner.game_state.player_states[hero_position].initial_stack as i64;

        hero_winnings += change;

        debug!("Iteration {}, hero change {}", it_num, change.to_formatted_string(&Locale::en));
        
        //if we have enough hands and this hand is not worse than the worst hand
        if heap.len() == num_worst_hands_to_keep && change > heap.peek().unwrap().0 {
            continue;            
        }

        heap.push((change, it_num,
            //game_runner.to_game_log_string(true, true, hero_position),
            game_runner.to_game_log().unwrap(),
            //game_runner.to_pokerstars_string()
            ));

        if heap.len() > num_worst_hands_to_keep {
            heap.pop();
        }

        hero_position = (hero_position + 1) % game_runner.game_state.player_states.len();
        //if it_num == 5 || it_num == 36
        //if it_num == 35
        //if it_num == 70
        //if it_num == 101
        // if it_num == 89 {
        //     debug!(
        //         "Losing hand #{}\n{}",
        //         it_num,
        //         game_runner.to_game_log_string(true, true, hero_position)
        //     );
        //     //panic!();
        // }
    }

    let mut json_filenames = Vec::new();

    for (_i, (_change, it_num, game_log)) in heap.into_iter().enumerate() {
        // let file_path = hh_path.join(format!("{}.txt", it_num));
        // fs::write(file_path, &log).unwrap();
        // let file_path = ps_hh_path.join(format!("{}.txt", it_num));
        // fs::write(file_path, ps_str).unwrap();

        //let game_log: GameLog = log.parse().unwrap();
        let json_str = serde_json::to_string_pretty(&game_log).unwrap();
        let json_filename = format!("{}.json", it_num);
        let file_path = json_hh_path.join(&json_filename);
        json_filenames.push(json_filename);
        fs::write(file_path, json_str).unwrap();
        
    }

    let mut overview: HashMap<String, serde_json::Value> = HashMap::new();
    overview.insert("json_filenames".to_string(), serde_json::to_value(json_filenames).unwrap());
    let overview_filename = json_hh_path.join("overview.json");
    fs::write(overview_filename, serde_json::to_string_pretty(&overview).unwrap()).unwrap();

    debug!(
        "Hero winnings: {}; per hand {:.1} in {} iterations",
        hero_winnings,
        hero_winnings as f64 / num_total_iterations as f64,
        num_total_iterations
    );
}
