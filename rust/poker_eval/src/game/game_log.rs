//A game log is all the information needed to reconstruct a game.

use std::cmp::min;
use std::str::FromStr;

use itertools::Itertools;
use log::trace;
use serde::Serialize;

use crate::Card;
use crate::HoleCards;

use crate::game::game_log_parser::GameLogParser;
use crate::ChipType;
use crate::PlayerAction;
use crate::PokerError;
use crate::Position;
use crate::Round;
use crate::rank_cards;

#[derive(Serialize)]
pub struct InitialPlayerState {
    pub stack: ChipType,
    pub player_name: String,

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    pub position: Position,

    pub cards: Option<HoleCards>,
}

#[derive(Default, Serialize)]
pub struct GameLog {
    //Sb first; then left to right
    pub players: Vec<InitialPlayerState>,
    pub sb: ChipType,
    pub bb: ChipType,

    //depending on the game, maybe this is 0, 3, 4, 5 cards
    pub board: Vec<Card>,

    pub actions: Vec<PlayerAction>,

    pub final_stacks: Vec<ChipType>,
}

impl GameLog {
    
    pub fn to_game_log_string(
        &self,
        with_player_comments: bool,
        with_hole_cards_in_name: bool,
        //for the url
        hero_position: usize,
    ) -> String {
        let player_names = self
            .players
            .iter()
            .map(|p| {
                if with_hole_cards_in_name {
                    format!(
                        "{} ({})",
                        p.player_name,
                        p.cards.unwrap()
                    )
                } else {
                    p.player_name.clone()
                }
            })
            .collect::<Vec<String>>();

        //Find longest player id width
        let max_player_id_width = player_names
            .iter()
            .map(|player_name| player_name.len())
            .max()
            .unwrap_or(0);

        let mut s = String::new();
        s.push_str("*** Players ***\n");
        for (pi, player_state) in self.players.iter().enumerate() {
            s.push_str(&format!(
                "{:width$} - {} - {}\n",
                player_names[pi],
                player_state.stack,
                player_state.cards.unwrap(),
                width = max_player_id_width
            ));
        }

        s.push_str("*** Blinds ***\n");
        s.push_str(&format!(
            "{:width$} - {}\n",
            &player_names[0],
            min(
                self.sb,
                self.players[0].stack
            ),
            width = max_player_id_width
        ));
        s.push_str(&format!(
            "{:width$} - {}\n",
            &player_names[1],
            min(
                self.bb,
                self.players[1].stack
            ),
            width = max_player_id_width
        ));

        let mut round = Round::River;

        for action in &self.actions {
            if action.round != round {
                round = action.round;
                s.push_str(&format!("*** {} ***\n", round));

                if round == Round::Flop {
                    self.board[0..3]
                        .iter()
                        .for_each(|c| {
                            s.push_str(&format!("{} ", c));
                        });
                    s.push_str("\n");
                } else if round == Round::Turn {
                    s.push_str(&format!("{}\n", self.board[3]));
                } else if round == Round::River {
                    s.push_str(&format!("{}\n", self.board[4]));
                }
            }

            s.push_str(&format!(
                "{:width$} {} # {}{}\n",
                &player_names[action.player_index],
                action.action,
                if with_player_comments {
                    action.player_comment.as_deref().unwrap_or("")
                } else {
                    ""
                },
                if with_player_comments
                    && action.player_comment.is_some()
                {
                    " - "
                } else {
                    ""
                },
                width = max_player_id_width
            ));
        }

        //basically we assume if we have river cards, either someone is all in or 
        //round is already river
        let any_all_in = self.board.len() == 5;

        if any_all_in && round != Round::River {
            round = round.next().unwrap();
            loop {
                s.push_str(&format!("*** {} ***\n", round));

                if round == Round::Flop {
                    self.board[0..3]
                        .iter()
                        .for_each(|c| {
                            s.push_str(&format!("{} ", c));
                        });
                    s.push_str("\n");
                } else if round == Round::Turn {
                    s.push_str(&format!("{}\n", self.board[3]));
                } else if round == Round::River {
                    s.push_str(&format!("{}\n", self.board[4]));
                }

                if round == Round::River {
                    break;
                }
                round = round.next().unwrap();
            }
        }

        //Create a link with board and hero cards
        //http://localhost:5173/?board=2s7c8s2h2d&hole=As2c

        let board_url_param = self
            .board
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("");
        //Hard code hero as button
        let hero_cards = self.players[hero_position].cards.unwrap();
        let hero_url_param = format!("{}{}", hero_cards.get_hi_card(), hero_cards.get_lo_card());

        let url = format!(
            "http://localhost:5173/?board={}&hero={}",
            board_url_param, hero_url_param
        );
        s.push_str("*** Summary *** # ");
        s.push_str(&url);
        s.push_str("\n");

        for (pi, player_state) in self.players.iter().enumerate() {
            s.push_str(&format!(
                "{:width$} - {} # {} Started with {} change {}\n",
                &player_names[pi],
                player_state.stack,
                self.get_final_eval_comment(pi),
                player_state.stack,
                self.final_stacks[pi] as i64 - (player_state.stack as i64),
                //This info is in the last action for the player
                //player_state.player_name.total_put_in_pot,
                width = max_player_id_width
            ));
        }

        s
    }

    pub fn to_pokerstars_string(
        &self,        
        
    ) -> String {
        let player_names = self
            .players
            .iter()
            .map(|p| {
                
                    p.player_name.clone().replace("%", "")
                
            })
            .collect::<Vec<String>>();

        

        let mut s = String::new();

        s.push_str(&format!("PokerStars Hand #1704526657997: Hold'em No Limit (2/5) - 2024/01/06 00:00:00 WET\n"));
        s.push_str(&format!("Table 'WinningPokerHud' 9-max Seat #{} is the button\n", player_names.len()));

        for (pi, player_state) in self.players.iter().enumerate() {
            s.push_str(&format!(
                "Seat {}: {} ({} in chips)\n",
                pi + 1,
                player_names[pi],
                player_state.stack,
            ));
        }

        s.push_str(&format!(
            "{}: posts small blind {}\n",
            &player_names[0],
            min(
                self.sb,
                self.players[0].stack
            ),
        ));

        s.push_str(&format!(
            "{}: posts big blind {}\n",
            &player_names[1],
            min(
                self.bb,
                self.players[1].stack
            ),
        ));

        
        s.push_str("*** HOLE CARDS ***\n");

        for  player_state in self.players.iter() {
            let hole_cards = player_state.cards.unwrap();
            s.push_str(&format!(
                "Dealt to {} [{} {}]\n",
                player_state.player_name,
                hole_cards.get_hi_card(),
                hole_cards.get_lo_card(),
            ));
        }
        
      
        let mut round = Round::River;

        for action in &self.actions {
            if action.round != round {
                round = action.round;
                //Don't print preflop
                if round != Round::Preflop {
                    if round == Round::Flop {
                        s.push_str(&format!("*** FLOP *** [{}]\n", 
                        self.board[0..3]
                            .iter()
                            .map(|c| format!("{}", c)).join(" ")));
                    } else if round == Round::Turn {
                        s.push_str(&format!("*** TURN *** [{}] [{}]\n", 
                        self.board[0..3]
                            .iter()
                            .map(|c| format!("{}", c)).join(" "), self.board[3]));
                    } else if round == Round::River {
                        s.push_str(&format!("*** RIVER *** [{}] [{}]\n", 
                        self.board[0..4]
                            .iter()
                            .map(|c| format!("{}", c)).join(" "), self.board[4]));
                    }
                    
                }
            }

            s.push_str(&format!(
                "{}: {}\n",
                &player_names[action.player_index],
                action.action,
                
            ));
        }

        let mut last_action_for_player = Vec::with_capacity(player_names.len());
        for pi in 0..player_names.len() {
            let last_action = self.actions.iter().rev().find(|a| a.player_index == pi).unwrap();
            last_action_for_player.push(last_action);
        }

        s.push_str("*** SHOW DOWN ***\n");

        for (pi, player_state) in self.players.iter().enumerate() {
            if player_state.stack < player_state.stack {
                // stack = initial_stack + get from pot - put in pot
                // 2845 = 500 + 2845 - 500 
                // 700 = 500 + 400 - 200 
                // stack - initial_stack + put_in_pot = get_from_pot
                //Might need to add final call / raise ? 
                let get_from_pot = self.final_stacks[pi] + last_action_for_player[pi].total_amount_put_in_pot - player_state.stack; 
                s.push_str(&format!(
                    "{} collected {} from pot\n",
                    player_state.player_name,
                    get_from_pot,
                ));
            }
        }


        //might need to add last call/raise ?
        s.push_str("*** SUMMARY ***\n");
        s.push_str(&format!("Total pot {} | Rake 0\n", self.actions.last().unwrap().pot));
        s.push_str(&format!("Board [{}]\n", 
            self.board
                .iter()
                .map(|c| format!("{}", c)).join(" ")));

        s
    }

    fn get_final_eval_comment(&self, player_index: usize) -> String {
        let mut eval_cards = self.board.to_vec();
        eval_cards.extend(self.players[player_index].cards.unwrap().as_slice());

        let rank = rank_cards(eval_cards.iter());
        rank.print_winning(&eval_cards)
    }
}

impl FromStr for GameLog {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = GameLogParser::new();
        let mut remaining_str = s;

        let players = p.parse_players(&mut remaining_str)?;

        let (sb, bb) = p.parse_blinds(&players, &mut remaining_str)?;

        let _section_name = p.parse_section_name(&mut remaining_str, Some("Preflop"))?;

        let preflop_actions =
            p.parse_round_actions(&players, Round::Preflop, &mut remaining_str)?;

        //If fold to bb, the bb wouldn't act
        if preflop_actions.len() < players.len() - 1 {
            return Err(PokerError::from_string(format!(
                "Expected at least {} preflop actions, got {} in   {:.100}",
                players.len() - 1,
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

            let round_actions = p.parse_round_actions(&players, *round, &mut remaining_str)?;

            trace!("{} Round actions parsed for {}", round_actions.len(), round);

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

//Just so we can use this in a heap, consider everything equal
impl Ord for GameLog {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}
impl PartialOrd for GameLog {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for GameLog {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for GameLog {}

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
    use crate::{init_test_logger, ActionEnum};

    use super::*;

    //https://crates.io/crates/pokerlookup
    //https://github.com/traversgrayson/Poker-Hand-Parser/tree/master/Parser%20and%20Data
    #[test]
    fn test_parse_game_log_from_str() {
        init_test_logger();

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
Seat 2 calls 10 # call is difference needed, can put call amount in comments
Seat 3 checks # BB Acts last preflop
*** Flop ***
2s 7c 8s
Seat 2 checks
Seat 3 bets 5
Seat 2 folds
*** Summary ***
Seat 2 - 12  # This section is just to verify
Seat 3 - 148
Seat 6 - 55


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
                ..PlayerAction::default()
            }
        );
        assert_eq!(
            game_log.actions[1],
            PlayerAction {
                player_index: 0,
                action: ActionEnum::Call(10),
                round: Round::Preflop,
                ..PlayerAction::default()
            }
        );
        assert_eq!(
            game_log.actions[2],
            PlayerAction {
                player_index: 1,
                action: ActionEnum::Check,
                round: Round::Preflop,
                ..PlayerAction::default()
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
                ..PlayerAction::default()
            }
        );

        assert_eq!(
            game_log.actions[4],
            PlayerAction {
                player_index: 1,
                action: ActionEnum::Bet(5),
                round: Round::Flop,
                ..PlayerAction::default()
            }
        );

        assert_eq!(
            game_log.actions[5],
            PlayerAction {
                player_index: 0,
                action: ActionEnum::Fold,
                round: Round::Flop,
                ..PlayerAction::default()
            }
        );
    }

    #[test]
    fn test_parse_with_hole_cards() {
        init_test_logger();

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
Plyr C calls 10   # UTG acts first
Plyr D calls 10
Plyr A calls 10
Plyr B raises 10 to 20 
Plyr C folds
Plyr D calls 20 
Plyr A calls 20 
*** Flop ***
2s 7c 8s
Plyr A checks
Plyr B bets 5
Plyr D calls 5
Plyr A calls 5
*** Turn ***
2h 
Plyr A checks
Plyr B bets 5
Plyr D folds
Plyr A calls 5
*** River ***
2d
Plyr A bets 15
Plyr B raises 15 to 30 # minimum raise
Plyr A raises 15 to 45
Plyr B calls 45
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

        assert_eq!(
            game_log.players[0].cards.unwrap(),
            "As Kh".parse::<HoleCards>().unwrap()
        );
        assert_eq!(
            game_log.players[1].cards.unwrap(),
            "2d 2c".parse::<HoleCards>().unwrap()
        );
        assert_eq!(
            game_log.players[2].cards.unwrap(),
            "7d 2h".parse::<HoleCards>().unwrap()
        );
        assert_eq!(
            game_log.players[3].cards.unwrap(),
            "Ks Kh".parse::<HoleCards>().unwrap()
        );

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

    #[test]
    fn test_parse_machine_generated_game_log() {
        init_test_logger();

        let hh = "
        *** Players ***
Agent 0               - 500 - 9c 8s
Agent 1               - 500 - Td 7d
1 Calling Station 75% - 500 - Jh 2d
2 Calling Station 75% - 500 - Kc 2c
Agent 4               - 500 - Ac 8h
*** Blinds ***
Agent 0               - 2
Agent 1               - 5
*** Preflop ***
1 Calling Station 75% calls 5 # Player #2 1 Calling Station 75% calls 5 (of 5) with 41.7% pot equity with 12 in the pot
2 Calling Station 75% calls 5 # Player #3 2 Calling Station 75% calls 5 (of 5) with 29.4% pot equity with 17 in the pot
Agent 4               calls 5 # Player #4 Agent 4 calls 5 (of 5) with 22.7% pot equity with 22 in the pot
Agent 0               calls 5 # Player #0 Agent 0 calls 3 (of 5) with 12.0% pot equity with 25 in the pot
*** Flop ***
7c 3s 4s
Agent 0               checks #
Agent 1               checks #
1 Calling Station 75% checks #
2 Calling Station 75% checks #
Agent 4               checks #
*** Turn ***
2h
Agent 0               checks #
Agent 1               checks #
1 Calling Station 75% checks #
2 Calling Station 75% checks #
Agent 4               checks #
*** River ***
2s
Agent 0               checks #
Agent 1               checks #
1 Calling Station 75% checks #
2 Calling Station 75% checks #
Agent 4               checks #
*** Summary ***
Agent 0               - 495 # Started with 500 change -5
Agent 1               - 495 # Started with 500 change -5
1 Calling Station 75% - 495 # Started with 500 change -5
2 Calling Station 75% - 520 # Started with 500 change 20
Agent 4               - 495 # Started with 500 change -5
";
        let _game_log: GameLog = hh.parse().unwrap();
    }
}
