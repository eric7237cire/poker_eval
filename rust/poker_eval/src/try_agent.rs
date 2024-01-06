use std::{cell::RefCell, collections::BinaryHeap, rc::Rc, path::{Path, PathBuf}, fs};

use log::{debug, info};
use poker_eval::{
    agents::{
        build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource,
        PassiveCallingStation, Tag,
    },
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
    game_runner_source::GameRunnerSourceEnum,
    init_logger, test_game_runner, Card, Deck, GameRunner, InitialPlayerState,
};

fn build_agents(
    flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
    partial_rank_db: Rc<RefCell<EvalCacheWithHcReDb<ProducePartialRankCards>>>,
    hero_position: usize,
) -> Vec<Box<dyn Agent>> {
    let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";

    let mut agents: Vec<Box<dyn Agent>> = Vec::new();

    agents.push(Box::new(PassiveCallingStation::new(
        None,
        "Call 100% A",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    )));
    agents.push(Box::new(PassiveCallingStation::new(
        None,
        "Call 100% B",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    )));

    for i in 0..6 {
        let agent = PassiveCallingStation::new(
            Some(calling_75),
            &format!("{} Cal Stn 75%", i + 1),
            flop_texture_db.clone(),
            partial_rank_db.clone(),
        );
        agents.push(Box::new(agent));
    }

    let tag = Tag::new(
        "JJ+,AJs+,AQo+,KQs",
        "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+",
        "Hero",
        flop_texture_db.clone(),
        partial_rank_db.clone(),
    );
    //agents.push(Box::new(tag));

    
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

    let mut agent_deck = Deck::new();

    let mut hero_winnings: i64 = 0;

    //we want to track the worst loses
    let mut heap: BinaryHeap<(i64, i32, String, String)> = BinaryHeap::new();

    let num_total_iterations = 200;
    let mut hero_position = 0;

    let hh_path = PathBuf::from("/home/eric/git/poker_eval/rust/hand_history");

    //delete tree hh_path
    if hh_path.exists() {
        std::fs::remove_dir_all(&hh_path).unwrap();
    }
    fs::create_dir_all(&hh_path).unwrap();

    for it_num in 0..num_total_iterations {
        agent_deck.reset();

        let mut agents = build_agents(rcref_ftdb.clone(), rcref_pdb.clone(), hero_position);
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

        test_game_runner(&mut game_runner).unwrap();

        let change = game_runner.game_state.player_states[hero_position].stack as i64
            - game_runner.game_state.player_states[hero_position].initial_stack as i64;

        hero_winnings += change;

        heap.push((change, it_num,
             game_runner.to_game_log_string(true, true, hero_position),
            game_runner.to_pokerstars_string()
            ));

        if heap.len() > 5 {
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

    for (i, (change, it_num, log, ps_str)) in heap.into_iter().enumerate() {
        let file_path = hh_path.join(format!("{}.txt", it_num));
        fs::write(file_path, ps_str).unwrap();
        // if it_num == 69 {
        //     continue;
        // }
        if it_num == 38 {
            continue;
        }
        // if it_num == 119 {
        //     continue;
        // }
        
        debug!(
            "Losing hand #{} (iteration {})\nLoss: {}\n{}\n#{}",
            i, it_num, change, log, it_num
        );
    }

    debug!(
        "Hero winnings: {}; per hand {:.1} in {} iterations",
        hero_winnings,
        hero_winnings as f64 / num_total_iterations as f64,
        num_total_iterations
    );
}
