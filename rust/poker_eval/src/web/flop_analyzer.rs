use std::{cmp, fmt::Display, mem};

use log::{debug, error, info, trace, warn};
use postflop_solver::card_pair_to_index;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    get_filtered_range_set, range_string_to_set, rank_cards, Card, CardUsedType, InRangeType, Rank, NUM_RANK_FAMILIES,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
//extern crate wasm_bindgen;
//extern crate console_error_panic_hook;
type ResultType = u32;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Default, Serialize, Clone)]
pub struct Results {
    pub num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    win_eq: f64,
    tie_eq: f64,

    //total is win+tie

    //count made hands
    num_gut_shots: ResultType,

    //2 cards to straight
    num_str8_draw: ResultType,

    num_flush_draw: ResultType,
    num_top_pair: ResultType,

    rank_family_count: [ResultType; 9],
}

#[wasm_bindgen]
impl Results {
    pub fn get_perc_family(&self, family_index: usize) -> f64 {
        self.rank_family_count[family_index] as f64 / self.num_iterations as f64
    }
    pub fn get_perc_family_or_better(&self, family_index: usize) -> f64 {
        let mut total=0.0;
        for i in family_index..NUM_RANK_FAMILIES {
            total+=self.get_perc_family(i)
        }
        total
    }
    pub fn get_equity(&self) -> f64 {
        self.win_eq + self.tie_eq
    }
}

#[derive(Debug)]
pub struct MyError {
    details: String,
}

impl MyError {
    fn from_str(msg: &str) -> MyError {
        MyError {
            details: msg.to_string(),
        }
    }
    fn from_string(msg: String) -> MyError {
        MyError { details: msg }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl From<MyError> for JsValue {
    fn from(failure: MyError) -> Self {
        js_sys::Error::new(&failure.to_string()).into()
    }
}

#[derive(Clone, Eq, PartialEq)]
#[repr(u8)]
enum PlayerPreFlopState {
    Disabled = 0,
    UseHoleCards = 1,
    UseRange = 2,
}

impl From<u8> for PlayerPreFlopState {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(cmp::min(value, 2)) }
    }
}

impl Default for PlayerPreFlopState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PreflopPlayerInfo {
    range_string: String,
    results: Results,
    hole_cards: Vec<Card>,
    range_set: InRangeType,
    state: PlayerPreFlopState,
}

#[wasm_bindgen]
//doing this to stop warnings in vs code about camel case in the wasm function names
#[allow(non_camel_case_types)]
pub struct flop_analyzer {
    board_cards: Vec<Card>,
    player_info: Vec<PreflopPlayerInfo>,
    villian_results: Results,

    pub num_iterations: u32,
}

//hero is 0
const MAX_PLAYERS: usize = 5;

const MAX_RAND_NUMBER_ATTEMPS: usize = 1000;

#[wasm_bindgen]
impl flop_analyzer {
    pub fn new() -> Self {
        if cfg!(any(target_family = "wasm")) {
            console_error_panic_hook::set_once();
            wasm_logger::init(wasm_logger::Config::default());
        }

        info!("FlopAnalyzer::new()");
        debug!("debug");
        warn!("warn");
        trace!("trace");
        error!("error");

        info!("Initializing FlopAnalyzer ");

        Self {
            board_cards: Vec::with_capacity(7),
            player_info: Vec::with_capacity(MAX_PLAYERS),

            villian_results: Results::default(),

            num_iterations: 0,
        }
    }

    pub fn set_board_cards(&mut self, cards: &[u8]) {
        self.board_cards.clear();
        info!("set_board_cards: len {}", cards.len());
        let bc = &mut self.board_cards;
        for c in cards.iter() {
            let card = Card::from(*c);
            debug!("card: {:?}", card);
            bc.push(card);
        }
    }

    pub fn set_player_cards(&mut self, player_idx: usize, cards: &[u8]) {
        info!("set_player_cards: {} len {}", player_idx, cards.len());
        let pc = &mut self.player_info[player_idx].hole_cards;
        pc.clear();
        for c in cards.iter() {
            let card = Card::from(*c);
            debug!("card: {:?}", card);
            pc.push(card);
        }
    }

    pub fn set_player_range(&mut self, player_idx: usize, range_str: &str) {
        info!("set_player_range: {} [{}]", player_idx, range_str);

        if range_str.is_empty() {
            warn!("set_player_range: empty string");
            return;
        }
        let range_set = range_string_to_set(range_str);

        info!("% is {}", range_set.count_ones() as f64 / 2652.0);

        self.player_info[player_idx].range_set = range_set;
        self.player_info[player_idx].range_string = range_str.to_string();
    }

    pub fn set_player_state(&mut self, player_idx: usize, state: u8) {
        info!("set_player_state: {} {}", player_idx, state);
        self.player_info[player_idx].state = state.into();
    }

    pub fn clear_player_cards(&mut self, player_idx: usize) {
        self.player_info[player_idx].hole_cards.clear();
    }

    pub fn reset(&mut self) {
        self.board_cards.clear();
        self.player_info.clear();
        info!("reset to {} players", MAX_PLAYERS);
        for _ in 0..MAX_PLAYERS {
            let p_info = PreflopPlayerInfo::default();
            //p_info.results.rank_family_count = vec![0; 9];
            self.player_info.push(p_info);
        }
        self.villian_results = Results::default();

        self.num_iterations = 42;
    }

    pub fn simulate_flop(&mut self, num_iterations: u32) -> Result<(), MyError> {
        let n_players = self.player_info.len();
        let mut rng = StdRng::seed_from_u64(42);

        let n_active_players = self
            .player_info
            .iter()
            .filter(|p| p.state != PlayerPreFlopState::Disabled)
            .count();

        //let mut dist_count = [0; 52*51/2];

        for _it_num in 0..num_iterations {
            //debug!("simulate_flop: iteration {}", it_num);

            //let mut deck = self.prepare_deck();

            //with flop, players with hole cards
            let mut eval_cards = Vec::with_capacity(7);
            let mut used_cards = self.init_cards_used(&mut eval_cards)?;

            assert_eq!(3, self.board_cards.len());

            // assert_eq!(7, used_cards.count_ones());
            // assert_eq!(11029519011840, used_cards.data[0]);
            // let dbg_fs = get_filtered_range_set(&self.player_info[2].range_set, used_cards);
            // assert_eq!(373, dbg_fs.count_ones());
            //Even if we set some cards for players, for simplicity get enough for everyone ( 2 * n_players ) + turn & river
            //let (shuffled_cards, _) = deck.partial_shuffle(&mut rng, 2 + 2 * n_players);

            let mut hand_evals: Vec<Option<Rank>> = vec![None; n_players];

            //First we choose hole cards for players that are using a range
            self.set_cards_from_ranges(&mut rng, &mut used_cards)?;

            //debugging
            // let p2_ridx = card_pair_to_index(self.player_info[2].hole_cards[0].into(),
            //     self.player_info[2].hole_cards[1].into());
            // dist_count[p2_ridx] += 1;
            //end debugging

            assert_eq!(
                self.board_cards.len() + 2 * n_active_players,
                used_cards.count_ones()
            );

            //Do just river for now
            assert_eq!(3, eval_cards.len());
            self.add_board_card(
                get_unused_card(&mut rng, &mut used_cards).unwrap(),
                &mut eval_cards,
                &mut used_cards,
            )?;
            self.add_board_card(
                get_unused_card(&mut rng, &mut used_cards).unwrap(),
                &mut eval_cards,
                &mut used_cards,
            )?;

            assert_eq!(5 + 2 * n_active_players, used_cards.count_ones());
            assert_eq!(5, eval_cards.len());

            for p_idx in 0..n_players {
                if self.player_info[p_idx].state == PlayerPreFlopState::Disabled {
                    continue;
                }

                //For players with ranges we already chose their cards

                //if self.player_info[p_idx].state == PlayerPreFlopState::UseHoleCards {
                if 2 != self.player_info[p_idx].hole_cards.len() {
                    return Err(MyError::from_string(format!(
                        "simulate_flop: player {} has {} hole cards",
                        p_idx,
                        self.player_info[p_idx].hole_cards.len()
                    )));
                }
                eval_cards.extend(self.player_info[p_idx].hole_cards.iter());
                // } else {
                //     self.board_cards.push(shuffled_cards[2 + 2 * p_idx]);
                //     self.board_cards.push(shuffled_cards[2 + 2 * p_idx + 1]);
                // }

                {
                    let results = &mut self.player_info[p_idx].results;
                    results.num_iterations += 1;
                }

                //Did the player fold?
                // if self.player_info[p_idx].state == PlayerPreFlopState::UseRange {
                //     let range_index = core_cards_to_range_index(
                //         shuffled_cards[2 * p_idx],
                //         shuffled_cards[2 * p_idx + 1],
                //     );
                //     if !self.player_info[p_idx].range_set[range_index] {
                //         {
                //             let results = &mut self.player_info[p_idx].results;
                //             results.folded += 1;
                //         }
                //         self.board_cards.pop();
                //         self.board_cards.pop();
                //         hand_evals[p_idx] = None;
                //         continue;
                //     }
                // }

                let rank = rank_cards(&eval_cards);

                let results = &mut self.player_info[p_idx].results;
                update_results_from_rank(results, rank);

                hand_evals[p_idx] = Some(rank);

                eval_cards.pop();
                eval_cards.pop();

                assert_eq!(5, eval_cards.len());
            }

            let (winner_indexes, _num_non_folded) = indices_of_max_values(&hand_evals);

            assert!(winner_indexes.len() > 0);

            for winner_idx in winner_indexes.iter() {
                let results = &mut self.player_info[*winner_idx].results;
                if winner_indexes.len() == 1 {
                    results.win_eq += 1.0;
                } else {
                    results.tie_eq += 1.0 / winner_indexes.len() as f64;
                }
            }

            //pop turn & river
            eval_cards.pop();
            eval_cards.pop();

            assert_eq!(3, eval_cards.len());
        }

        // let non_zero_count = dist_count.iter().filter(|&&x| x > 0).count();
        // assert_eq!(non_zero_count, 373);
        // for i in 0..52*51/2 {
        //     if dist_count[i] > 0 {
        //         debug!("{} = {} / {} = {:.3}", i, dist_count[i], num_iterations, 100.0*dist_count[i] as f64 / num_iterations as f64);
        //     }
        // }

        Ok(())
    }

    fn add_board_card(
        &self,
        c_index: usize,
        eval_cards: &mut Vec<Card>,
        cards_used: &mut CardUsedType,
    ) -> Result<(), MyError> {
        let count_before = cards_used.count_ones();
        cards_used.set(c_index, true);
        let count_after = cards_used.count_ones();

        if count_before + 1 != count_after {
            return Err(MyError::from_string(format!(
                "Card already used {} in board",
                Card::from(c_index).to_string()
            )));
        }

        eval_cards.push(Card::from(c_index));

        Ok(())
    }

    fn init_cards_used(&self, eval_cards: &mut Vec<Card>) -> Result<CardUsedType, MyError> {
        let mut cards_used = CardUsedType::default();
        for c in self.board_cards.iter() {
            self.add_board_card((*c).into(), eval_cards, &mut cards_used)?;
        }

        for p_idx in 0..self.player_info.len() {
            if self.player_info[p_idx].state != PlayerPreFlopState::UseHoleCards {
                continue;
            }

            for c in self.player_info[p_idx].hole_cards.iter() {
                let count_before = cards_used.count_ones();
                cards_used.set(c.to_range_index_part(), true);
                let count_after = cards_used.count_ones();

                if count_before + 1 != count_after {
                    return Err(MyError::from_str(
                        format!("Card already used {} in pidx {}", c.to_string(), p_idx).as_str(),
                    ));
                }
            }
        }

        Ok(cards_used)

        // let mut deck: Vec<Card> = Vec::with_capacity(52);
        // for c in 0..52 {
        //     if !cards_used[c] {
        //         deck.push(Card::from_range_index_part(c));
        //     }
        // }
        // deck
    }

    fn set_cards_from_ranges(
        &mut self,
        rng: &mut StdRng,
        cards_used: &mut CardUsedType,
    ) -> Result<(), MyError> {
        let num_players = self.player_info.len();
        for p_idx in 0..num_players {
            if self.player_info[p_idx].state != PlayerPreFlopState::UseRange {
                continue;
            }

            let mut attempts = 0;
            let mut card1_index;
            let mut card2_index;

            loop {
                attempts += 1;

                if attempts > MAX_RAND_NUMBER_ATTEMPS {
                    return Err(MyError::from_string(
                        format!("Unable to find cards for player {} after {} attempts.  Cards used count {} range str {} == {:.1}%",
                        p_idx, attempts, cards_used.count_ones(),
                        &self.player_info[p_idx].range_string, self.player_info[p_idx].range_set.count_ones() as f64 / 2652.0 * 100.0)
                    ));
                }

                card1_index = get_unused_card(rng, cards_used).unwrap();
                card2_index = get_unused_card(rng, cards_used).unwrap();

                if card1_index == card2_index {
                    continue;
                }

                let range_index = card_pair_to_index(card1_index as u8, card2_index as u8);

                if !self.player_info[p_idx].range_set[range_index] {
                    continue;
                }

                break;
            }

            //we set their cards
            let hc = &mut self.player_info[p_idx].hole_cards;
            hc.clear();
            hc.push(Card::from_range_index_part(card1_index));
            hc.push(Card::from_range_index_part(card2_index));
            let count_before = cards_used.count_ones();
            cards_used.set(card1_index, true);
            cards_used.set(card2_index, true);
            let count_after = cards_used.count_ones();

            if count_before + 2 != count_after {
                return Err(MyError::from_str(
                    format!("Range choice invalid for pidx {}", p_idx).as_str(),
                ));
            }
        }

        Ok(())
    }

    pub fn get_results(&self) -> Vec<Results> {
        self.player_info.iter().map(|p| p.results.clone()).collect()
    }

    pub fn get_result(&self, player_idx: usize) -> Result<JsValue, JsValue> {
        info!(
            "get_result: {} num iterations {}",
            player_idx, self.player_info[player_idx].results.num_iterations
        );

        Ok(serde_wasm_bindgen::to_value(
            &&self.player_info[player_idx].results,
        )?)
    }
}

fn update_results_from_rank(results: &mut Results, rank: Rank) {
    results.rank_family_count[rank.get_family_index()] += 1;
}

//returns winners and how many players were considered (non None rank)
fn indices_of_max_values(arr: &[Option<Rank>]) -> (Vec<usize>, usize) {
    let mut non_none_count = 0;
    let mut max_indices = Vec::with_capacity(MAX_PLAYERS);
    let mut max_value = Rank::HighCard(0);

    for (index, &value) in arr.iter().enumerate() {
        if let Some(value) = value {
            non_none_count += 1;
            if value > max_value {
                max_value = value;
                max_indices.clear();
                max_indices.push(index);
            } else if value == max_value {
                max_indices.push(index);
            }
        }
    }

    (max_indices, non_none_count)
}

fn get_unused_card(rng: &mut StdRng, cards_used: &CardUsedType) -> Option<usize> {
    let mut attempts = 0;
    loop {
        let rand_int: usize = rng.gen_range(0..52);
        assert!(rand_int < 52);
        //let card = Card::from(rand_int);
        if !cards_used[rand_int] {
            return Some(rand_int);
        }
        attempts += 1;
        if attempts > MAX_RAND_NUMBER_ATTEMPS {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{card_u8s_from_string, web::flop_analyzer::PlayerPreFlopState};

    fn assert_equity(equity: f64, target: f64, tolerance: f64) {
        let passed = (equity - target).abs() < tolerance;
        if !passed {
            println!("assert_equity failed: {} != {}", equity, target);
        }
        assert!(passed);
    }

    #[test]
    fn test_heads_up_both_with_hole_cards() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();
        let tolerance = 0.1;

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("7h 6s").as_slice());

        analyzer.set_player_cards(3, card_u8s_from_string("Th 9h").as_slice());

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice());

        let num_it = 10_000;
        analyzer.simulate_flop(num_it).unwrap();

        let results = analyzer
            .player_info
            .iter()
            .map(|p| &p.results)
            .collect_vec();

        assert_eq!(results[0].num_iterations, num_it);
        let not_folded = results[0].num_iterations;

        assert_equity(
            100.0 * results[0].win_eq / not_folded as f64,
            21.92,
            tolerance,
        );

        assert_eq!(results[3].num_iterations, num_it);
        let not_folded = results[3].num_iterations;

        assert_equity(
            100.0 * results[3].win_eq / not_folded as f64,
            78.08,
            tolerance,
        );
    }

    #[test]
    fn test_3way_with_ranges() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseRange as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("8d 7s").as_slice());

        analyzer.set_player_cards(3, card_u8s_from_string("Qd 5c").as_slice());

        analyzer.set_player_range(
            2,
            "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o",
        );

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice());

        let num_it = 400_000;

        let tolerance = 0.1;

        analyzer.simulate_flop(num_it).unwrap();

        let results = analyzer.get_results();

        assert_eq!(results[0].num_iterations, num_it);

        assert_equity(
            100.0 * results[0].win_eq / results[0].num_iterations as f64,
            21.03,
            tolerance,
        );
        assert_equity(
            100.0 * results[0].tie_eq / results[0].num_iterations as f64,
            0.12,
            0.05,
        );

        assert_eq!(results[3].num_iterations, num_it);

        // assert_equity(
        //     100.0 * results[3].eq_not_folded / not_folded as f64,
        //     50.93 + 0.82,
        //     0.7,
        // );
        assert_equity(
            100.0 * results[3].win_eq / results[3].num_iterations as f64,
            50.93,
            tolerance,
        );
        assert_equity(
            100.0 * results[3].tie_eq / results[3].num_iterations as f64,
            0.82,
            tolerance,
        );

        assert_eq!(results[2].num_iterations, num_it);
        //let not_folded = results[3].num_iterations;

        assert_equity(
            100.0 * results[2].win_eq / results[2].num_iterations as f64,
            26.14,
            tolerance,
        );
        assert_equity(
            100.0 * results[2].tie_eq / results[2].num_iterations as f64,
            0.95,
            tolerance,
        );
    }
}
