use crate::game::core::{ChipType, CommentedAction, GameState, InitialPlayerState, PlayerState};
use crate::{game::runner::GameLogSource, Card, HoleCards, PokerError};
use enum_dispatch::enum_dispatch;

#[cfg(not(target_arch = "wasm32"))]
use crate::game::agents::AgentSource;

#[enum_dispatch]
pub enum GameRunnerSourceEnum {
    GameLogSource,
    #[cfg(not(target_arch = "wasm32"))]
    AgentSource,
}

#[enum_dispatch(GameRunnerSourceEnum)]
pub trait GameRunnerSource {
    fn get_initial_players(&self) -> &[InitialPlayerState];

    fn get_small_blind(&self) -> ChipType;
    fn get_big_blind(&self) -> ChipType;

    fn get_action(
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> Result<CommentedAction, PokerError>;

    //get cards for player?
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError>;
}

