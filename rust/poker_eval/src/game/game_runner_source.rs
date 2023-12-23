use crate::{
    game::game_log_source::GameLogSource, ActionEnum, Card, ChipType, GameState, HoleCards,
    InitialPlayerState, PlayerState, PokerError,
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub enum GameRunnerSourceEnum {
    GameLogSource,
    //LogarithmicKnob,
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
    ) -> Result<ActionEnum, PokerError>;

    //get cards for player?
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError>;

    //get board cards?
    fn get_next_board_card(&mut self) -> Result<Card, PokerError>;

    fn set_final_player_state(
        &mut self,
        player_index: usize,
        player_state: &PlayerState,
        comment: Option<String>,
    ) -> Result<(), PokerError>;
}
