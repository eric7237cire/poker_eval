//A game log is all the information needed to reconstruct a game.

use std::str::FromStr;

use crate::Card;
use crate::HoleCards;

use crate::game::game_log_parser::GameLogParser;
use crate::ChipType;
use crate::PlayerAction;
use crate::PokerError;
use crate::Position;
use crate::Round;

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

    actions: Vec<PlayerAction>,

    final_stacks: Vec<ChipType>,
}

impl FromStr for GameLog {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = GameLogParser::new();
        let mut remaining_str = s;

        let players= p.parse_players(&mut remaining_str)?;

        let (sb, bb) = p.parse_blinds(&players, &mut remaining_str)?;
        

        let _section_name =
            p.parse_section_name(&mut remaining_str, Some("Preflop"))?;
        

        let preflop_actions =
            p.parse_round_actions(&players, Round::Preflop, &mut remaining_str)?;

        if preflop_actions.len() < players.len() {
            return Err(PokerError::from_string(format!(
                "Expected at least {} preflop actions, got {} in   {:.100}",
                players.len(),
                preflop_actions.len(),
                &remaining_str
            )));
        }

        let mut actions = preflop_actions;

        let mut board_cards = Vec::new();

        //The remaining rounds have the same structure
        // section name, then cards, then actions
        for round in [Round::Flop, Round::Turn, Round::River].iter() {
            let section_name = p.parse_section_name(&mut remaining_str, None)?;            

            if section_name == "Summary" {
                break;
            }

            if section_name != round.to_string() {
                return Err(PokerError::from_string(format!(
                    "Expected section [{}], got [{}]",
                    round.to_string(),
                    section_name
                )));
            }

            let cards = p.parse_cards(&mut remaining_str)?;

            if *round == Round::Flop {
                if cards.len() != 3 {
                    return Err(PokerError::from_string(format!(
                        "Expected 3 cards, got {}",
                        cards.len()
                    )));
                }
            }

            board_cards.extend(cards);

            let round_actions =
                p.parse_round_actions(&players, *round, &mut remaining_str)?;

            actions.extend(round_actions);
        }

        let final_stacks = p.parse_summary(&mut remaining_str, &players)?;

        if final_stacks.len() != players.len() {
            return Err(PokerError::from_string(format!(
                "Expected {} final player stacks, got {} in {:.100}",
                players.len(),
                final_stacks.len(),
                &remaining_str
            )));
        }

        let mut game_log = GameLog::default();
        game_log.players = players;
        game_log.sb = sb;
        game_log.bb = bb;
        game_log.actions = actions;
        game_log.board = board_cards;
        game_log.final_stacks = final_stacks;

        Ok(game_log)
    }
}

#[allow(dead_code)]
pub struct CurrentPlayerState {
    stack: ChipType,
    //Player id?
    folded: bool,
    all_in: bool,
}

//Now when we play back a game, we can pass the current state to the UI
#[allow(dead_code)]
struct GameState {
    player_states: Vec<CurrentPlayerState>,

    current_to_act: usize,

    pot: ChipType,

    current_to_call: ChipType,
}

#[cfg(test)]
mod tests {
    use crate::{ActionEnum, CardVec};

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
Seat 2 calls  # call is difference needed, can put call amount in comments
Seat 3 checks # BB Acts last preflop
*** Flop ***
2s 7c 8s
Seat 2 checks
Seat 3 bets 5
Seat 2 folds
*** Summary ***
Seat 2 - 12.57  # This section is just to verify
Seat 3 - 148.19
Seat 6 - 55.30


    ";
        let game_log: GameLog = hh.parse().unwrap();

        assert_eq!(3, game_log.players.len());
        assert_eq!(5, game_log.sb);
        assert_eq!(10, game_log.bb);

        assert_eq!(
            3,
            game_log
                .actions
                .iter()
                .filter(|a| a.round == Round::Preflop)
                .count()
        );

        assert_eq!(
            game_log.actions[0],
            PlayerAction {
                player_index: 2,
                action: ActionEnum::Fold,
                round: Round::Preflop,
            }
        );
        assert_eq!(
            game_log.actions[1],
            PlayerAction {
                player_index: 0,
                action: ActionEnum::Call,
                round: Round::Preflop,
            }
        );
        assert_eq!(
            game_log.actions[2],
            PlayerAction {
                player_index: 1,
                action: ActionEnum::Check,
                round: Round::Preflop,
            }
        );

        assert_eq!(3, game_log.board.len());
        assert_eq!(Card::try_from("2s").unwrap(), game_log.board[0]);
        assert_eq!(Card::try_from("7c").unwrap(), game_log.board[1]);
        assert_eq!(Card::try_from("8s").unwrap(), game_log.board[2]);

        assert_eq!(
            game_log.actions[3],
            PlayerAction {
                player_index: 0,
                action: ActionEnum::Check,
                round: Round::Flop,
            }
        );

        assert_eq!(
            game_log.actions[4],
            PlayerAction {
                player_index: 1,
                action: ActionEnum::Bet(5),
                round: Round::Flop,
            }
        );

        assert_eq!(
            game_log.actions[5],
            PlayerAction {
                player_index: 0,
                action: ActionEnum::Fold,
                round: Round::Flop,
            }
        );
    }

    #[test]
    fn test_parse_with_hole_cards() {
        init();

        let hh = "
*** Players *** 
Plyr A - 12 - As Kh
Plyr B - 147 - 2d 2c
Plyr C - 55 - 7d 2h
Plyr D - 55 - Ks Kh
*** Blinds *** 
Plyr A - 5
Plyr B - 10
*** Preflop ***
Plyr C calls    # UTG acts first
Plyr D raises 10
Plyr A calls 
Plyr B raises 20 # so puts in an additional 15
Plyr C folds
Plyr D calls
Plyr A calls
*** Flop ***
2s 7c 8s
Plyr A checks
Plyr B bets 5
Plyr D calls
Plyr A calls
*** Turn ***
2h 
Plyr A checks
Plyr B bets 5
Plyr D folds
Plyr A calls
*** River ***
2d
Plyr A bets 15
Plyr B raises 30 # minimum raise
Plyr A raises 45
Plyr B calls
*** Summary ***
Plyr A - 12 # though this is not valid, the parsing just wants correct syntax
Plyr B - 148 # Plyr B loses 100 with 2h As Kh 2d 7c
Plyr C - 55 # can put in comments showdown, wins / losses side pot / etc
Plyr D - 90
    ";
        let game_log: GameLog = hh.parse().unwrap();

        assert_eq!(4, game_log.players.len());
        assert_eq!(5, game_log.sb);
        assert_eq!(10, game_log.bb);

        assert_eq!(game_log.players[0].cards.unwrap(), 
        "As Kh".parse::<HoleCards>().unwrap());
        assert_eq!(game_log.players[1].cards.unwrap(),
        "2d 2c".parse::<HoleCards>().unwrap());
        assert_eq!(game_log.players[2].cards.unwrap(),
        "7d 2h".parse::<HoleCards>().unwrap());
        assert_eq!(game_log.players[3].cards.unwrap(),
        "Ks Kh".parse::<HoleCards>().unwrap());

        assert_eq!(game_log.board.len(), 5);
        assert_eq!(game_log.board[0], "2s".parse::<Card>().unwrap());
        assert_eq!(game_log.board[1], "7c".parse::<Card>().unwrap());
        assert_eq!(game_log.board[2], "8s".parse::<Card>().unwrap());
        assert_eq!(game_log.board[3], "2h".parse::<Card>().unwrap());
        assert_eq!(game_log.board[4], "2d".parse::<Card>().unwrap());

        assert_eq!(game_log.final_stacks.len(), 4);
        assert_eq!(game_log.final_stacks[0], 12);
        assert_eq!(game_log.final_stacks[1], 148);
        assert_eq!(game_log.final_stacks[2], 55);
        assert_eq!(game_log.final_stacks[3], 90);




        
    }
}
