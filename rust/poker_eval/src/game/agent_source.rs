use log::info;

use crate::{
    ActionEnum, Card, ChipType, GameState, HoleCards, InitialPlayerState, PlayerState, PokerError,
};

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
        let agent = &self.agents[player_index];
        Ok(agent.get_hole_cards())
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
    use postflop_solver::Range;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        game::{
            agents::{Agent, JustFold, PassiveCallingStation},
            game_runner_source::GameRunnerSourceEnum,
        },
        get_unused_card, init_test_logger, test_game_runner, Card, CardUsedType, GameLog,
        GameRunner, HoleCards, InitialPlayerState, Position,
    };

    use super::AgentSource;

    #[test]
    fn test_agents() {
        init_test_logger();

        let calling_75 = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,93s+,96o+,85s+,87o,75s+";
        let calling_75_range: Range = calling_75.parse().unwrap();

        let mut rng = StdRng::seed_from_u64(42);

        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        agents.push(Box::new(JustFold::default()));
        agents.push(Box::new(PassiveCallingStation::default()));

        for i in 0..3 {
            let mut agent = PassiveCallingStation::default();
            agent.calling_range = Some(calling_75_range.clone());
            agent.name = format!("{} Calling Station 75%", i + 1);
            agents.push(Box::new(agent));
        }

        agents.push(Box::new(PassiveCallingStation::default()));
        agents.push(Box::new(JustFold::default()));

        let mut used_cards = CardUsedType::default();

        let mut players: Vec<InitialPlayerState> = Vec::new();

        for agent_index in 0..agents.len() {
            let agent = &mut agents[agent_index];

            let card1 = get_unused_card(&mut rng, &used_cards).unwrap();
            used_cards.set(card1, true);
            let card2 = get_unused_card(&mut rng, &used_cards).unwrap();
            used_cards.set(card2, true);
            let agent_hole_cards = HoleCards::new(
                Card::try_from(card1).unwrap(),
                Card::try_from(card2).unwrap(),
            )
            .unwrap();
            agent.set_hole_cards(agent_hole_cards);

            let player_name = if agent.get_name().to_string().len() > 0 {
                agent.get_name().to_string()
            } else {
                format!("Agent {}", agent_index)
            };
            let player = InitialPlayerState {
                player_name,
                stack: 1000,
                position: Position::try_from(agent_index).unwrap(),
                cards: Some(agent.get_hole_cards()),
            };
            players.push(player);
        }

        players[1].player_name = "Passive Calling Station 1".to_string();

        let mut board: Vec<Card> = Vec::new();
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
