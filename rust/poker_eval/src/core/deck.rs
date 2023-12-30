use std::ops::BitAnd;

use log::trace;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{BoolRange, Card, CardUsedType, HoleCards, PokerError, InRangeType, ALL_CARD_RANGES, ALL_HOLE_CARDS};

pub struct Deck {
    rng: StdRng,
    used_cards: CardUsedType, 

    available_range: BoolRange,
}

const MAX_RAND_NUMBER_ATTEMPS: usize = 10_000;

impl Deck {
    pub fn new() -> Self {
        let rng = StdRng::seed_from_u64(42);
        
        let mut d = Deck {
            rng,
            used_cards: CardUsedType::default(),
            available_range: BoolRange::default(),
        };

        d.reset();

        d
    }

    pub fn reset(&mut self) {
        self.used_cards = CardUsedType::default();
        self.available_range.data.fill(true);
    }

    pub fn get_board(&mut self) -> Vec<Card> {
        let mut board = Vec::new();
        for _ in 0..5 {
            let card = self.get_unused_card().unwrap();
            board.push(Card::try_from(card).unwrap());
        }
        board
    }

    

    pub fn set_used_card(&mut self, card: Card) {
        let card_index: usize = card.into();
        let count_before = self.used_cards.count_ones();
        self.used_cards.set(card_index, true);
        
        self.available_range.data &= &ALL_CARD_RANGES[card_index].inverse.data;

        let count_after = self.used_cards.count_ones();
        assert_eq!(count_after, count_before + 1);
    }

    pub fn choose_available_in_range(
        &mut self,
        range: &BoolRange,
    ) -> Result<HoleCards, PokerError> {
        
        let possible_range = range.data & self.available_range.data;

        if possible_range.data.is_empty() {
            return Err(PokerError::from_string(format!(
                "No available hole cards in range {}",
                range.data.count_ones()
            )));
        }

        let num_possible = possible_range.count_ones();
        
        let rand_int: usize = self.rng.gen_range(0..num_possible);

        let hole_position = possible_range.iter_ones().skip(rand_int).take(1).next().unwrap();

        let hole_cards = ALL_HOLE_CARDS[hole_position];

        self.set_used_card(hole_cards.get_hi_card());
        self.set_used_card(hole_cards.get_lo_card());

        Ok(hole_cards)
    }

    pub fn get_unused_card(&mut self) -> Result<Card, PokerError> {
        //Usually most of the deck is available
        let mut attempts = 0;
        loop {
            let rand_int: usize = self.rng.gen_range(0..52);
            assert!(rand_int < 52);
            //let card = Card::from(rand_int);
            if !self.used_cards[rand_int] {
                let card: Card = rand_int.try_into()?;
                self.set_used_card(card);
                
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


#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng, seq::SliceRandom};

    use crate::{pre_calc::NUMBER_OF_HOLE_CARDS, init_test_logger, CardValue};

    use super::*;

    #[test]
    fn test_choose_available_in_range_aces() {
        init_test_logger();
        let mut deck = Deck::new();
        let iter_count = 1_000;
        let mut total = vec![0i32; NUMBER_OF_HOLE_CARDS];

        let just_aces: BoolRange = "AA".parse().unwrap();

        for _ in 0..iter_count {
            deck.reset();
            
            let hole_cards = deck.choose_available_in_range(&just_aces).unwrap();
            total[hole_cards.to_range_index()] += 1;
        }

        let hc_aa : HoleCards = "AsAc".parse().unwrap();
        let perc = total[hc_aa.to_range_index()] as f64 / iter_count as f64;
        trace!("Perc {:.2} vs {:.2}", perc*100.0, 100f64/6.0);
        //within 2%
        assert!( (perc - 1f64 / 6.0).abs() < 0.02);

        let mut total = vec![0i32; NUMBER_OF_HOLE_CARDS];
        let ace_clubs : Card = "Ah".parse().unwrap();
        for _ in 0..iter_count {
            deck.reset();

            deck.set_used_card(ace_clubs);
            
            let hole_cards = deck.choose_available_in_range(&just_aces).unwrap();
            total[hole_cards.to_range_index()] += 1;
        }

        let hc_aa : HoleCards = "AsAc".parse().unwrap();
        let perc = total[hc_aa.to_range_index()] as f64 / iter_count as f64;
        trace!("Perc {:.2} vs {:.2}", perc*100.0, 100f64/3.0);
        //within 2%
        assert!( (perc - 1f64 / 3.0).abs() < 0.02);

    }

    #[test]
    fn test_choose_available_larger_range() {
        //AT-A2,KT-K2,QT-Q2,J2+

        init_test_logger();
        let mut deck = Deck::new();
        let iter_count = 1_000;
        let mut total = vec![0i32; NUMBER_OF_HOLE_CARDS];

        let with_tens: BoolRange = "AT-A2,KT-K2,QT-Q2,J2+".parse().unwrap();

        for _ in 0..iter_count {
            deck.reset();
            
            let hole_cards = deck.choose_available_in_range(&with_tens).unwrap();
            assert!(with_tens.data[hole_cards.to_range_index()]);

            deck.reset();

            deck.set_used_card("5c".parse().unwrap());
            deck.set_used_card("5d".parse().unwrap());
            deck.set_used_card("5h".parse().unwrap());
            deck.set_used_card("5s".parse().unwrap());
            
            let hole_cards = deck.choose_available_in_range(&with_tens).unwrap();
            assert!(with_tens.data[hole_cards.to_range_index()]);

            assert!(hole_cards.get_lo_card().value != CardValue::Five);
            //assert!(hole_cards.get_lo_card().value != CardValue::Four);
        }

        
    }
}
