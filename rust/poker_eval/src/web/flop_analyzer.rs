extern crate wasm_bindgen;

type ResultType = u32;

#[wasm_bindgen]
pub struct Results {
    num_iterations: ResultType,
    
    //count made hands
    num_hi_card: u32

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
    playerRangesStrings: Vec<String>,
    //playerRangeSet
}

//hero is 0
const MAX_PLAYERS = 5;

#[wasm_bindgen]
impl FlopAnalyzer {
    pub fn new() -> Self {
        Self {
            cards: Vec::with_capacity(7),
            range_strings: 
        }
    }

    pub fn set_board_cards(&mut self, card_str: &str ) {
        self.cards.clear();
        add_cards_from_string(&mut self.cards, cards);
    }
}

