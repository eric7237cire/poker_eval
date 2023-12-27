use log::warn;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{Card, CardUsedType, PokerError};

pub struct Deck {
    rng: StdRng,
    used_cards: CardUsedType, //= CardUsedType::default();
}

const MAX_RAND_NUMBER_ATTEMPS: usize = 10_000;

impl Deck {
    pub fn new() -> Self {
        let rng = StdRng::seed_from_u64(42);
        Deck {
            rng,
            used_cards: CardUsedType::default(),
        }
    }

    pub fn reset(&mut self) {
        self.used_cards = CardUsedType::default();
    }

    pub fn get_board(&mut self) -> Vec<Card> {
        let mut board = Vec::new();
        for _ in 0..5 {
            let card = self.get_unused_card().unwrap();
            board.push(Card::try_from(card).unwrap());
        }
        board
    }

    pub fn clear_used_card(&mut self, card: Card) {
        let card_index: usize = card.into();
        let count_before = self.used_cards.count_ones();
        self.used_cards.set(card_index, false);
        let count_after = self.used_cards.count_ones();
        assert_eq!(count_after + 1, count_before);
    }

    pub fn get_unused_card(&mut self) -> Result<Card, PokerError> {
        let mut attempts = 0;
        loop {
            let rand_int: usize = self.rng.gen_range(0..52);
            assert!(rand_int < 52);
            //let card = Card::from(rand_int);
            if !self.used_cards[rand_int] {
                self.used_cards.set(rand_int, true);
                return Ok(rand_int.try_into()?);
            }
            attempts += 1;
            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return Err(format!(
                    "Unable to find unused card after {} attempts, # of used cards {}",
                    attempts,
                    self.used_cards.count_ones()
                ).into());
            }
        }
    }
}
