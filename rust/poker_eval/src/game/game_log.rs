//A game log is all the information needed to reconstruct a game.

use std::str::FromStr;

use crate::Action;
use crate::Card;
use crate::HoleCards;

use crate::game::game_log_parser::GameLogParser;
use crate::ChipType;
use crate::PokerError;
use crate::Position;
use log::trace;
use regex::Regex;

pub struct InitialPlayerState {
    stack: ChipType,
    //Player id?

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    position: Position,

    cards: Option<HoleCards>,
}

#[derive(Default)]
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
        let p = GameLogParser::new();
        let (section_name, mut remaining_str) = p.parse_section_name(s)?;

        let mut player_names = Vec::new();
        let mut player_stacks = Vec::new();

        loop {
            let (p_info, new_remaining_str) = p.parse_player_name_stack(remaining_str)?;

            assert!(new_remaining_str.len() < remaining_str.len());

            remaining_str = new_remaining_str;
            //ready for next section
            if p_info.is_none() {
                break;
            }

            let (player_name, stack) = p_info.unwrap();

            trace!("Player name: {} stack {}", player_name, stack);
            player_names.push(player_name);
            player_stacks.push(stack);
        }

        let mut game_log = GameLog::default();

        Ok(game_log)
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
    use std::io::Write;

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{}:{} [{}] - {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.level(),
                    record.args()
                )
            })
            .try_init();
    }
    //https://crates.io/crates/pokerlookup
    //https://github.com/traversgrayson/Poker-Hand-Parser/tree/master/Parser%20and%20Data
    #[test]
    fn test_parse_game_log_from_str() {
        init();

        let hh = "
*** Players ***
Seat 6 - 55 # [button] but can be inferred from the blinds
Seat 2 - 12  # order is to the left
Seat 3 - 147
*** Blinds *** 
Seat 2 - 5
Seat 3 - 10
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
        let game_log: GameLog = hh.parse().unwrap();

        assert_eq!(3, game_log.players.len());
    }
}
