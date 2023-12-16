
use std::{cmp, mem};

use log::{info, error, trace, warn, debug};
use rand::{thread_rng, seq::SliceRandom};

use crate::{Card, InRangeType, add_cards_from_string, range_string_to_set, CardUsedType, rank_cards, core_cards_to_range_index, cards_from_string, Rank};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
//extern crate wasm_bindgen;
//extern crate console_error_panic_hook;
type ResultType = u32;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Default, Clone, Serialize)]
pub struct Results {
    pub num_iterations: ResultType,
    pub  folded: ResultType,

    //not folded = num_iterations - folded
    //win = 1, tie = 1 / num players in tie, loss = 0
    pub eq_not_folded: f64,
    
    //count made hands
    num_hi_card: u32,

    num_gut_shots: ResultType,

    //2 cards to straight
    num_str8_draw: ResultType,

    num_flush_draw: ResultType,
    num_top_pair: ResultType,

    pub num_pair: ResultType,
    pub num_two_pair: ResultType,
    pub num_trips: ResultType,
    pub num_str8: ResultType,
    pub num_flush: ResultType,
    pub num_full_house: ResultType,
    pub num_quads: ResultType,
    pub num_str8_flush: ResultType,
    
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
#[derive(Default, Clone)]
pub struct PreflopPlayerInfo {
    //range_string: String,
    results: Results,
    hole_cards: Vec<Card>,
    range_set: InRangeType,
    state: PlayerPreFlopState
}

#[wasm_bindgen]
pub struct flop_analyzer {
    board_cards: Vec<Card>,
    player_info: Vec<PreflopPlayerInfo>,
    villian_results: Results,    

    pub num_iterations: u32,
}

//hero is 0
const MAX_PLAYERS: usize = 5;

#[wasm_bindgen]
impl flop_analyzer {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());

        info!("FlopAnalyzer::new()"	);
        debug!("debug");
        warn!("warn");
        trace!("trace");
        error!("error");

        info!("Initializing FlopAnalyzer ");
       

        Self {
            board_cards: Vec::with_capacity(7),
            player_info: Vec::with_capacity(MAX_PLAYERS),            

            villian_results: Results::default(),

            num_iterations:0,
        }
    }

    pub fn set_board_cards(&mut self, cards: &[u8] ) {
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
        let pc = &mut self.player_info[player_idx].hole_cards ;
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
        self.player_info[player_idx].range_set = range_set;
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
            self.player_info.push(PreflopPlayerInfo::default());
        }
        self.villian_results = Results::default();      

        self.num_iterations = 42;  
    }


    pub fn simulate_flop(&mut self, num_iterations: u32) {

        let n_players = self.player_info.len();
        let mut rng = thread_rng();

        for it_num in 0..num_iterations {

            debug!("simulate_flop: iteration {}", it_num);

            let mut deck = self.prepare_deck();

            //Even if we set some cards for players, for simplicity get enough for everyone ( 2 * n_players ) + turn & river
            let (shuffled_cards, _) = deck.partial_shuffle(&mut rng, 2+2 * n_players);

            let mut hand_evals: Vec<Option<Rank>> = vec![None; n_players];

            //Do just river for now
            self.board_cards.push(shuffled_cards[0]);
            self.board_cards.push(shuffled_cards[1]);
            
            for p_idx in 0..n_players {
                if self.player_info[p_idx].state == PlayerPreFlopState::Disabled {
                    continue;
                }

                if self.player_info[p_idx].state == PlayerPreFlopState::UseHoleCards {
                    self.board_cards.push(self.player_info[p_idx].hole_cards[0]);
                    self.board_cards.push(self.player_info[p_idx].hole_cards[1]);
                } else {
                    self.board_cards.push(shuffled_cards[2+2 * p_idx]);
                    self.board_cards.push(shuffled_cards[2+2 * p_idx + 1]);
                }

                {
                    let results = &mut self.player_info[p_idx].results;
                    results.num_iterations += 1;
                }

                //Did the player fold?
                if self.player_info[p_idx].state == PlayerPreFlopState::UseRange {
                let range_index = core_cards_to_range_index(shuffled_cards[2 * p_idx], shuffled_cards[2 * p_idx+1]);
                if !self.player_info[p_idx].range_set[range_index] {
                    {
                        let results = &mut self.player_info[p_idx].results;
                        results.folded += 1;
                    }
                    self.board_cards.pop();
                    self.board_cards.pop();
                    hand_evals[p_idx] = None;
                    continue;
                }
            }

                let rank = rank_cards(&self.board_cards);

                let results = &mut self.player_info[p_idx].results;
                update_results_from_rank(results, rank);

                hand_evals[p_idx] = Some(rank);

                self.board_cards.pop();
                self.board_cards.pop();
            }

            let (winner_indexes, _num_non_folded) = indices_of_max_values(&hand_evals);

            for winner_idx in winner_indexes.iter() {
                let results = &mut self.player_info[*winner_idx].results;
                results.eq_not_folded += 1.0 / winner_indexes.len() as f64;
            }

            //pop turn & river
            self.board_cards.pop();
            self.board_cards.pop();

        }
    }

    fn prepare_deck(&self) -> Vec<Card> {

        let mut cards_used = CardUsedType::default();
            for c in self.board_cards.iter() {
                cards_used.set(c.to_range_index_part(),  true);
            }

            for p_idx in 0..self.player_info.len() {
                for c in self.player_info[p_idx].hole_cards.iter() {
                    cards_used.set(c.to_range_index_part(), true);
                    
                }
            }

            let mut deck: Vec<Card> = Vec::with_capacity(52);
            for c in 0..52 {
                if !cards_used[c] {
                    deck.push(Card::from_range_index_part(c));
                }
            }
            deck
    }

    pub fn get_results(&self) -> Vec<Results> {
        self.player_info.iter().map(|p| p.results.clone()).collect()
    }

    pub fn get_result(&self, player_idx: usize) -> Result<JsValue, JsValue> {
        info!("get_result: {} num iterations {}", player_idx, self.player_info[player_idx].results.num_iterations);
       
        Ok(serde_wasm_bindgen::to_value(&&self.player_info[player_idx].results)?)
    }
}



fn update_results_from_rank(results: &mut Results, rank: Rank) {
    match rank {
        Rank::HighCard(_) => results.num_hi_card += 1,
        Rank::OnePair(_) => results.num_pair += 1,
        Rank::TwoPair(_) => results.num_two_pair += 1,
        Rank::ThreeOfAKind(_) => results.num_trips += 1,
        Rank::Straight(_) => results.num_str8 += 1,
        Rank::Flush(_) => results.num_flush += 1,
        Rank::FullHouse(_) => results.num_full_house += 1,
        Rank::FourOfAKind(_) => results.num_quads += 1,
        Rank::StraightFlush(_) => results.num_str8_flush += 1,
    }
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