use std::{cell::RefCell, collections::HashMap, fs, rc::Rc};

use log::debug;
use num_format::{Locale, ToFormattedString};
use poker_eval::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    game::runner::{GameRunner, GameRunnerSourceEnum},
    game::{
        agents::{
            build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource, EqAgent,
            EqAgentConfig, Tag,
        },
        runner::GameRunnerSource,
        agents::info_state::InfoStateDb
    },
    game::{
        agents::{AgentEnum, InfoStateAgent, },
        core::InitialPlayerState,
    },
    init_logger,
    pre_calc::{get_repo_root, perfect_hash::load_boomperfect_hash},
    Card, Deck,
};
use rand::seq::SliceRandom;

fn build_agents(
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    monte_carlo_equity_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    info_state_db: Rc<RefCell<InfoStateDb>>,
    num_total_players: usize,
) -> Vec<AgentEnum> {
    //let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";

    let mut agents: Vec<AgentEnum> = Vec::new();

    agents.push(AgentEnum::from(EqAgent::new(
        "EqAggroA",
        EqAgentConfig::get_aggressive(),
        flop_texture_db.clone(),
        partial_rank_db.clone(),
        monte_carlo_equity_db.clone(),
    )));

    agents.push(AgentEnum::from(EqAgent::new(
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

    agents.push(AgentEnum::from(tag));

    let tag = Tag::new(
        "JJ+,AJs+,AQo+,KQs",
        "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
        "HeroDeux",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    );

    agents.push(AgentEnum::from(tag));

    let info_state_agent = InfoStateAgent::new(
        "InfoStateAgent",
        monte_carlo_equity_db.clone(),
        info_state_db.clone(),
    );

    agents.push(AgentEnum::from(info_state_agent));

    let mut i = 0;
    while agents.len() < num_total_players {
        i += 1;
        agents.push(AgentEnum::from(EqAgent::new(
            &format!("EqPsvAgent{}", i + 1),
            EqAgentConfig::get_passive(),
            flop_texture_db.clone(),
            partial_rank_db.clone(),
            monte_carlo_equity_db.clone(),
        )));
    }

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

    let info_state_db = InfoStateDb::new(false).unwrap();

    let rcref_is_db = Rc::new(RefCell::new(info_state_db));

    let hash_func = load_boomperfect_hash();

    let mut agent_deck = Deck::new();

    //we want to track the worst loses
    //let mut heap: BinaryHeap<(i64, i32, GameLog)> = BinaryHeap::new();

    let num_total_iterations = 10_000;
    let _num_worst_hands_to_keep = 5;
    let num_players = 9;
    let hero_name = "InfoStateAgent";
    let mut winnings: HashMap<String, i64> = HashMap::new();

    let repo_root = get_repo_root();
    //let hh_path = repo_root.join("rust/hand_history");
    //let ps_hh_path = repo_root.join("rust/ps_hand_history");
    let json_hh_path = repo_root.join("vue-poker/src/assets/hand_history");
    let csv_path = repo_root.join("python/hand_history.csv");

    //delete tree hh_path
    // for path in [hh_path.clone(), ps_hh_path.clone(), json_hh_path.clone()].iter() {
    //     if path.exists() {
    //         std::fs::remove_dir_all(path).unwrap();
    //     }
    //     fs::create_dir_all(path).unwrap();
    // }

    let mut wtr = csv::Writer::from_path(csv_path).unwrap();
    let mut json_filenames = Vec::new();

    for it_num in 0..num_total_iterations {
        agent_deck.reset();

        let mut agents = build_agents(
            rcref_ftdb.clone(),
            rcref_pdb.clone(),
            rcref_mcedb.clone(),
            rcref_is_db.clone(),
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

        let mut game_runner = GameRunner::new(
            game_source.get_initial_players(),
            game_source.get_small_blind(),
            game_source.get_big_blind(),
            &board,
        )
        .unwrap();

        for _ in 0..2000 {
            let action_count_before = game_runner.game_state.actions.len();
            let action = game_source
                .get_action(
                    game_runner.get_current_player_state(),
                    &game_runner.game_state,
                )
                .unwrap();
            let r = game_runner.process_next_action(&action).unwrap();
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

        for p in game_runner.game_state.player_states.iter() {
            let winnings = winnings.entry(p.player_name.clone()).or_insert(0);
            *winnings += p.stack as i64 - p.initial_stack as i64;
        }

        if it_num % 100 == 0 {
            debug!("Iteration {}", it_num,);
        }

        let mut game_log = game_runner.to_game_log().unwrap();

        game_log.calc_best_hands();
        let json_str = serde_json::to_string_pretty(&game_log).unwrap();
        let json_filename = format!("{}.json", it_num);
        let file_path = json_hh_path.join(&json_filename);
        json_filenames.push(json_filename);
        fs::write(file_path, json_str).unwrap();

        // if it_num == 107 {
        //     break;
        // }

        let game_csv_line = game_log
            .get_csv_line(hero_index, it_num, rcref_mcedb.clone(), &hash_func)
            .unwrap();
        wtr.serialize(game_csv_line).unwrap();
        // for (c, it, _log) in heap.iter() {
        //     debug!(
        //         "In heap at iteration {}, have {}, {}",
        //         it_num,
        //         c,
        //         it,
        //     );
        // }

        //if we have enough hands and this hand is not worse than the worst hand
        // if heap.len() == num_worst_hands_to_keep && change > heap.peek().unwrap().0 {
        //     continue;
        // }

        // heap.push((
        //     change,
        //     it_num,
        //     //game_runner.to_game_log_string(true, true, hero_position),
        //     game_log,
        //     //game_runner.to_pokerstars_string()
        // ));

        // if heap.len() > num_worst_hands_to_keep {
        //     heap.pop();
        // }

        // if it_num >= 79 {

        //     assert!(heap.iter().any(|(_c, it, _log)| *it==79));
        // }

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

    // for (_i, (_change, it_num, mut game_log)) in heap.into_iter().enumerate() {
    //     // let file_path = hh_path.join(format!("{}.txt", it_num));
    //     // fs::write(file_path, &log).unwrap();
    //     // let file_path = ps_hh_path.join(format!("{}.txt", it_num));
    //     // fs::write(file_path, ps_str).unwrap();
    //     game_log.calc_best_hands();
    //     let json_str = serde_json::to_string_pretty(&game_log).unwrap();
    //     let json_filename = format!("{}.json", it_num);
    //     let file_path = json_hh_path.join(&json_filename);
    //     json_filenames.push(json_filename);
    //     fs::write(file_path, json_str).unwrap();
    // }

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

    for (name, winnings) in winnings.iter() {
        debug!(
            "{} winnings: {}; per hand {:.1} in {} iterations",
            name,
            winnings.to_formatted_string(&Locale::en),
            *winnings as f64 / num_total_iterations as f64,
            num_total_iterations.to_formatted_string(&Locale::en)
        );
    }
}
