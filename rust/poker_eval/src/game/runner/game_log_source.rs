use crate::{
    Card, HoleCards, PokerError,
};
use crate::game::core::{ChipType, CommentedAction, GameState, InitialPlayerState, PlayerState};
use crate::game::runner::{GameRunnerSource, GameLog};

pub struct GameLogSource {
    game_log: GameLog,
    cur_action: usize,
}

impl GameLogSource {
    pub fn new(game_log: GameLog) -> Self {
        GameLogSource {
            game_log,
            cur_action: 0,
        }
    }
}

impl GameRunnerSource for GameLogSource {
    fn get_initial_players(&self) -> &[InitialPlayerState] {
        &self.game_log.players
    }

    fn get_small_blind(&self) -> ChipType {
        self.game_log.sb
    }

    fn get_big_blind(&self) -> ChipType {
        self.game_log.bb
    }

    fn get_action(
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> Result<CommentedAction, PokerError> {
        if self.cur_action >= self.game_log.actions.len() {
            return Err(PokerError::from_string(format!(
                "Invalid action index {}",
                self.cur_action
            )));
        }
        let action = &self.game_log.actions[self.cur_action];

        let player = &self.game_log.players[action.player_index];

        if action.round != game_state.current_round {
            return Err(PokerError::from_string(format!(
                "Round mismatch {} != {}",
                action.round, game_state.current_round
            )));
        }

        if player.player_name != player_state.player_name {
            return Err(PokerError::from_string(format!(
                "Player name mismatch {} != {}",
                player.player_name, player_state.player_name
            )));
        }

        self.cur_action += 1;
        Ok(CommentedAction {
            action: action.action,
            comment: None,
        })
    }

    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
        if player_index >= self.game_log.players.len() {
            return Err(PokerError::from_string(format!(
                "Invalid player index {}",
                player_index
            )));
        }

        self.game_log.players[player_index]
            .cards
            .ok_or(PokerError::from_string(format!(
                "Player {} does not have hole cards",
                player_index
            )))
    }

    

    
}
