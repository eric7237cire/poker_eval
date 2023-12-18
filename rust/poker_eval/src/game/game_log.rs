//A game log is all the information needed to reconstruct a game.

use crate::Action;
use crate::Card;
use crate::HoleCards;

use crate::ChipType;

pub struct InitialPlayerState {
    stack: ChipType,
    //Player id?

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    position: usize,

    cards: HoleCards,
}

struct GameLog {
    players: Vec<InitialPlayerState>,
    sb: ChipType,
    bb: ChipType,


    //depending on the game, maybe this is 0, 3, 4, 5 cards
    board: Vec<Card>,

    actions: Vec<Action>,
}

pub struct CurrentPlayerState {
    stack: ChipType,
    //Player id?

    folded: bool,
    all_in: bool,
}

//Now when we play back a game, we can pass the current state to the UI
struct GameState {
    player_states: Vec<CurrentPlayerState>,

    current_to_act: usize,

    pot: ChipType,

    current_to_call: ChipType,
}