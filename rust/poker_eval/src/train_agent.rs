use std::{cell::RefCell, rc::Rc, time::Instant};

use log::debug;
use poker_eval::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
    board_hc_eval_cache_redb::{
        EvalCacheWithHcReDb, ProduceMonteCarloEval, ProducePartialRankCards,
    },
    game::agents::info_state::{
        info_state_actions, InfoStateDb, InfoStateDbEnum, InfoStateDbTrait,
    },
    game::agents::{
        build_initial_players_from_agents, set_agent_hole_cards, Agent, AgentSource,
        DebugJsonWriter, EqAgent, EqAgentConfig, Tag,
    },
    game::{agents::PanicAgent, core::InitialPlayerState},
    game::{
        agents::{run_full_game_tree, AgentEnum},
        runner::GameRunnerSourceEnum,
    },
    init_logger, Card, Deck,
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

    // Since we are training the agent, this one should be asked for an action
    agents.push(AgentEnum::from(PanicAgent::new("PanicAgent")));

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

// cargo run --release --bin train_agent
pub fn main() {
    let mut last_status_update = Instant::now();

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

    let num_total_iterations = 500;

    let num_players = 9;

    let hero_name = "PanicAgent";

    //Start with clean database
    let info_state_db = InfoStateDbEnum::from(InfoStateDb::new(true).unwrap());
    let rcref_info_state_db = Rc::new(RefCell::new(info_state_db));

    let debug_json_writer = DebugJsonWriter::new();

    for it_num in 0..num_total_iterations {
        if last_status_update.elapsed().as_secs() > 10 {
            last_status_update = Instant::now();
            debug!("Iteration: {} of {}", it_num, num_total_iterations);
        }

        agent_deck.reset();

        let mut agents: Vec<AgentEnum> = build_agents(
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

        let infostate_values = run_full_game_tree(
            &mut game_source,
            board,
            hero_index,
            rcref_mcedb.clone(),
            // &mut Some(&mut debug_json_writer)
            None,
            rcref_info_state_db.clone(),
        )
        .unwrap();

        for (infostate, action) in infostate_values {
            //println!("{} {:?}", infostate, action);
            let mut infostate_weights = rcref_info_state_db
                .borrow()
                .get(&infostate)
                .unwrap()
                .unwrap_or_default();

            // if infostate.num_players == 4
            //     && infostate.hole_card_category == 3
            //     && infostate.equity == 0
            //     && infostate.bet_situation == 1
            //     && infostate.round == 0
            // {
            //     debug!(
            //         "#{} Info state weights: {:?}\nAdding {:?}",
            //         it_num, &infostate_weights, &action
            //     );
            // }
            // for i in 0..infostate_weights.len() {
            //     infostate_weights[i] += action[i].unwrap_or(0.0);
            // }
            // rcref_info_state_db
            //     .borrow_mut()
            //     .put(&infostate, infostate_weights)
            //     .unwrap();
        }
    }

    debug_json_writer.write_overview();
}
