use itertools::Itertools;

use crate::{Card, InRangeType, add_cards_from_string, range_string_to_set, CardUsedType};

extern crate wasm_bindgen;

type ResultType = u32;

#[wasm_bindgen]
#[derive(Default)]
pub struct Results {
    num_iterations: ResultType,
    folded: ResultType,
    
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
            villian_results: Results::default(),
        }
    }

    pub fn set_board_cards(&mut self, card_str: &str ) {
        self.cards.clear();
        add_cards_from_string(&mut self.cards, card_str);
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
                cards_used[c.to_range_index_part()] = true;
            }

            let mut deck: Vec<Card> = Vec::with_capacity(52);
            for c in 0..52 {
                if !cards_used[c] {
                    deck.push(Card::from_range_index(c));
                }
            }
            let (player_cards, _) = deck.partial_shuffle(&mut rng, 2 * n_players);

            for p_idx in 0..n_players {
                self.cards.push(player_cards[2 * p_idx]);
                self.cards.push(player_cards[2 * p_idx + 1]);

                let rank = rank_cards(&self.cards);

                let mut results = &mut self.player_results[p_idx];
                results.num_iterations += 1;

                //Did the player fold?
                let range_index = core_cards_to_range_index(player_cards[2 * p_idx], player_cards[2 * p_idx+1]);
                if !self.player_range_sets[p_idx][range_index] {
                    results.folded += 1;
                    self.cards.pop();
                    self.cards.pop();
                    continue;
                }

                self.cards.pop();
                self.cards.pop();
            }
        }
    }
}

