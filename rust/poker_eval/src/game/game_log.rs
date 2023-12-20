//A game log is all the information needed to reconstruct a game.

use crate::Action;
use crate::Card;
use crate::HoleCards;

use crate::ChipType;

pub struct InitialPlayerState {
    stack: ChipType,
    //Player id?

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    position: Position,

    cards: Option<HoleCards>,
}

struct GameLog {
    players: Vec<InitialPlayerState>,
    sb: ChipType,
    bb: ChipType,

    //depending on the game, maybe this is 0, 3, 4, 5 cards
    board: Vec<Card>,

    actions: Vec<Action>,
}

impl FromStr for GameLog {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let player_regex = Regex::new(
            r#"(?x)  # Enable verbose mode
    Seat\ (\d+)               # Capture seat number
    \ -\ (\d+\.\d+)           # Capture stack amount
    (\ \#\ \[button\])?       # Optional button marker
"#,
        )
        .unwrap();
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    //https://crates.io/crates/pokerlookup
    //https://github.com/traversgrayson/Poker-Hand-Parser/tree/master/Parser%20and%20Data
    #[test]
    fn test_parse_game_log_from_str() {
        let hh = "
*** Players ***
Seat 6 - 55.30 # [button] but can be inferred from the blinds
Seat 2 - 12.82  # order is to the left
Seat 3 - 147.69 
*** Blinds *** 
Seat 2 - 0.25
Seat 3 - 0.50
*** Preflop ***
Seat 6 folds   # UTG acts first
Seat 2 calls 0.25 # call is difference needed
Seat 3 checks # BB Acts last preflop
*** Flop ***
2s 7c 8s
Seat 2 checks
Seat 3 bets 0.50
Seat 2 folds
*** Summary ***
Seat 3 wins 0.75 # split pots?
*** Final chip counts *** 
Seat 6 - 55.30
Seat 2 - 12.57
Seat 3 - 148.19
    ";
        let game_log = hh.parse();
    }
}
