use std::{cmp, mem};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{BoolRange, Deck, HoleCards, PokerError, ALL_HOLE_CARDS};

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
    pub(crate) range: BoolRange,
    pub(crate) state: PlayerPreFlopState,
}

pub fn get_all_player_hole_cards(
    //usize is the original player index
    // we may have removed some players
    active_players: &[(usize, &PreflopPlayerInfo)],
    deck: &mut Deck,
    //usize is the index in active players
    all_possible_hole_cards: &Vec<(usize, Vec<HoleCards>)>,
) -> Result<Vec<HoleCards>, PokerError> {
    let mut player_cards: Vec<HoleCards> = vec![ALL_HOLE_CARDS[0]; active_players.len()];

    let mut hole_cards_set = 0;

    //Fill in hole cards for players that have them
    for (active_player_index, (_player_index, p)) in active_players.iter().enumerate() {
        if p.state != PlayerPreFlopState::UseHoleCards {
            continue;
        }

        hole_cards_set += 1;
        let pc = p.hole_cards.ok_or(PokerError::from_string(format!(
            "Player missing hole cards"
        )))?;
        player_cards[active_player_index] = pc;
    }

    assert_eq!(
        active_players.len(),
        all_possible_hole_cards.len() + hole_cards_set
    );

    for (active_player_index, possible_hole_cards) in all_possible_hole_cards.iter() {
        let p = active_players[*active_player_index].1;
        assert!(p.state == PlayerPreFlopState::UseRange);

        let hc = deck.choose_available_in_range(possible_hole_cards)?;

        player_cards[*active_player_index] = hc;
    }

    Ok(player_cards)
}
