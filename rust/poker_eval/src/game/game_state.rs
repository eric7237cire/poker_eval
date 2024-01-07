use crate::Board;

use crate::InitialPlayerState;
use crate::PlayerAction;
use crate::Position;

use crate::ChipType;
use crate::Round;

pub struct PlayerState {
    pub position: Position,
    pub player_name: String,
    pub initial_stack: ChipType,

    //what has not yet been put in the middle
    pub stack: ChipType,
    pub folded: bool,

    //None means not yet acted this round
    //already deducted from stack
    pub cur_round_putting_in_pot: Option<ChipType>,

    pub total_put_in_pot: ChipType,

    //In current betting round, so == remaining stack
    pub all_in: bool,
    //pub final_eval_comment: Option<String>,
}

impl PlayerState {
    pub fn new(initial_player_state: &InitialPlayerState) -> Self {
        PlayerState {
            position: initial_player_state.position,
            stack: initial_player_state.stack,
            initial_stack: initial_player_state.stack,
            folded: false,
            cur_round_putting_in_pot: None,
            all_in: false,
            player_name: initial_player_state.player_name.clone(),
            total_put_in_pot: 0,
        }
    }

    //Still in the hand, able to act
    pub fn is_active(&self) -> bool {
        !self.folded && !self.all_in
    }

    pub fn player_index(&self) -> usize {
        self.position.into()
    }
}

pub struct GameState {
    //sb first order
    pub player_states: Vec<PlayerState>,

    pub current_to_act: Position,
    pub current_round: Round,

    //Note this 'resets' when there is a bet or raise as we need to go around again
    //pub num_acted_this_round: u8,
    //acted this round == total_active_players - num_left_to_act
    //note that if a player folds or goes all in, they are no longer counted as active
    //the last to act (unless they bet/raise) will have this == 0 when they decide
    pub num_left_to_act: u8,
    //active means not folded and not all in
    pub total_active_players: u8,
    pub total_players_all_in: u8,
    //so number non folded == total_active_players + total_players_all_in
    pub prev_round_pot: ChipType,

    //Until current rounds are finished, is not added to pot
    pub round_pot: ChipType,

    //Amount next player needs to call (total, not diff what they need to put in)
    pub current_to_call: ChipType,

    //Initial bet is also considered a raise for this value
    pub min_raise: ChipType,

    pub board: Board,

    pub sb: ChipType,
    pub bb: ChipType,

    //All actions in the game, posting blinds not considered an action
    pub actions: Vec<PlayerAction>,
}

impl GameState {
    pub fn pot(&self) -> ChipType {
        self.prev_round_pot + self.round_pot
    }
}

pub struct OldGameState {
    //pot from prev. betting rounds
    pub current_pot: ChipType,
}

impl Default for OldGameState {
    fn default() -> Self {
        OldGameState { current_pot: 0 }
    }
}

impl OldGameState {
    pub fn new(_num_players: u8) -> Self {
        OldGameState { current_pot: 0 }
    }
}
