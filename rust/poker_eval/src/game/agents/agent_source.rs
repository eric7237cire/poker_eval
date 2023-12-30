use log::trace;

use crate::{
    Card, ChipType, CommentedAction, GameState, HoleCards, InitialPlayerState, PlayerState,
    PokerError,
};

use crate::game::agents::Agent;
use crate::game::game_runner_source::GameRunnerSource;

pub struct AgentSource {
    agents: Vec<Box<dyn Agent>>,
    pub players: Vec<InitialPlayerState>,
    pub sb: ChipType,
    pub bb: ChipType,

    //depending on the game, maybe this is 0, 3, 4, 5 cards
    pub board: Vec<Card>,
    //store results
    //pub final_stacks: Vec<ChipType>,
}

impl GameRunnerSource for AgentSource {
    fn get_initial_players(&self) -> &[InitialPlayerState] {
        &self.players
    }

    fn get_small_blind(&self) -> ChipType {
        self.sb
    }

    fn get_big_blind(&self) -> ChipType {
        self.bb
    }

    fn get_action(
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> Result<CommentedAction, PokerError> {
        let player_index: usize = player_state.position.into();
        let agent = &mut self.agents[player_index];
        Ok(agent.decide(player_state, game_state))
    }

    //get cards for player?
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
        //Agents shouldn't say what cards they have, get it from player data
        self.players[player_index].cards.ok_or_else(|| {
            PokerError::from_string(format!("No hole cards for player {}", player_index))
        })
    }

    //get board cards?
    fn get_next_board_card(&mut self) -> Result<Card, PokerError> {
        if self.board.is_empty() {
            return Err(PokerError::from_string(format!(
                "No more board cards to provide"
            )));
        }
        let card = self.board.remove(0);
        Ok(card)
    }

    fn set_final_player_state(
        &mut self,
        player_index: usize,
        player_state: &PlayerState,
        comment: Option<String>,
    ) -> Result<(), PokerError> {
        trace!(
            "set_final_player_state: {} stack {} comment {}",
            player_index,
            player_state.stack,
            comment.unwrap_or("".to_string())
        );
        //let agent = &mut self.agents[player_index];
        //agent.set_final_player_state(player_state, comment)
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, collections::BinaryHeap, rc::Rc};

    use log::info;

    use crate::{
        board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
        game::{
            agents::{
                build_initial_players_from_agents, set_agent_hole_cards, Agent,
                PassiveCallingStation, Tag,
            },
            game_runner_source::GameRunnerSourceEnum,
        },
        init_test_logger, test_game_runner, Card, Deck, GameRunner, InitialPlayerState,
        PartialRankContainer,
    };

    use super::AgentSource;

    fn build_agents(
        flop_texture_db: Rc<RefCell<EvalCacheReDb<ProduceFlopTexture>>>,
        partial_rank_db: Rc<
            RefCell<EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>,
        >,
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

    //#[test]
    #[allow(dead_code)]
    fn test_agents() {
        init_test_logger();

        /*
        cargo test  agent --lib --release -- --nocapture --test-threads=1
        */

        let partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards, _> =
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

            let mut game_runner =
                GameRunner::new(GameRunnerSourceEnum::from(agent_source)).unwrap();

            test_game_runner(&mut game_runner).unwrap();

            let change = game_runner.game_state.player_states[4].stack as i64
                - game_runner.game_state.player_states[4].initial_stack as i64;

            hero_winnings += change;

            heap.push((change, it_num, game_runner.to_game_log_string(true, true)));

            if heap.len() > 5 {
                heap.pop();
            }

            if it_num == 5 || it_num == 36
            // change < -50 {
            {
                info!(
                    "Losing hand #{}\n{}",
                    it_num,
                    game_runner.to_game_log_string(true, true)
                );
            }
        }

        // for (i, (change, it_num, log)) in heap.into_iter().enumerate() {
        //     debug!("Losing hand #{} (iteration {})\nLoss: {}\n{}", i, it_num, change, log);
        // }

        //assert_eq!(hero_winnings, 5835);
    }
}
