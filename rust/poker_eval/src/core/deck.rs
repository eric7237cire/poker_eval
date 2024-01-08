use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{Card, CardUsedType, HoleCards, PokerError, ALL_CARDS};

pub struct Deck {
    pub(crate) rng: StdRng,
    pub(crate) used_cards: CardUsedType,
    //available_range: BoolRange,
}

const MAX_RAND_NUMBER_ATTEMPS: usize = 1_000;

impl Deck {
    pub fn new() -> Self {
        let rng = StdRng::seed_from_u64(42);
        //fastrand::seed(42);

        let mut d = Deck {
            rng,
            used_cards: CardUsedType::default(),
            // available_range: BoolRange::default(),
        };

        d.reset();

        d
    }

    pub fn reset(&mut self) {
        self.used_cards = CardUsedType::default();
        //self.available_range.data.fill(true);
    }

    pub fn get_number_of_used_cards(&self) -> usize {
        self.used_cards.count_ones() as usize
    }

    pub fn is_used(&self, card: Card) -> bool {
        self.used_cards[card.index as usize]
    }

    pub fn choose_new_board(&mut self) -> Vec<Card> {
        let mut board = Vec::with_capacity(5);
        for _ in 0..5 {
            let card = self.get_unused_card().unwrap();
            board.push(Card::try_from(card).unwrap());
        }
        board
    }

    pub fn set_used_card(&mut self, card: Card) {
        assert!(!self.used_cards[card.index as usize]);
        //let count_before = self.used_cards.count_ones();
        self.used_cards.set(card.index as usize, true);

        //self.available_range.data &= &ALL_CARD_RANGES[card.index].inverse.data;

        //let count_after = self.used_cards.count_ones();
        //assert_eq!(count_after, count_before + 1);
    }

    /*
    Chooses from the list, retrying if those cards are already used
    Sets the used cards in the deck
    */
    pub fn choose_available_in_range(
        &mut self,
        possible_hole_cards: &Vec<HoleCards>,
    ) -> Result<HoleCards, PokerError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            //trace!("Attempt {}", attempts);
            //let rand_int: usize = fastrand::usize(0..possible_hole_cards.len());
            let rand_int: usize = self.rng.gen_range(0..possible_hole_cards.len());
            let hole_cards = possible_hole_cards[rand_int];

            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return Err(format!(
                    "Unable to find unused card after {} attempts, # of used cards {}",
                    attempts,
                    self.used_cards.count_ones()
                )
                .into());
            }

            if self.used_cards[hole_cards.get_hi_card().index as usize] {
                continue;
            }
            if self.used_cards[hole_cards.get_lo_card().index as usize] {
                continue;
            }

            self.set_used_card(hole_cards.get_hi_card());
            self.set_used_card(hole_cards.get_lo_card());
            return Ok(hole_cards);
        }
    }

    pub fn get_unused_card(&mut self) -> Result<Card, PokerError> {
        //Usually most of the deck is available
        let mut attempts = 0;
        loop {
            let rand_int: usize = self.rng.gen_range(0..52);
            //let rand_int: usize = fastrand::usize(0..52);
            assert!(rand_int < 52);

            attempts += 1;
            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return Err(format!(
                    "Unable to find unused card after {} attempts, # of used cards {}",
                    attempts,
                    self.used_cards.count_ones()
                )
                .into());
            }

            //let card = Card::from(rand_int);
            if !self.used_cards[rand_int] {
                let card: Card = ALL_CARDS[rand_int];
                self.set_used_card(card);

                return Ok(card);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_choose_available_in_range_aces() {
    //     init_test_logger();
    //     let mut deck = Deck::new();
    //     let iter_count = 1_000;
    //     let mut total = vec![0i32; NUMBER_OF_HOLE_CARDS];

    //     let just_aces: BoolRange = "AA".parse().unwrap();

    //     for _ in 0..iter_count {
    //         deck.reset();

    //         let hole_cards = deck.choose_available_in_range(&just_aces).unwrap();
    //         total[hole_cards.to_range_index()] += 1;
    //     }

    //     let hc_aa : HoleCards = "AsAc".parse().unwrap();
    //     let perc = total[hc_aa.to_range_index()] as f64 / iter_count as f64;
    //     //trace!("Perc {:.2} vs {:.2}", perc*100.0, 100f64/6.0);
    //     //within 2%
    //     assert!( (perc - 1f64 / 6.0).abs() < 0.02);

    //     let mut total = vec![0i32; NUMBER_OF_HOLE_CARDS];
    //     let ace_clubs : Card = "Ah".parse().unwrap();
    //     for _ in 0..iter_count {
    //         deck.reset();

    //         deck.set_used_card(ace_clubs);

    //         let hole_cards = deck.choose_available_in_range(&just_aces).unwrap();
    //         total[hole_cards.to_range_index()] += 1;
    //     }

    //     let hc_aa : HoleCards = "AsAc".parse().unwrap();
    //     let perc = total[hc_aa.to_range_index()] as f64 / iter_count as f64;
    //     //trace!("Perc {:.2} vs {:.2}", perc*100.0, 100f64/3.0);
    //     //within 2%
    //     assert!( (perc - 1f64 / 3.0).abs() < 0.02);

    // }

    // #[test]
    // fn test_choose_available_larger_range() {
    //AT-A2,KT-K2,QT-Q2,J2+

    //     init_test_logger();
    //     let mut deck = Deck::new();
    //     let iter_count = 1_000;

    //     let with_tens: BoolRange = "AT-A2,KT-K2,QT-Q2,J2+".parse().unwrap();

    //     for _ in 0..iter_count {
    //         deck.reset();

    //         let hole_cards = deck.choose_available_in_range(&with_tens).unwrap();
    //         assert!(with_tens.data[hole_cards.to_range_index()]);

    //         deck.reset();

    //         deck.set_used_card("5c".parse().unwrap());
    //         deck.set_used_card("5d".parse().unwrap());
    //         deck.set_used_card("5h".parse().unwrap());
    //         deck.set_used_card("5s".parse().unwrap());

    //         let hole_cards = deck.choose_available_in_range(&with_tens).unwrap();
    //         assert!(with_tens.data[hole_cards.to_range_index()]);

    //         assert!(hole_cards.get_lo_card().value != CardValue::Five);
    //         //assert!(hole_cards.get_lo_card().value != CardValue::Four);
    //     }

    // }
}
