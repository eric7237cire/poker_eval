use log::trace;

use crate::{
    Card, ChipType, CommentedAction, GameLog, GameState, HoleCards, InitialPlayerState,
    PlayerState, PokerError,
};

use super::game_runner_source::GameRunnerSource;

pub struct GameLogSource {
    game_log: GameLog,
    cur_action: usize,
    cur_board_card: usize,
}

impl GameLogSource {
    pub fn new(game_log: GameLog) -> Self {
        GameLogSource {
            game_log,
            cur_action: 0,
            cur_board_card: 0,
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

    fn get_next_board_card(&mut self) -> Result<Card, PokerError> {
        if self.cur_board_card >= self.game_log.board.len() {
            return Err(PokerError::from_string(format!(
                "Invalid board card index {}",
                self.cur_board_card
            )));
        }
        let card = self.game_log.board[self.cur_board_card];
        self.cur_board_card += 1;
        Ok(card)
    }

    fn set_final_player_state(
        &mut self,
        player_index: usize,
        player_state: &PlayerState,
        comment: Option<String>,
    ) -> Result<(), PokerError> {
        trace!(
            "set_final_player_state({}) with comment {}",
            player_index,
            comment.unwrap_or("None".to_string())
        );

        if player_index >= self.game_log.players.len() {
            return Err(PokerError::from_string(format!(
                "Invalid player index {}",
                player_index
            )));
        }
        let player = &mut self.game_log.players[player_index];

        if player.player_name != player_state.player_name {
            return Err(PokerError::from_string(format!(
                "Player name mismatch {} != {}",
                player.player_name, player_state.player_name
            )));
        }

        if self.game_log.final_stacks[player_index] != player_state.stack {
            return Err(PokerError::from_string(format!(
                "Player stack mismatch {} != {}",
                self.game_log.final_stacks[player_index], player_state.stack
            )));
        }

        Ok(())
    }
}
