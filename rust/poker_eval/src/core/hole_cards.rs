use crate::{CardUsedType, set_used_card};
use postflop_solver::card_pair_to_index;

use crate::{Card, PokerError};

pub struct HoleCards {
    card_hi: Card,
    card_lo: Card
}

impl HoleCards {
    pub fn new(card1: Card, card2: Card) -> Result<Self, PokerError> {

        let card1_index: u8 = card1.into();
        let card2_index: u8 = card2.into();

        if card1_index == card2_index {
            return Err(PokerError::from_str("Hole cards must be different"));
        }

        let card_hi = if card1_index > card2_index { card1 } else { card2 };
        let card_lo = if card1_index > card2_index { card2 } else { card1 };
        
        Ok(HoleCards {
            card_hi,
            card_lo
        })
    }

    pub fn to_range_index(&self) -> usize {
        card_pair_to_index(self.card_lo.into(), self.card_hi.into())
    }


    pub fn to_range_string(&self) -> String {

        if self.card_hi.value == self.card_lo.value {
            return format!("{}{}", self.card_hi.value, self.card_lo.value);
        }

        if self.card_hi.suit == self.card_lo.suit {
            return format!("{}{}s", self.card_hi.value, self.card_lo.value);
        }

        format!("{}{}o", self.card_hi.value, self.card_lo.value)
    }

    pub fn set_used(&self, cards_used: &mut CardUsedType) -> Result<(), PokerError> {
        set_used_card(self.card_hi.into(), cards_used)?;
        set_used_card(self.card_lo.into(), cards_used)?;
        Ok(())
    }

    pub fn add_to_eval(&self, eval_cards: &mut Vec<Card>)  {
        eval_cards.push(self.card_hi.into());
        eval_cards.push(self.card_lo.into());
    }

    pub fn remove_from_eval(&self, eval_cards: &mut Vec<Card>) -> Result<(), PokerError> {
        let c1 = eval_cards.pop().ok_or(PokerError::from_str("No cards to remove"))?;
        let c2 = eval_cards.pop().ok_or(PokerError::from_str("No cards to remove"))?;

        if c2 != self.card_hi.into() || c1 != self.card_lo.into() {
            return Err(PokerError::from_str("Cards to remove do not match hole cards"));
        }

        Ok(())
    }

    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_range_string() {
        assert_eq!(HoleCards::new("As".parse().unwrap(), 
        "Ac".parse().unwrap()).unwrap().to_range_string(), "AA");

        assert_eq!(HoleCards::new("As".parse().unwrap(),
        "Ks".parse().unwrap()).unwrap().to_range_string(), "AKs");

        assert_eq!(HoleCards::new("2c".parse().unwrap(),
        "7c".parse().unwrap()).unwrap().to_range_string(), "72s");

        assert_eq!(HoleCards::new("Th".parse().unwrap(),
        "9h".parse().unwrap()).unwrap().to_range_string(), "T9s");

        assert_eq!(HoleCards::new("8d".parse().unwrap(),
        "9h".parse().unwrap()).unwrap().to_range_string(), "98o");
    }
}