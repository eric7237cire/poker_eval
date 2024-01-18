use crate::{Card, Deck, HoleCards, game::core::{InitialPlayerState, Position}, };

use super::Agent;

pub fn set_agent_hole_cards(deck: &mut Deck, agents: &mut Vec<Box<dyn Agent>>) {
    for agent_index in 0..agents.len() {
        let agent = &mut agents[agent_index];

        let card1 = deck.get_unused_card().unwrap();

        let card2 = deck.get_unused_card().unwrap();
        let agent_hole_cards = HoleCards::new(
            Card::try_from(card1).unwrap(),
            Card::try_from(card2).unwrap(),
        )
        .unwrap();
        agent.set_hole_cards(agent_hole_cards);
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
