use std::{
    fmt::{Display, Formatter},
    iter::{once, Chain, Once},
    str::FromStr,
};

use once_cell::sync::Lazy;
use serde::Serialize;

use crate::{pre_calc::NUMBER_OF_HOLE_CARDS, set_used_card, unset_used_card, CardUsedType};

use crate::{Card, PokerError};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize)]
pub struct HoleCards {
    card_hi_lo: [Card; 2],
    //card_lo: Card
}

pub const SIMPLE_RANGE_INDEX_LEN: usize = 169;

impl HoleCards {
    pub fn new(card1: Card, card2: Card) -> Result<Self, PokerError> {
        let card1_index: u8 = card1.into();
        let card2_index: u8 = card2.into();

        if card1_index == card2_index {
            return Err(PokerError::from_str("Hole cards must be different"));
        }

        let card_hi = if card1_index > card2_index {
            card1
        } else {
            card2
        };
        let card_lo = if card1_index > card2_index {
            card2
        } else {
            card1
        };

        Ok(HoleCards {
            card_hi_lo: [card_hi, card_lo],
        })
    }

    //This converts our exact hole cards to the range index from 0 to 52*51/2
    pub fn to_range_index(&self) -> usize {
        let lo_card: usize = self.card_hi_lo[1].into();
        let hi_card: usize = self.card_hi_lo[0].into();

        //Uses sum formula for how many cards come before it
        //card2 > card1
        lo_card as usize * (101 - lo_card as usize) / 2 + hi_card as usize - 1

        //assert_eq!(ret, card_pair_to_index(hi_card as u8, lo_card as u8));

        //ret
    }

    //This is to convert to the range index from 0 to 169
    // row 0, col 0 is AA
    // row 0, col 1 is AKs
    // row 1, col 0 is AKo
    pub fn to_simple_range_index(&self) -> usize {
        //suited
        if self.card_hi_lo[0].suit == self.card_hi_lo[1].suit {
            //ace is first row, 2 is last row
            let row = 12 - self.card_hi_lo[0].value as usize;

            let col = 12 - self.card_hi_lo[1].value as usize;

            return row * 13 + col;
        }

        //not suited

        //ace is first col, 2 is last col
        let col = 12 - self.card_hi_lo[0].value as usize;
        //ace is first row, 2 is last row
        let row = 12 - self.card_hi_lo[1].value as usize;

        return row * 13 + col;
    }

    pub fn to_simple_range_string(&self) -> String {
        let mut s = String::new();
        s.push(char::from(self.card_hi_lo[0].value));
        s.push(char::from(self.card_hi_lo[1].value));
        if self.card_hi_lo[0].value != self.card_hi_lo[1].value {
            s.push(char::from(
                if self.card_hi_lo[0].suit == self.card_hi_lo[1].suit {
                    's'
                } else {
                    'o'
                }));
        }
        s
    }

    pub fn get_hi_card(&self) -> Card {
        assert!(self.card_hi_lo[0].value >= self.card_hi_lo[1].value);
        self.card_hi_lo[0]
    }

    pub fn get_lo_card(&self) -> Card {
        assert!(self.card_hi_lo[0].value >= self.card_hi_lo[1].value);
        self.card_hi_lo[1]
    }

    pub fn is_pocket_pair(&self) -> bool {
        self.card_hi_lo[0].value == self.card_hi_lo[1].value
    }

    pub fn as_slice(&self) -> &[Card] {
        &self.card_hi_lo
    }

    pub fn to_range_string(&self) -> String {
        if self.card_hi_lo[0].value == self.card_hi_lo[1].value {
            return format!("{}{}", self.card_hi_lo[0].value, self.card_hi_lo[1].value);
        }

        if self.card_hi_lo[0].suit == self.card_hi_lo[1].suit {
            return format!("{}{}s", self.card_hi_lo[0].value, self.card_hi_lo[1].value);
        }

        format!("{}{}o", self.card_hi_lo[0].value, self.card_hi_lo[1].value)
    }

    pub fn set_used(&self, cards_used: &mut CardUsedType) -> Result<(), PokerError> {
        set_used_card(self.card_hi_lo[0].into(), cards_used)?;
        set_used_card(self.card_hi_lo[1].into(), cards_used)?;
        Ok(())
    }

    pub fn unset_used(&self, cards_used: &mut CardUsedType) -> Result<(), PokerError> {
        unset_used_card(self.card_hi_lo[0].into(), cards_used)?;
        unset_used_card(self.card_hi_lo[1].into(), cards_used)?;
        Ok(())
    }

    pub fn get_iter(&self) -> Chain<Once<Card>, Once<Card>> {
        once(self.get_hi_card()).chain(once(self.get_lo_card()))
    }

    pub fn add_to_eval(&self, eval_cards: &mut Vec<Card>) {
        eval_cards.push(self.card_hi_lo[0].into());
        eval_cards.push(self.card_hi_lo[1].into());
    }

    pub fn remove_from_eval(&self, eval_cards: &mut Vec<Card>) -> Result<(), PokerError> {
        let c1 = eval_cards
            .pop()
            .ok_or(PokerError::from_str("No cards to remove"))?;
        let c2 = eval_cards
            .pop()
            .ok_or(PokerError::from_str("No cards to remove"))?;

        if c2 != self.card_hi_lo[0].into() || c1 != self.card_hi_lo[1].into() {
            return Err(PokerError::from_str(
                "Cards to remove do not match hole cards",
            ));
        }

        Ok(())
    }
}

impl FromStr for HoleCards {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().filter(|c| !c.is_whitespace());

        let v1 = chars
            .next()
            .ok_or(PokerError::from_string(format!("Need another charecter")))?;
        let s1 = chars
            .next()
            .ok_or(PokerError::from_string(format!("Need another charecter")))?;
        let v2 = chars
            .next()
            .ok_or(PokerError::from_string(format!("Need another charecter")))?;
        let s2 = chars
            .next()
            .ok_or(PokerError::from_string(format!("Need another charecter")))?;

        Ok(HoleCards::new(
            Card::new(v1.try_into()?, s1.try_into()?),
            Card::new(v2.try_into()?, s2.try_into()?),
        )?)
    }
}

impl Display for HoleCards {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.card_hi_lo[0], self.card_hi_lo[1])
    }
}

pub static ALL_HOLE_CARDS: Lazy<Vec<HoleCards>> = Lazy::new(|| {
    let mut vec_hole_cards = Vec::with_capacity(NUMBER_OF_HOLE_CARDS);
    for card1 in 0..52u8 {
        for card2 in card1 + 1..52 {
            let hc = HoleCards::new(
                Card::try_from(card1).unwrap(),
                Card::try_from(card2).unwrap(),
            )
            .unwrap();
            vec_hole_cards.push(hc);
        }
    }
    assert_eq!(vec_hole_cards.len(), NUMBER_OF_HOLE_CARDS);
    vec_hole_cards
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_range_string() {
        assert_eq!(
            HoleCards::new("As".parse().unwrap(), "Ac".parse().unwrap())
                .unwrap()
                .to_range_string(),
            "AA"
        );

        assert_eq!(
            HoleCards::new("As".parse().unwrap(), "Ks".parse().unwrap())
                .unwrap()
                .to_range_string(),
            "AKs"
        );

        assert_eq!(
            HoleCards::new("2c".parse().unwrap(), "7c".parse().unwrap())
                .unwrap()
                .to_range_string(),
            "72s"
        );

        assert_eq!(
            HoleCards::new("Th".parse().unwrap(), "9h".parse().unwrap())
                .unwrap()
                .to_range_string(),
            "T9s"
        );

        assert_eq!(
            HoleCards::new("8d".parse().unwrap(), "9h".parse().unwrap())
                .unwrap()
                .to_range_string(),
            "98o"
        );
    }

    #[test]
    fn test_simplified_range_index() {
        assert_eq!(
            HoleCards::new("Ac".parse().unwrap(), "Ad".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            0
        );

        assert_eq!(
            HoleCards::new("2c".parse().unwrap(), "Ac".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            12
        );

        assert_eq!(
            HoleCards::new("Kc".parse().unwrap(), "Ad".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            13
        );

        assert_eq!(
            HoleCards::new("Kd".parse().unwrap(), "3d".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            24
        );

        assert_eq!(
            HoleCards::new("2c".parse().unwrap(), "2d".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            168
        );

        assert_eq!(
            HoleCards::new("7c".parse().unwrap(), "2d".parse().unwrap())
                .unwrap()
                .to_simple_range_index(),
            163
        );
    }

    #[test]
    fn test_all_hole_cards() {
        let mut index_check = 0;
        for card1 in 0..52u8 {
            for card2 in card1 + 1..52 {
                let hc = HoleCards::new(
                    Card::try_from(card1).unwrap(),
                    Card::try_from(card2).unwrap(),
                )
                .unwrap();

                assert_eq!(hc.to_range_index(), index_check);
                index_check += 1;

                assert_eq!(ALL_HOLE_CARDS[hc.to_range_index()], hc);
            }
        }
    }
}
