use log::trace;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{BoolRange, Card, CardUsedType, HoleCards, PokerError};

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

    pub fn set_used_card(&mut self, card: Card) {
        let card_index: usize = card.into();
        let count_before = self.used_cards.count_ones();
        self.used_cards.set(card_index, true);
        let count_after = self.used_cards.count_ones();
        assert_eq!(count_after, count_before + 1);
    }

    pub fn choose_available_in_range(
        &mut self,
        range: &BoolRange,
    ) -> Result<HoleCards, PokerError> {
        let _out_range = BoolRange::default();
        //there should be a better way to do this
        //perhaps each card has the bit set for the range it is in

        let mut possible_hole_cards = Vec::new();

        for card1_usize in 0..52usize {
            for card2_usize in card1_usize + 1..52usize {
                let card1 = Card::try_from(card1_usize).unwrap();
                let card2 = Card::try_from(card2_usize).unwrap();
                let hc = crate::HoleCards::new(card1, card2).unwrap();
                let hc_index = hc.to_range_index();

                //Not in original range
                if !range.data[hc_index] {
                    continue;
                }

                //Used already
                if self.used_cards[card1_usize] || self.used_cards[card2_usize] {
                    continue;
                }

                //out_range.data.set(hc_index, true);
                possible_hole_cards.push(hc);
            }
        }

        trace!("possible_hole_cards {}", possible_hole_cards.len());

        if possible_hole_cards.is_empty() {
            return Err(PokerError::from_string(format!(
                "No available hole cards in range {}",
                range.data.count_ones()
            )));
        }

        let rand_int: usize = self.rng.gen_range(0..possible_hole_cards.len());

        let hole_cards = possible_hole_cards[rand_int];

        self.set_used_card(hole_cards.get_hi_card());
        self.set_used_card(hole_cards.get_lo_card());

        Ok(hole_cards)
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
                )
                .into());
            }
        }
    }
}
