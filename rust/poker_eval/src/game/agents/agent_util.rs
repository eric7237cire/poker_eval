use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{Card, CardUsedType, HoleCards, InitialPlayerState, Position};

use super::Agent;

pub struct AgentDeck {
    rng: StdRng,
    used_cards: CardUsedType, //= CardUsedType::default();
}

const MAX_RAND_NUMBER_ATTEMPS: usize = 1000;

impl AgentDeck {
    pub fn new() -> Self {
        let rng = StdRng::seed_from_u64(42);
        AgentDeck {
            rng,
            used_cards: CardUsedType::default(),
        }
    }

    pub fn reset(&mut self) {
        self.used_cards = CardUsedType::default();
    }

    pub fn set_agent_hole_cards(&mut self, agents: &mut Vec<Box<dyn Agent>>) {
        for agent_index in 0..agents.len() {
            let agent = &mut agents[agent_index];

            let card1 = self.get_unused_card().unwrap();

            let card2 = self.get_unused_card().unwrap();
            let agent_hole_cards = HoleCards::new(
                Card::try_from(card1).unwrap(),
                Card::try_from(card2).unwrap(),
            )
            .unwrap();
            agent.set_hole_cards(agent_hole_cards);
        }
    }

    pub fn get_board(&mut self) -> Vec<Card> {
        let mut board = Vec::new();
        for _ in 0..5 {
            let card = self.get_unused_card().unwrap();
            board.push(Card::try_from(card).unwrap());
        }
        board
    }

    fn get_unused_card(&mut self) -> Option<usize> {
        let mut attempts = 0;
        loop {
            let rand_int: usize = self.rng.gen_range(0..52);
            assert!(rand_int < 52);
            //let card = Card::from(rand_int);
            if !self.used_cards[rand_int] {
                self.used_cards.set(rand_int, true);
                return Some(rand_int);
            }
            attempts += 1;
            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return None;
            }
        }
    }
}

pub fn build_initial_players_from_agents(agents: &[Box<dyn Agent>]) -> Vec<InitialPlayerState> {
    let mut players: Vec<InitialPlayerState> = Vec::new();

    for agent_index in 0..agents.len() {
        let agent = &agents[agent_index];

        let player_name = if agent.get_name().to_string().len() > 0 {
            agent.get_name().to_string()
        } else {
            format!("Agent {}", agent_index)
        };
        let player = InitialPlayerState {
            player_name,
            stack: 500,
            position: Position::try_from(agent_index).unwrap(),
            cards: Some(agent.get_hole_cards()),
        };
        players.push(player);
    }

    players
}
