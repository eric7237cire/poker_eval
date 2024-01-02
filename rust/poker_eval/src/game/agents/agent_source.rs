use log::trace;

use crate::{
    Card, ChipType, CommentedAction, GameState, HoleCards, InitialPlayerState, PlayerState,
    PokerError,
};

use crate::game::agents::Agent;
use crate::game::game_runner_source::GameRunnerSource;

pub struct AgentSource {
    pub agents: Vec<Box<dyn Agent>>,
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

    


    
   
}
