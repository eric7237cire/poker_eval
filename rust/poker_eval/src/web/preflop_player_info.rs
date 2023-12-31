use std::{cmp, mem};

use rand::rngs::StdRng;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    get_unused_card, web::MAX_RAND_NUMBER_ATTEMPS, Card, CardUsedType, HoleCards, InRangeType,
    PokerError, ALL_CARDS,
};

#[derive(Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PlayerPreFlopState {
    Disabled = 0,
    UseHoleCards = 1,
    UseRange = 2,
}

impl From<u8> for PlayerPreFlopState {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(cmp::min(value, 2)) }
    }
}

impl Default for PlayerPreFlopState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PreflopPlayerInfo {
    pub(crate) range_string: String,
    //results: Results,
    pub(crate) hole_cards: Option<HoleCards>,
    pub(crate) range_set: InRangeType,
    pub(crate) state: PlayerPreFlopState,
}

pub fn get_all_player_hole_cards(
    active_players: &[(usize, &PreflopPlayerInfo)],
    rng: &mut StdRng,
    cards_used: &mut CardUsedType,
) -> Result<Vec<HoleCards>, PokerError> {
    let mut player_cards: Vec<HoleCards> = Vec::with_capacity(active_players.len());

    for (p_idx, p) in active_players.iter() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        if p.state == PlayerPreFlopState::UseHoleCards {
            let pc = p.hole_cards.ok_or(PokerError::from_string(format!(
                "Player missing hole cards"
            )))?;
            player_cards.push(pc);
            continue;
        }

        assert_eq!(p.state, PlayerPreFlopState::UseRange);

        //Now deal with ranges
        let mut attempts = 0;
        let mut card1_index;
        let mut card2_index;

        loop {
            attempts += 1;

            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return Err(PokerError::from_string(
                    format!("Unable to find cards for player {} after {} attempts.  Cards used count {} range str {} == {:.1}%",
                    p_idx, attempts, cards_used.count_ones(),
                    &p.range_string, p.range_set.count_ones() as f64 / 2652.0 * 100.0)
                ));
            }

            card1_index = get_unused_card(rng, cards_used).unwrap();
            card2_index = get_unused_card(rng, cards_used).unwrap();

            if card1_index == card2_index {
                continue;
            }

            let hc = HoleCards::new(ALL_CARDS[card1_index], ALL_CARDS[card2_index])?;

            let range_index = hc.to_range_index();

            if !p.range_set[range_index] {
                continue;
            }

            break;
        }

        //we set their cards
        let pc = HoleCards::new(ALL_CARDS[card1_index], ALL_CARDS[card2_index])?;
        pc.set_used(cards_used)?;
        player_cards.push(pc);
    }

    Ok(player_cards)
}
