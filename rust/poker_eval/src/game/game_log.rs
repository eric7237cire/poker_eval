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
    pub stack: ChipType,
    pub player_name: String,

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    pub position: Position,

    pub cards: Option<HoleCards>,
}

#[derive(Default)]
struct GameLog {
    //Sb first; then left to right
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

        if section_name != "Players" {
            return Err(PokerError::from_string(format!(
                "Expected section [Players], got [{}]",
                section_name
            )));
        }

        let (players, new_remaining_str) = p.parse_players(remaining_str)?;

        remaining_str = new_remaining_str;

        let (section_name, new_remaining_str) = p.parse_section_name(remaining_str)?;
        remaining_str = new_remaining_str;

        if section_name != "Blinds" {
            return Err(PokerError::from_string(format!(
                "Expected section Blinds, got {}",
                section_name
            )));
        }

        let (sb, bb, new_remaining_str) = p.parse_blinds(&players, remaining_str)?;
        remaining_str = new_remaining_str;

        let mut game_log = GameLog::default();
        game_log.players = players;
        game_log.sb = sb;
        game_log.bb = bb;

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
Seat 2 - 12 # [Small blind first] # order is to the left
Seat 3 - 147
Seat 6 - 55
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
Seat 2 - 12.57
Seat 3 - 148.19
Seat 6 - 55.30


    ";
        let game_log: GameLog = hh.parse().unwrap();

        assert_eq!(3, game_log.players.len());
        assert_eq!(5, game_log.sb);
        assert_eq!(10, game_log.bb);
    }
}
