use rand::{rngs::StdRng, Rng};

use crate::{CardUsedType, PokerError, Card};

const MAX_RAND_NUMBER_ATTEMPS: usize = 1000;

pub fn set_used_card(
    c_index: usize, 
    cards_used: &mut CardUsedType,
) -> Result<(), PokerError> {
    let count_before = cards_used.count_ones();
    cards_used.set(c_index, true);
    let count_after = cards_used.count_ones();

    if count_before + 1 != count_after {
        return Err(PokerError::from_string(format!(
            "Card already used {} in board",
            Card::try_from(c_index)?.to_string()
        )));
    }

    Ok(())
}

pub fn unset_used_card(
    c_index: usize, 
    cards_used: &mut CardUsedType,
) -> Result<(), PokerError> {
    let count_before = cards_used.count_ones();
    cards_used.set(c_index, false);
    let count_after = cards_used.count_ones();

    if count_before != count_after + 1 {
        return Err(PokerError::from_string(format!(
            "Card was not used {} in board",
            Card::try_from(c_index)?.to_string()
        )));
    }

    Ok(())
}


pub fn add_eval_card(
    c_index: usize,
    eval_cards: &mut Vec<Card>,
    cards_used: &mut CardUsedType,
) -> Result<(), PokerError> {
    set_used_card(c_index, cards_used)?;

    eval_cards.push(Card::try_from(c_index)?);

    Ok(())
}

pub fn get_unused_card(rng: &mut StdRng, cards_used: &CardUsedType) -> Option<usize> {
    let mut attempts = 0;
    loop {
        let rand_int: usize = rng.gen_range(0..52);
        assert!(rand_int < 52);
        //let card = Card::from(rand_int);
        if !cards_used[rand_int] {
            return Some(rand_int);
        }
        attempts += 1;
        if attempts > MAX_RAND_NUMBER_ATTEMPS {
            return None;
        }
    }
}
