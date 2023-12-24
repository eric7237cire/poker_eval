use log::info;

use crate::{
    ActionEnum, Card, ChipType, GameState, HoleCards, InitialPlayerState, PlayerState, PokerError,
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
    ) -> Result<ActionEnum, PokerError> {
        let player_index: usize = player_state.position.into();
        let agent = &mut self.agents[player_index];
        Ok(agent.decide(player_state, game_state))
    }

    //get cards for player?
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
        let agent = &self.agents[player_index];
        Ok(agent.get_hole_cards())
    }

    //get board cards?
    fn get_next_board_card(&mut self) -> Result<Card, PokerError> {
        if self.board.len() >= 5 {
            return Err(PokerError::from_string(format!(
                "Too many board cards {}",
                self.board.len()
            )));
        }
        let card = self.board[self.board.len()];
        self.board.push(card);
        Ok(card)
    }

    fn set_final_player_state(
        &mut self,
        player_index: usize,
        player_state: &PlayerState,
        comment: Option<String>,
    ) -> Result<(), PokerError> {
        info!(
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
    use log::{debug, info};
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{init_test_logger, game::{agents::{Agent, PassiveCallingStation, JustFold}, game_runner_source::GameRunnerSourceEnum}, InitialPlayerState, Position, CardUsedType, get_unused_card, HoleCards, Card, GameRunner, GameLog, test_game_runner};

    use super::AgentSource;


    #[test]
    fn test_agents() {
        init_test_logger();

        let mut rng = StdRng::seed_from_u64(42);
        
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        agents.push(Box::new(JustFold::default()));
        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(JustFold::default()));

        let mut used_cards = CardUsedType::default();

        let mut players : Vec<InitialPlayerState> = Vec::new();

        for agent_index in 0..agents.len() {
            let agent = &mut agents[agent_index];

            let card1 = get_unused_card(&mut rng, &used_cards).unwrap();
            used_cards.set(card1, true);
            let card2 = get_unused_card(&mut rng, &used_cards).unwrap();
            used_cards.set(card2, true);
            let agent_hole_cards = HoleCards::new(
                Card::try_from(card1).unwrap(), Card::try_from(card2).unwrap()).unwrap();
            agent.set_hole_cards(agent_hole_cards);

            let player_name = format!("Agent {}", agent_index);
            let player = InitialPlayerState {
                player_name,
                stack: 1000,
                position: Position::try_from(agent_index).unwrap(),
                cards: Some(agent.get_hole_cards()),
            };
            players.push(player);
        };

        players[1].player_name = "Passive Calling Station 1".to_string();

        let mut board : Vec<Card> = Vec::new();
        for _ in 0..5 {
            let card = get_unused_card(&mut rng, &used_cards).unwrap();
            used_cards.set(card, true);
            board.push(Card::try_from(card).unwrap());
        }

        let agent_source = AgentSource {
            agents,
            players,
            sb: 5,
            bb: 10,
            board,
        };

        let mut game_runner = GameRunner::new(GameRunnerSourceEnum::from(agent_source)).unwrap();

        test_game_runner(&mut game_runner).unwrap();

        assert!(false);
    }
}