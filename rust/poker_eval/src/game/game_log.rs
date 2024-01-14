//A game log is all the information needed to reconstruct a game.

use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use std::str::FromStr;

use boomphf::Mphf;
use itertools::Itertools;
use log::trace;
use serde::Serialize;

use crate::ActionEnum;
use crate::Card;
use crate::FinalPlayerState;

use crate::InitialPlayerState;
use crate::OldRank;
use crate::board_hc_eval_cache_redb::EvalCacheWithHcReDb;
use crate::board_hc_eval_cache_redb::ProduceMonteCarloEval;
use crate::game::game_log_parser::GameLogParser;
use crate::monte_carlo_equity::get_equivalent_hole_board;
use crate::pre_calc::fast_eval::fast_hand_eval;
use crate::pre_calc::rank::Rank;
use crate::rank_cards;
use crate::ChipType;
use crate::PlayerAction;
use crate::PokerError;
use crate::Round;



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
    pub final_states: Vec<FinalPlayerState>,
    // Show best hand for all players, all rounds
    // v [round_index][player_index] = 5 best cards
    pub best_player_hands: Vec<Vec<[Card; 5]>>,

    //1 for best, etc.  can have repeated ranks for ties
    pub player_ranks_per_round: Vec<Vec<u8>>,
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
                    format!("{} ({})", p.player_name, p.cards.unwrap())
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
            min(self.sb, self.players[0].stack),
            width = max_player_id_width
        ));
        s.push_str(&format!(
            "{:width$} - {}\n",
            &player_names[1],
            min(self.bb, self.players[1].stack),
            width = max_player_id_width
        ));

        let mut round = Round::River;

        for action in &self.actions {
            if action.round != round {
                round = action.round;
                s.push_str(&format!("*** {} ***\n", round));

                if round == Round::Flop {
                    self.board[0..3].iter().for_each(|c| {
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
                if with_player_comments && action.player_comment.is_some() {
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
                    self.board[0..3].iter().for_each(|c| {
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

        let mut last_action_for_player = Vec::with_capacity(player_names.len());
        for pi in 0..player_names.len() {
            let last_action = self
                .actions
                .iter()
                .rev()
                .find(|a| a.player_index == pi)
                .unwrap();
            last_action_for_player.push(last_action.get_fields_after_action());
        }

        for (pi, player_state) in self.players.iter().enumerate() {
            s.push_str(&format!(
                "{:width$} - {} # {} Started with {} change {}; put in pot {}\n",
                &player_names[pi],
                self.final_stacks[pi],
                self.get_final_eval_comment(pi),
                player_state.stack,
                self.final_stacks[pi] as i64 - (player_state.stack as i64),
                last_action_for_player[pi].total_amount_put_in_pot,
                width = max_player_id_width
            ));
        }

        s
    }

    pub fn to_pokerstars_string(&self) -> String {
        let player_names = self
            .players
            .iter()
            .map(|p| p.player_name.clone().replace("%", ""))
            .collect::<Vec<String>>();

        let mut s = String::new();

        s.push_str(&format!(
            "PokerStars Hand #1704526657997: Hold'em No Limit (2/5) - 2024/01/06 00:00:00 WET\n"
        ));
        s.push_str(&format!(
            "Table 'WinningPokerHud' 9-max Seat #{} is the button\n",
            player_names.len()
        ));

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
            min(self.sb, self.players[0].stack),
        ));

        s.push_str(&format!(
            "{}: posts big blind {}\n",
            &player_names[1],
            min(self.bb, self.players[1].stack),
        ));

        s.push_str("*** HOLE CARDS ***\n");

        for player_state in self.players.iter() {
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
                        s.push_str(&format!(
                            "*** FLOP *** [{}]\n",
                            self.board[0..3].iter().map(|c| format!("{}", c)).join(" ")
                        ));
                    } else if round == Round::Turn {
                        s.push_str(&format!(
                            "*** TURN *** [{}] [{}]\n",
                            self.board[0..3].iter().map(|c| format!("{}", c)).join(" "),
                            self.board[3]
                        ));
                    } else if round == Round::River {
                        s.push_str(&format!(
                            "*** RIVER *** [{}] [{}]\n",
                            self.board[0..4].iter().map(|c| format!("{}", c)).join(" "),
                            self.board[4]
                        ));
                    }
                }
            }

            s.push_str(&format!(
                "{}: {}\n",
                &player_names[action.player_index], action.action,
            ));
        }

        let mut last_action_for_player = Vec::with_capacity(player_names.len());
        for pi in 0..player_names.len() {
            let last_action = self
                .actions
                .iter()
                .rev()
                .find(|a| a.player_index == pi)
                .unwrap();
            last_action_for_player.push(last_action.get_fields_after_action());
        }

        s.push_str("*** SHOW DOWN ***\n");

        for (pi, player_state) in self.players.iter().enumerate() {
            if player_state.stack < self.final_stacks[pi] {
                // stack = initial_stack + get from pot - put in pot
                // 2845 = 500 + 2845 - 500
                // 700 = 500 + 400 - 200
                // stack - initial_stack + put_in_pot = get_from_pot
                let get_from_pot = self.final_stacks[pi]
                    + last_action_for_player[pi].total_amount_put_in_pot
                    - player_state.stack;
                s.push_str(&format!(
                    "{} collected {} from pot\n",
                    player_state.player_name, get_from_pot,
                ));
            }
        }

        //might need to add last call/raise ?
        s.push_str("*** SUMMARY ***\n");
        s.push_str(&format!(
            "Total pot {} | Rake 0\n",
            self.actions.last().unwrap().pot
        ));
        s.push_str(&format!(
            "Board [{}]\n",
            self.board.iter().map(|c| format!("{}", c)).join(" ")
        ));

        s
    }

    fn get_final_eval_comment(&self, player_index: usize) -> String {
        let mut eval_cards = self.board.to_vec();
        eval_cards.extend(self.players[player_index].cards.unwrap().as_slice());

        let rank = rank_cards(eval_cards.iter());
        rank.print_winning(&eval_cards)
    }

    /*
    Fills in the field for best hands for each player in each round
    */
    pub fn calc_best_hands(&mut self) {
        let mut v: Vec<Vec<[Card; 5]>> = Vec::new();
        let mut player_rank_order: Vec<Vec<u8>> = Vec::new();

        let final_round = self.actions.last().unwrap().round;
        let mut round = Some(Round::Flop);

        while round.is_some() {
            let cur_round = round.unwrap();
            if final_round < cur_round {
                break;
            }

            let mut player_hand_ranks = self
                .players
                .iter()
                .enumerate()
                .map(|(p_idx, p)| {
                    let mut board_cards = self
                        .board
                        .iter()
                        .take(cur_round.get_num_board_cards())
                        .cloned()
                        .collect_vec();

                    board_cards.extend(p.cards.as_ref().unwrap().as_slice());

                    let rank = rank_cards(board_cards.iter());
                    let winning_cards = rank.get_winning(&board_cards);
                    (rank, p_idx, winning_cards)
                })
                .collect_vec();

            let best_player_hands = player_hand_ranks
                .iter()
                .map(|(_, _, wc)| {
                    *wc
                })
                .collect::<Vec<[Card; 5]>>();

            v.push(best_player_hands);

            player_hand_ranks.sort_by(|a, b| b.0.cmp(&a.0));

            //1 is best hand
            let mut cur_round_rank_order = vec![0; self.players.len()];
            let mut rank_order = 0;

            let mut last_rank_value = OldRank::StraightFlush(1 << 31);

            for (rank, p_idx, _) in player_hand_ranks.iter() {
                if rank != &last_rank_value {
                    rank_order += 1;
                    last_rank_value = *rank;
                }
                
                cur_round_rank_order[*p_idx] = rank_order;
            }

            player_rank_order.push(cur_round_rank_order);

            round = cur_round.next();
        }

        self.player_ranks_per_round = player_rank_order;
        self.best_player_hands = v;
    }

    pub fn get_csv_line(
        &self,
        hero_index: usize,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
        hash_func: &Mphf<u32>
    ) -> Result<CsvLineForPokerHand, PokerError> {
        let mut ret = CsvLineForPokerHand::default();

        let mut mc_db = monte_carlo_db.borrow_mut();
        //Position 0 sb, 1 bb, 2 UTG

        ret.position = hero_index as u8;

        //Number of players in hand

        ret.players_starting_preflop = self.players.len() as u8;
        

        let mut when_players_folded: Vec<Option<Round>> = vec![None; self.players.len()];

        let mut first_hero_action = true;

        let mut cur_round = Round::Preflop;
        for action in self.actions.iter() {
            trace!("Action {}", action.non_folded_players);

            if action.action == ActionEnum::Fold {
                when_players_folded[action.player_index] = Some(action.round);
            }

            //First action of round
            if action.round != cur_round {
                cur_round = action.round;

                match cur_round {
                    Round::Flop => {
                        ret.players_starting_flop = action.non_folded_players as u8;
                        ret.pot_start_flop = action.pot as f64 / self.bb as f64;
                    }
                    Round::Turn => {
                        ret.players_starting_turn = action.non_folded_players as u8;
                        ret.pot_start_turn = action.pot as f64 / self.bb as f64;
                    }
                    Round::River => {
                        ret.players_starting_river = action.non_folded_players as u8;
                        ret.pot_start_river = action.pot as f64 / self.bb as f64;
                    }
                    _ => panic!("Invalid round"),
                }

                first_hero_action = true;
            }

            if action.player_index == hero_index && first_hero_action {
                let (eq_hole_cards, mut eq_board) = get_equivalent_hole_board(
                    &self.players[hero_index].cards.as_ref().unwrap(),
                    &self.board[0..cur_round.get_num_board_cards()],
                );
                eq_board.get_index();

                trace!("First hero action {}", &action);

                assert!(action.non_folded_players >= 2);

                //First hero action of round
                match cur_round {
                    Round::Preflop => {
                        ret.players_before_hero_pre_flop =
                            self.players.len() as u8 - action.players_left_to_act - 1;
                        ret.hero_eq_start_pre_flop = mc_db
                            .get_put(&eq_board, &eq_hole_cards, action.non_folded_players)
                            .unwrap();
                        ret.amt_to_call_start_preflop =
                            action.current_amt_to_call as f64 / self.bb as f64;
                        ret.first_action_preflop = action.action.into();
                        ret.first_action_amount_preflop =
                            get_action_amount(&action, self.bb);
                    }
                    Round::Flop => {
                        ret.players_before_hero_flop =
                            ret.players_starting_flop as u8 - action.players_left_to_act - 1;
                        ret.hero_eq_start_flop = mc_db
                            .get_put(&eq_board, &eq_hole_cards, action.non_folded_players)
                            .unwrap();
                        ret.amt_to_call_start_flop =
                            action.current_amt_to_call as f64 / self.bb as f64;
                        ret.first_action_flop = action.action.into();
                        ret.first_action_amount_flop =
                            get_action_amount(&action, self.bb);
                    }
                    Round::Turn => {
                        ret.players_before_hero_turn =
                            ret.players_starting_turn as u8 - action.players_left_to_act - 1;
                        ret.hero_eq_start_turn = mc_db
                            .get_put(&eq_board, &eq_hole_cards, action.non_folded_players)
                            .unwrap();
                        ret.amt_to_call_start_turn =
                            action.current_amt_to_call as f64 / self.bb as f64;
                        ret.first_action_turn = action.action.into();
                        ret.first_action_amount_turn =
                            get_action_amount(&action, self.bb);
                    }
                    Round::River => {
                        ret.players_before_hero_river =
                            ret.players_starting_river as u8 - action.players_left_to_act - 1;
                        ret.hero_eq_start_river = mc_db
                            .get_put(&eq_board, &eq_hole_cards, action.non_folded_players)
                            .unwrap();
                        ret.amt_to_call_start_river =
                            action.current_amt_to_call as f64 / self.bb as f64;
                        ret.first_action_river = action.action.into();
                        ret.first_action_amount_river =
                            get_action_amount(&action, self.bb);
                    }
                }

                first_hero_action = false;
            }
        }

        //calculate hand strength in each round
        for round_index in (Round::Flop as u8)..=(Round::River as u8) {

            //If everyone folded, we are done
            if self.board.is_empty() {
                break;
            }

            let round = round_index.try_into()?;

            let players_in_hand = (0..self.players.len())
                .filter(|p_idx| {
                    //either never folded or folded on or after this round
                    //So a player that folded in the flop is still there at start of flop
                    when_players_folded[*p_idx].is_none()
                        || when_players_folded[*p_idx].unwrap() >= round
                })
                .collect_vec();

            let round_var = 
            match round {
                Round::Flop => &mut ret.players_starting_flop,
                Round::Turn => &mut ret.players_starting_turn,
                Round::River => &mut ret.players_starting_river,
                _ => panic!("Invalid round"),
                
            };

            //In case of all ins, we might not have the rounds with no actions
            if *round_var == 0 {
                *round_var = players_in_hand.len() as u8;
            } else {
                //if it is set, it should be consistent
                assert_eq!(*round_var, players_in_hand.len() as u8);
            }

            if players_in_hand.len() <= 0 {
                continue;
            }
            
            trace!("Round {} Players in hand {}", 
                round,
                players_in_hand.len());
            // for p_idx in players_in_hand.iter() {
            //     trace!("Player {} has not folded", &self.players[*p_idx].player_name);
            // }
            //assert_eq!(players_in_hand.len(), num_players_in_round as usize);

            let hero_strength = fast_hand_eval(
                self.board.iter().take(round.get_num_board_cards()).chain(
                    self.players[hero_index].cards.as_ref().unwrap().as_slice()), hash_func);

            let all_strength = players_in_hand
                .iter()
                .map(|p_idx| {
                    fast_hand_eval(self.board.iter().take(round.get_num_board_cards()).chain(
                        self.players[*p_idx].cards.as_ref().unwrap().as_slice()), hash_func)
                })
                .collect_vec();

            let num_above = all_strength.iter().filter(|s| **s > hero_strength).count();

            match round {
                Round::Preflop => panic!("Hand rank not on preflop"),
                Round::Flop => ret.hero_hand_rank_flop = (num_above + 1) as u8,
                Round::Turn => ret.hero_hand_rank_turn = (num_above + 1) as u8,
                Round::River => ret.hero_hand_rank_river = (num_above + 1) as u8,
            }
        }

        ret.initial_stack = self.players[hero_index].stack as f64 / self.bb as f64;
        ret.final_stack = self.final_stacks[hero_index] as f64 / self.bb as f64;
        ret.final_pot = self.actions.last().unwrap().get_fields_after_action().pot as f64 / self.bb as f64;
        
        ret.in_showdown = match self.final_states[hero_index] {
            FinalPlayerState::Folded(_) => false,
            FinalPlayerState::LostShowdown => true,
            FinalPlayerState::WonShowdown => true,
            FinalPlayerState::EveryoneElseFolded => false,
        };
        
        if ret.in_showdown {

            //In the case of an all in, we may not have round data
            if ret.pot_start_river == 0.0 {
                ret.pot_start_river = ret.final_pot;
            }
            if ret.pot_start_turn == 0.0 {
                ret.pot_start_turn = ret.final_pot;
            }
            if ret.pot_start_flop == 0.0 {
                ret.pot_start_flop = ret.final_pot;
            }

            let player_ranks = self.players.iter().enumerate().map(|(p_idx, p)| {

                if self.final_states[p_idx].is_folded() {
                    Rank::lowest_rank()
                } else {

                    let mut board_cards = self.board.clone();
                    board_cards.extend(p.cards.as_ref().unwrap().as_slice());

                    fast_hand_eval(self.board.iter().chain(p.cards.as_ref().unwrap().as_slice()), hash_func)
                }
            }).collect_vec();

            let hero_rank = player_ranks[hero_index];
            let max_rank = player_ranks.iter().enumerate().map(|(p_idx, rank)|{
                if p_idx == hero_index {
                    Rank::lowest_rank()
                } else {
                    *rank
                }
            }).max().unwrap();
            ret.hero_hand_showdown = hero_rank.get_rank_enum() as u8;
            ret.non_hero_hand_showdown = max_rank.get_rank_enum() as u8;
        }



        Ok(ret)
    }
}

fn get_action_amount(action: &PlayerAction, bb: ChipType) -> f64 {
    match action.action {
        ActionEnum::Fold => 0.0,
        //As this is the 1st action, this would not be calling a raise so should be == to call amount
        ActionEnum::Call(amount) => amount as f64 / bb as f64,
        ActionEnum::Check => 0.0,
        ActionEnum::Bet(amount) => amount as f64 / bb as f64,
        ActionEnum::Raise(_, amount) => amount as f64 / bb as f64,
    }

}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum ActionString {
    Fold,
    Call,
    Check,
    Bet,
    Raise,
    CheckRaise,
    NA,
}

impl From<ActionEnum> for ActionString {
    fn from(value: ActionEnum) -> Self {
        match value {
            ActionEnum::Fold => ActionString::Fold,
            ActionEnum::Call(_) => ActionString::Call,
            ActionEnum::Check => ActionString::Check,
            ActionEnum::Bet(_) => ActionString::Bet,
            ActionEnum::Raise(_, _) => ActionString::Raise,
        }
    }
}

impl Default for ActionString {
    fn default() -> Self {
        ActionString::NA
    }
}

// impl Serialize for ActionString {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(match self {
//             ActionString::Fold => "fold",
//             ActionString::Call => "call",
//             ActionString::Check => "check",
//             ActionString::Bet => "bet",
//             ActionString::Raise => "raise",
//             ActionString::CheckRaise => "check-raise",
//             ActionString::NA => "NA",
//         })
//     }
// }


#[derive(Serialize, Debug, Default)]
pub struct CsvLineForPokerHand {
    #[serde(rename = "POSITION")]
    pub position: u8,

    //Number of players in hand
    #[serde(rename = "PLR_START_PREFLOP")]
    pub players_starting_preflop: u8,

    //Number of players starting flop
    //Number of players starting turn
    //Number of players starting river
    #[serde(rename = "PLR_START_FLOP")]
    pub players_starting_flop: u8,

    #[serde(rename = "PLR_START_TURN")]
    pub players_starting_turn: u8,

    #[serde(rename = "PLR_START_RIVER")]
    pub players_starting_river: u8,

    //Among all non folded players, before action starts
    //best hand is 1, ties also 1
    //calculated even if hero folded
    //Useful to calculate when folded too much
    #[serde(rename = "HND_RNK_PREFLOP")]
    pub hero_hand_rank_preflop: u8,

    #[serde(rename = "HND_RNK_FLOP")]
    pub hero_hand_rank_flop: u8,

    #[serde(rename = "HND_RNK_TURN")]
    pub hero_hand_rank_turn: u8,

    #[serde(rename = "HND_RNK_RIVER")]
    pub hero_hand_rank_river: u8,


    //Number of players acting before hero on flop (so 3rd, this == 2)
    //Number of players acting before hero on turn
    //Number of players acting before hero on river
    #[serde(rename = "PLR_BEFORE_HERO_PREFLOP")]
    pub players_before_hero_pre_flop: u8,

    #[serde(rename = "PLR_BEFORE_HERO_FLOP")]
    pub players_before_hero_flop: u8,

    #[serde(rename = "PLR_BEFORE_HERO_TURN")]
    pub players_before_hero_turn: u8,

    #[serde(rename = "PLR_BEFORE_HERO_RIVER")]
    pub players_before_hero_river: u8,

    //Pot at start of flop
    //Pot at start of turn
    //Pot at start of river
    //Final pot
    #[serde(rename = "POT_START_FLOP")]
    pub pot_start_flop: f64,

    #[serde(rename = "POT_START_TURN")]
    pub pot_start_turn: f64,

    #[serde(rename = "POT_START_RIVER")]
    pub pot_start_river: f64,

    #[serde(rename = "FINAL_POT")]
    pub final_pot: f64,

    //Hero eq at start of flop
    //Hero eq at start of turn
    //Hero eq at start of river
    #[serde(rename = "HERO_EQ_PREFLOP")]
    pub hero_eq_start_pre_flop: f64,

    //This is simulating equity up until the river
    #[serde(rename = "HERO_EQ_FLOP")]
    pub hero_eq_start_flop: f64,

    #[serde(rename = "HERO_EQ_TURN")]
    pub hero_eq_start_turn: f64,

    #[serde(rename = "HERO_EQ_RIVER")]
    pub hero_eq_start_river: f64,

    //Amt to call at start of flop
    //Amt to call at start of turn
    //Amt to call at start of river
    #[serde(rename = "CALL_AMT_PREFLOP")]
    pub amt_to_call_start_preflop: f64,

    #[serde(rename = "CALL_AMT_FLOP")]
    pub amt_to_call_start_flop: f64,

    #[serde(rename = "CALL_AMT_TURN")]
    pub amt_to_call_start_turn: f64,

    #[serde(rename = "CALL_AMT_RIVER")]
    pub amt_to_call_start_river: f64,

    //First Action pre-flop (raise, fold, call, check (for bb))
    //first action flop (check, bet, raise, fold, call, check-raise?)
    //Action turn
    //Action river
    #[serde(rename = "ACT_PREFLOP")]
    pub first_action_preflop: ActionString,

    #[serde(rename = "ACT_FLOP")]
    pub first_action_flop: ActionString,

    #[serde(rename = "ACT_TURN")]
    pub first_action_turn: ActionString,

    #[serde(rename = "ACT_RIVER")]
    pub first_action_river: ActionString,

    //First action amount pre-flop (check raise will be this amount)
    //First action amount flop
    //First action amount turn
    //First action amount river
    #[serde(rename = "ACT_AMT_PREFLOP")]
    pub first_action_amount_preflop: f64,

    #[serde(rename = "ACT_AMT_FLOP")]
    pub first_action_amount_flop: f64,

    #[serde(rename = "ACT_AMT_TURN")]
    pub first_action_amount_turn: f64,

    #[serde(rename = "ACT_AMT_RIVER")]
    pub first_action_amount_river: f64,

    #[serde(rename = "IN_SHOWDOWN")]
    pub in_showdown: bool,

    //0 high card, 1 pair
    #[serde(rename = "HERO_HAND")]
    pub hero_hand_showdown: u8,

    #[serde(rename = "BEST_NON_HERO_HAND")]
    pub non_hero_hand_showdown: u8,

    #[serde(rename = "INIT_STACK")]
    pub initial_stack: f64,

    #[serde(rename = "FINAL_STACK")]
    pub final_stack: f64,
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

// #[allow(dead_code)]
// pub struct CurrentPlayerState {
//     stack: ChipType,
//     //Player id?
//     folded: bool,
//     all_in: bool,
// }

// //Now when we play back a game, we can pass the current state to the UI
// #[allow(dead_code)]
// struct GameState {
//     player_states: Vec<CurrentPlayerState>,

//     current_to_act: usize,

//     pot: ChipType,

//     current_to_call: ChipType,
// }

#[cfg(test)]
mod tests {
    use crate::{init_test_logger, ActionEnum, pre_calc::perfect_hash::load_boomperfect_hash, game_log_source::GameLogSource, game_runner_source::GameRunnerSourceEnum, GameRunner, HoleCards};

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

    //Needs db
    //#[test]
    #[allow(dead_code)]
    fn test_csv_line() {
        
        init_test_logger();
    
    //Same as test_parse_with_hole_cards
        let hh = "
    *** Players *** 
    Plyr A - 12 - As Kh
    Plyr B - 147 - 2d 2c
    Plyr C - 55 - 7d 2h
    Plyr D - 55 - Ks Kc
    *** Blinds *** 
    Plyr A - 5
    Plyr B - 10
    *** Preflop ***
    Plyr C calls 10   # UTG acts first
    Plyr D calls 10
    Plyr A calls 5
    Plyr B raises 10 to 20 
    Plyr C folds
    Plyr D calls 10 
    Plyr A calls 2 # all in 
    *** Flop ***
    2s 7c 8s
    Plyr B bets 10
    Plyr D raises 10 to 20
    Plyr B calls 10
    *** Turn ***
    3h 
    Plyr B bets 10
    Plyr D folds
    *** River ***
    Kd
    *** Summary ***
    Plyr A - 0
    Plyr B - 209
    Plyr C - 45 
    Plyr D - 15
        ";
        let parsed_game_log: GameLog = hh.parse().unwrap();

        let monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
        EvalCacheWithHcReDb::new().unwrap();
        let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

        let hash_func = load_boomperfect_hash();

        //We run it through the game runner to have all fields filled in
        let game_log_source: GameLogSource = GameLogSource::new(parsed_game_log);

        let mut game_runner2 = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

        for _ in 0..200 {
            let r = game_runner2.process_next_action().unwrap();
            if r {
                break;
            }
        }

        let log2 = game_runner2
            .to_game_log()
            .unwrap();

        //Player B
        let game_line = log2.get_csv_line(1, rcref_mcedb.clone(), &hash_func).unwrap();

        assert_eq!(game_line.first_action_amount_preflop, 2.0);
        assert_eq!(game_line.first_action_amount_flop, 1.0);
        assert_eq!(game_line.first_action_amount_turn, 1.0);
        assert_eq!(game_line.first_action_amount_river, 0.0);

        assert_eq!(game_line.players_before_hero_pre_flop, 3);
        assert_eq!(game_line.players_before_hero_flop, 1);
        assert_eq!(game_line.players_before_hero_turn, 1);

        assert_eq!(game_line.players_starting_flop, 3);
        assert_eq!(game_line.players_starting_turn, 3);
        //1 is all in
        assert_eq!(game_line.players_starting_river, 2);

        assert_eq!(game_line.pot_start_flop, 62.0 / 10.0);
        assert_eq!(game_line.pot_start_turn, 102.0 / 10.0);
        assert_eq!(game_line.pot_start_river, 112.0 / 10.0);

        assert_eq!(game_line.final_stack, 209.0 / 10.0);

        //Player D
        let game_line = log2.get_csv_line(3, rcref_mcedb, &hash_func).unwrap();
        assert_eq!(game_line.first_action_amount_preflop, 1.0);
        assert_eq!(game_line.first_action_amount_flop, 2.0);
        assert_eq!(game_line.first_action_turn, ActionString::Fold);
        assert_eq!(game_line.first_action_amount_turn, 0.0);
        assert_eq!(game_line.first_action_river, ActionString::NA);
        assert_eq!(game_line.first_action_amount_river, 0.0);

        assert_eq!(game_line.pot_start_flop, 62.0 / 10.0);
        assert_eq!(game_line.pot_start_turn, 102.0 / 10.0);
        //We didn't make it
        assert_eq!(game_line.pot_start_river, 0.0);

        assert_eq!(game_line.final_stack, 15.0 / 10.0);

        //All in user counts, that's why it's 2 not 1
        assert_eq!(game_line.players_before_hero_turn, 2);

        assert_eq!(game_line.hero_hand_rank_flop, 2);
        assert_eq!(game_line.hero_hand_rank_turn, 2);
        assert_eq!(game_line.hero_hand_rank_river, 1);
    }
}
