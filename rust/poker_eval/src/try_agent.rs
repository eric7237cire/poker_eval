use std::{cell::RefCell, collections::BinaryHeap, rc::Rc};

use log::debug;
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

    for i in 0..2 {
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
    agents.push(Box::new(tag));

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
    let mut heap: BinaryHeap<(i64, i32, String)> = BinaryHeap::new();

    for it_num in 0..200 {
        agent_deck.reset();

        let mut agents = build_agents(rcref_ftdb.clone(), rcref_pdb.clone());
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

        let change = game_runner.game_state.player_states[4].stack as i64
            - game_runner.game_state.player_states[4].initial_stack as i64;

        hero_winnings += change;

        heap.push((change, it_num, game_runner.to_game_log_string(true, true)));

        if heap.len() > 5 {
            heap.pop();
        }

        //if it_num == 5 || it_num == 36
        //if it_num == 35
        //if it_num == 70
        // if it_num == 101
        // {
        //     debug!(
        //         "Losing hand #{}\n{}",
        //         it_num,
        //         game_runner.to_game_log_string(true, true)
        //     );
        //     panic!();
        // }
    }

    for (i, (change, it_num, log)) in heap.into_iter().enumerate() {
        debug!(
            "Losing hand #{} (iteration {})\nLoss: {}\n{}",
            i, it_num, change, log
        );
    }
}
