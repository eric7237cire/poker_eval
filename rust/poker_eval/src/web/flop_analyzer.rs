use itertools::Itertools;
use rand::{thread_rng, seq::SliceRandom};

use crate::{Card, InRangeType, add_cards_from_string, range_string_to_set, CardUsedType, rank_cards, core_cards_to_range_index, cards_from_string, Rank};
use wasm_bindgen::prelude::wasm_bindgen;
//extern crate wasm_bindgen;

type ResultType = u32;

#[wasm_bindgen]
#[derive(Default)]
pub struct Results {
    num_iterations: ResultType,
    folded: ResultType,

    //not folded = num_iterations - folded
    //win = 1, tie = 1 / num players in tie, loss = 0
    eq_not_folded: f64,
    
    //count made hands
    num_hi_card: u32,

    num_gut_shots: ResultType,

    //2 cards to straight
    num_str8_draw: ResultType,

    num_flush_draw: ResultType,

    num_pair: ResultType,
    num_top_pair: ResultType,

    num_two_pair: ResultType,
    num_trips: ResultType,
    num_str8: ResultType,
    num_flush: ResultType,
    num_full_house: ResultType,

    num_quads: ResultType,
    num_str8_flush: ResultType,
    
}

#[wasm_bindgen]
pub struct FlopAnalyzer {
    cards: Vec<Card>,
    player_range_strings: Vec<String>,
    player_range_sets: Vec<InRangeType>,

    player_results: Vec<Results>,
    player_cards: Vec<Option<[Card;2]>>,

    villian_results: Results,
    
}

//hero is 0
const MAX_PLAYERS: usize = 5;

#[wasm_bindgen]
impl FlopAnalyzer {
    pub fn new() -> Self {
        Self {
            cards: Vec::with_capacity(7),
            player_range_strings: Vec::with_capacity(MAX_PLAYERS),
            player_range_sets: Vec::with_capacity(MAX_PLAYERS),
            player_results: Vec::with_capacity(MAX_PLAYERS),

            player_cards: Vec::with_capacity(MAX_PLAYERS),

            villian_results: Results::default(),
        }
    }

    pub fn set_board_cards(&mut self, card_str: &str ) {
        self.cards.clear();
        add_cards_from_string(&mut self.cards, card_str);
    }

    pub fn set_player_cards(&mut self, player_idx: usize, card_str: &str) {
        let hole_cards = cards_from_string(card_str);
        
        assert_eq!(2, hole_cards.len());
        
        self.player_cards[player_idx] = Some([hole_cards[0], hole_cards[1]]);
    }

    pub fn clear_player_cards(&mut self, player_idx: usize) {
        self.player_cards[player_idx] = None;
    }

    pub fn reset(&mut self, num_players: usize, player_ranges: Vec<String>) {
        self.cards.clear();
        self.player_range_strings.clear();
        self.player_range_sets.clear();
        self.player_results.clear();
        self.villian_results = Results::default();

        for ply_range in player_ranges.iter() {
            self.player_range_strings.push(ply_range.clone());
            self.player_range_sets.push(range_string_to_set(&ply_range));
            self.player_results.push(Results::default());
        }
    }

    pub fn simulate_flop(&mut self, num_iterations: u32) {

        let n_players = self.player_results.len();
        let mut rng = thread_rng();

        for _ in 0..num_iterations {
            let mut cards_used = CardUsedType::default();
            for c in self.cards.iter() {
                cards_used.set(c.to_range_index_part(),  true);
            }

            for p_idx in 0..n_players {
                if let Some(hole_cards) = self.player_cards[p_idx] {
                    cards_used.set(hole_cards[0].to_range_index_part(), true);
                    cards_used.set(hole_cards[1].to_range_index_part(), true);
                }
            }

            let mut deck: Vec<Card> = Vec::with_capacity(52);
            for c in 0..52 {
                if !cards_used[c] {
                    deck.push(Card::from_range_index_part(c));
                }
            }

            //Even if we set some cards for players, for simplicity get enough for everyone ( 2 * n_players ) + turn & river
            let (player_cards, _) = deck.partial_shuffle(&mut rng, 2+2 * n_players);

            let mut hand_evals: Vec<Option<Rank>> = vec![None; n_players];

            //Do just river for now
            self.cards.push(player_cards[0]);
            self.cards.push(player_cards[1]);
            
            for p_idx in 0..n_players {
                if let Some(hole_cards) = self.player_cards[p_idx] {
                    self.cards.push(hole_cards[0]);
                    self.cards.push(hole_cards[1]);
                } else {
                    self.cards.push(player_cards[2+2 * p_idx]);
                    self.cards.push(player_cards[2+2 * p_idx + 1]);
                }

                let results = &mut self.player_results[p_idx];
                results.num_iterations += 1;

                //Did the player fold?
                let range_index = core_cards_to_range_index(player_cards[2 * p_idx], player_cards[2 * p_idx+1]);
                if !self.player_range_sets[p_idx][range_index] {
                    results.folded += 1;
                    self.cards.pop();
                    self.cards.pop();
                    hand_evals[p_idx] = None;
                    continue;
                }

                let rank = rank_cards(&self.cards);

                update_results_from_rank(results, rank);

                hand_evals[p_idx] = Some(rank);

                self.cards.pop();
                self.cards.pop();
            }

            let (winner_indexes, _num_non_folded) = indices_of_max_values(&hand_evals);

            for winner_idx in winner_indexes.iter() {
                let results = &mut self.player_results[*winner_idx];
                results.eq_not_folded += 1.0 / winner_indexes.len() as f64;
            }

            //pop turn & river
            self.cards.pop();
            self.cards.pop();

        }
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