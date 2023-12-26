use log::trace;

use crate::{Card, ChipType, GameState, HoleCards, InitialPlayerState, PlayerState, PokerError};

use crate::game::agents::Agent;
use crate::game::game_runner_source::GameRunnerSource;

use super::agents::AgentDecision;
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
    ) -> Result<AgentDecision, PokerError> {
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

    use log::info;
    use postflop_solver::Range;

    use crate::{
        game::{
            agents::{
                build_initial_players_from_agents, Agent, AgentDeck, PassiveCallingStation, Tag,
            },
            game_runner_source::GameRunnerSourceEnum,
        },
        game_runner_source::GameRunnerSource,
        init_test_logger, test_game_runner, Card, GameRunner, InitialPlayerState,
    };

    use super::AgentSource;

    fn build_agents() -> Vec<Box<dyn Agent>> {
        let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";
        let calling_75_range: Range = calling_75.parse().unwrap();

        let mut agents: Vec<Box<dyn Agent>> = Vec::new();

        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(PassiveCallingStation::default()));

        for i in 0..2 {
            let mut agent = PassiveCallingStation::default();
            agent.calling_range = Some(calling_75_range.clone());
            agent.name = format!("{} Cal Stn 75%", i + 1);
            agents.push(Box::new(agent));
        }

        let tag = Tag {
            three_bet_range: "JJ+,AJs+,AQo+,KQs".parse().unwrap(),
            pfr_range: "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+"
                .parse()
                .unwrap(),
            name: "Hero".to_string(),
            hole_cards: None,
        };
        agents.push(Box::new(tag));

        agents
    }

    #[test]
    fn test_agents() {
        init_test_logger();

        let mut agent_deck = AgentDeck::new();

        let mut hero_winnings: i64 = 0;

        for it_num in 0..200 {
            agent_deck.reset();

            let mut agents = build_agents();
            agent_deck.set_agent_hole_cards(&mut agents);

            let players: Vec<InitialPlayerState> = build_initial_players_from_agents(&agents);

            let board: Vec<Card> = agent_deck.get_board();
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

            if it_num == 5
            // change < -50 {
            {
                for pi in 0..5 {
                    game_runner.game_state.player_states[pi].player_name = format!(
                        "{} ({})",
                        game_runner.game_state.player_states[pi].player_name,
                        game_runner.game_runner_source.get_hole_cards(pi).unwrap()
                    );
                }
                game_runner.game_state.player_states[4].player_name = format!(
                    "Hero ({})",
                    game_runner.game_runner_source.get_hole_cards(4).unwrap()
                );
                info!(
                    "Losing hand #{}\n{}",
                    it_num,
                    game_runner.to_game_log_string(true)
                );
            }
        }

        assert_eq!(hero_winnings, 5835);
    }
}
