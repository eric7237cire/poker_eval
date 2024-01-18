

use crate::game::core::{InitialPlayerState, ChipType, PlayerState, CommentedAction, GameState};
use crate::{
    Card,  HoleCards, 
    PokerError,
};

use crate::game::agents::Agent;
use crate::game::runner::GameRunnerSource;

pub struct AgentSource {
    pub agents: Vec<Box<dyn Agent>>,

    //Contains their hole cards
    pub players: Vec<InitialPlayerState>,
    pub sb: ChipType,
    pub bb: ChipType,

    //depending on the game, maybe this is 0, 3, 4, 5 cards
    pub board: Vec<Card>,
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
    
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
        //Agents shouldn't say what cards they have, get it from player data
        self.players[player_index].cards.ok_or_else(|| {
            PokerError::from_string(format!("No hole cards for player {}", player_index))
        })
    }
    
    fn get_next_board_card(&mut self) -> Result<Card, PokerError> {
        if self.board.is_empty() {
            return Err(PokerError::from_string(format!(
                "No more board cards to provide"
            )));
        }
        let card = self.board.pop().unwrap();
        Ok(card)
    }

}

#[cfg(test)]
mod tests {}
