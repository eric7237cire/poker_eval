use crate::Card;
use crate::InitialPlayerState;
use crate::PlayerAction;
use crate::Position;

use crate::ChipType;
use crate::Round;

pub struct PlayerState {
    pub player_name: String,
    pub stack: ChipType,
    pub folded: bool,

    //Not yet taken from stack
    pub cur_round_putting_in_pot: ChipType,

    //In current betting round, so == remaining stack
    pub all_in_for: Option<ChipType>,

    //Used in all in, to see how much they can win
    pub max_pot: Option<ChipType>,
}

impl PlayerState {
    pub fn new(initial_player_state: &InitialPlayerState) -> Self {
        PlayerState {
            stack: initial_player_state.stack,
            folded: false,
            cur_round_putting_in_pot: 0,
            all_in_for: None,
            max_pot: None,
            player_name: initial_player_state.player_name.clone(),
        }
    }
}

pub struct GameState {
    //sb first order
    pub player_states: Vec<PlayerState>,

    pub current_to_act: Position,
    pub current_round: Round,

    pub prev_round_pot: ChipType,

    //Until current rounds are finished, is not added to pot
    pub round_pot: ChipType,

    pub current_to_call: Option<ChipType>,

    //Initial bet is also considered a raise for this value
    pub min_raise: ChipType,

    pub board: Vec<Card>,

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
