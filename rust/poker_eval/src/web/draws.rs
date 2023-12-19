use crate::{
    HoleCards, PokerError
};
use crate::web::{
    PlayerFlopResults,
    PlayerPreFlopState, 
    PreflopPlayerInfo,
};



use crate::{
    partial_rank_cards, Card, FlushDrawType, PartialRankContainer, StraightDrawType,
};
use wasm_bindgen::prelude::wasm_bindgen;
type ResultType = u32;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Default, Serialize)]
pub struct Draws {
    pub num_iterations: ResultType,

    pub gut_shot: ResultType,
    pub str8_draw: ResultType,
    pub flush_draw: ResultType,
    pub backdoor_flush_draw: ResultType,

    pub one_overcard: ResultType,
    pub two_overcards: ResultType,

    pub lo_paired: ResultType,
    pub hi_paired: ResultType,
    pub pp_paired: ResultType,
}

#[wasm_bindgen]
impl Draws {
    pub fn new() -> Self {
        Self {
            gut_shot: 0,
            str8_draw: 0,
            flush_draw: 0,
            backdoor_flush_draw: 0,
            one_overcard: 0,
            two_overcards: 0,

            lo_paired: 0,
            hi_paired: 0,
            pp_paired: 0,

            num_iterations: 0,
        }
    }
}

fn update_draw(results: &mut Draws, prc: &PartialRankContainer) {
    results.num_iterations += 1;

    if let Some(sd) = prc.straight_draw.as_ref() {
        match sd.straight_draw_type {
            StraightDrawType::GutShot(_cv) => results.gut_shot += 1,
            StraightDrawType::OpenEnded => results.str8_draw += 1,
            StraightDrawType::DoubleGutShot => results.str8_draw += 1,
        }
    }

    if let Some(fd) = prc.flush_draw.as_ref() {
        match fd.flush_draw_type {
            FlushDrawType::FlushDraw => results.flush_draw += 1,
            FlushDrawType::BackdoorFlushDraw => results.backdoor_flush_draw += 1,
        }
    }

    if let Some(_pp) = prc.pocket_pair.as_ref() {
        results.pp_paired += 1;
    }

    if let Some(_hi) = prc.hi_pair.as_ref() {
        results.hi_paired += 1;
    }

    if let Some(_lo) = prc.lo_pair.as_ref() {
        results.lo_paired += 1;
    }

    let mut num_overcards: usize = 0;

    if let Some(hi) = prc.hi_card.as_ref() {
        if hi.number_above == 0 {
            num_overcards += 1;

            //Only count lower card as overcard if the higher one did too
            if let Some(lo) = prc.lo_card.as_ref() {
                if lo.number_above == 0 {
                    num_overcards += 1;
                }
            }
        }
    }

    assert!(num_overcards <= 2);

    if num_overcards == 1 {
        results.one_overcard += 1;
    } else if num_overcards == 2 {
        results.two_overcards += 1;
    }
}

pub fn eval_current_draws(
    active_players: &[(usize, &PreflopPlayerInfo)],
    player_cards: &[HoleCards],
    eval_cards: &Vec<Card>,
    flop_results: &mut Vec<PlayerFlopResults>,
    //treat first active player as the hero, all others as villians
    villian_results: &mut PlayerFlopResults,
    draw_index: usize,
) -> Result<(), PokerError> {
    if eval_cards.len() < 3 {
        return Err(PokerError::from_string(format!(
            "eval_current: eval_cards needs at least 3 cards, but had {} cards",
            eval_cards.len()
        )));
    }
    if eval_cards.len() >= 5 {
        return Err(PokerError::from_string(format!(
            "eval_current: too many eval_cards, should be 4 max since we are drawing, but had {} cards",
            eval_cards.len()
        )));
    }

    //flop  = 0
    //turn = 1
    //we don't draw on the river
    assert!(draw_index < 2);

    let n_players = active_players.len();
    assert!(n_players > 1);
    assert_eq!(player_cards.len(), n_players);

    let mut best_villian_draws = PartialRankContainer::default();

    for (active_index, (_p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        let hc = &player_cards[active_index];
        let prc = partial_rank_cards(&hc, &eval_cards);

        update_draw(
            &mut flop_results[active_index].street_draws[draw_index],
            &prc,
        );

        if active_index > 0 {
            best_villian_draws.merge_best(&prc);
        }
    }

    update_draw(
        &mut villian_results.street_draws[draw_index],
        &best_villian_draws,
    );

    Ok(())
}
