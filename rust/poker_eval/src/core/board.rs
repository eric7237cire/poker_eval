use core::fmt;
use std::{fmt::Display, iter::Cloned, slice::Iter, str::FromStr};

use num_integer::binomial;

use crate::game::core::Round;
use crate::{Card, Deck, HoleCards, PokerError};

pub struct Board {
    cards: Vec<Card>,

    //unique index for # of cards and which combination it represents
    index: Option<u32>,
}

impl TryFrom<&str> for Board {
    type Error = PokerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut cards = Vec::with_capacity(7);
        let mut chars = value.chars().filter(|c| c.is_alphanumeric());
        while let Some(c) = chars.next() {
            let value = c;
            let suit = chars.next().ok_or(PokerError::from_string(format!(
                "Unable to parse suit from {}",
                value
            )))?;
            cards.push(Card::new(value.try_into()?, suit.try_into()?));
        }
        Ok(Board::new_from_cards(&cards))
    }
}

impl FromStr for Board {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::try_from(s)
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            cards: Vec::with_capacity(7),
            index: None,
        }
    }

    pub fn new_from_cards(cards: &[Card]) -> Self {
        let mut s = Board::new();
        for card in cards {
            s.add_card(*card).unwrap();
        }
        s
    }

    pub fn add_card(&mut self, card: Card) -> Result<(), PokerError> {
        if self.cards.len() == 7 {
            return Err(PokerError::from_string(format!(
                "Unable to add card {} to board {}",
                card, self
            )));
        }

        self.cards.push(card);
        self.index = None;
        Ok(())
    }

    pub fn add_cards_from_deck(
        &mut self,
        deck: &mut Deck,
        num_cards: usize,
    ) -> Result<(), PokerError> {
        self.cards.clear();

        for _ in 0..num_cards {
            self.cards.push(deck.get_unused_card()?);
        }

        self.index = None;
        Ok(())
    }

    pub fn clear_cards(&mut self) {
        self.cards.clear();
        self.index = None;
    }

    pub fn get_precalc_index(&self) -> Result<u32, PokerError> {
        self.index.ok_or(PokerError::from_string(format!(
            "Index not calculated for board {}",
            self
        )))
    }

    //This is the u32 that uniquely identifies the board
    //27 bits (enough for 7 cards) 2^27=134_217_728 and 52 choose 7 = 133_784_560
    //bits 30-28 are the length
    pub fn get_index(&mut self) -> u32 {
        if let Some(index) = self.index {
            return index;
        }

        assert!(self.cards.len() <= 7);

        let mut sorted_cards = self.cards.clone();
        sorted_cards.sort();

        let mut index = 0;

        for i in 0..sorted_cards.len() {
            let num_possible_before: u8 = sorted_cards[i].into(); // 0 to card[i] - 1
            let dim = i + 1;
            let ncr = binomial(num_possible_before as u32, dim as u32);
            index += ncr;
        }

        let cards_len_bits = (sorted_cards.len() as u32) << 27;
        //should be 0 overlap
        assert!(cards_len_bits & index == 0);
        index += cards_len_bits;

        self.index = Some(index);

        index
    }

    pub fn as_slice_card(&self) -> &[Card] {
        &self.cards
    }

    pub fn as_vec_u8(&self) -> Vec<u8> {
        self.cards
            .iter()
            .map(|c| {
                let c_u8: u8 = (*c).into();
                c_u8
            })
            .collect()
    }

    pub fn get_iter(&self) -> Cloned<Iter<Card>> {
        self.cards.iter().cloned()
    }

    pub fn get_round(&self) -> Result<Round, PokerError> {
        Ok(match self.cards.len() {
            0 => Round::Preflop,
            3 => Round::Flop,
            4 => Round::Turn,
            5 => Round::River,
            _ => return Err(format!("Invalid number of cards {}", self.cards.len()).into()),
        })
    }

    pub fn get_num_cards(&self) -> usize {
        self.cards.len()
    }

    pub fn intersects_holecards(&self, hole_cards: &HoleCards) -> bool {
        for card in hole_cards.iter() {
            if self.cards.contains(&card) {
                return true;
            }
        }
        false
    }

    pub fn to_string_no_spaces(&self) -> String {
        let mut s = String::new();
        for card in self.cards.iter() {
            s.push_str(&format!("{}", card));
        }
        s
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for card in self.cards.iter() {
            s.push_str(&format!("{} ", card));
        }
        write!(f, "{}", s.trim())
    }
}
