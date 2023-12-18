use std::{cmp::{self}, mem};
use crate::{PokerError, get_unused_card, add_eval_card, set_used_card, HoleCards};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use postflop_solver::card_pair_to_index;
use rand::{rngs::StdRng, SeedableRng};

use crate::{
    range_string_to_set, rank_cards, Card, CardUsedType, InRangeType, Rank,
    NUM_RANK_FAMILIES, partial_rank_cards, PartialRankContainer, StraightDrawType, FlushDrawType,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
//extern crate wasm_bindgen;
//extern crate console_error_panic_hook;
type ResultType = u32;
use serde::Serialize;

//#[wasm_bindgen]
#[derive(Default)]
pub struct RankResults {
    num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    win_eq: f64,
    tie_eq: f64,

    rank_family_count: [ResultType; 9],
}

#[derive(Default)]
#[wasm_bindgen]
pub struct PlayerFlopResults {
    /*
    This is when evaluating the flop vs the players
    */
    //pub num_iterations: ResultType,
    player_index: usize,

    //hand rankings as of the flop
    //3 of them, flop turn river
    street_rank_results: [RankResults; 3],

    //turn & river
    street_draws: [Draws; 2],
}

#[wasm_bindgen]
impl PlayerFlopResults {
    pub fn new() -> Self {
        let d = Self::default();
        //d.num_iterations = 0;
        d
    }

    
}


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

#[derive(Eq, PartialEq, Debug)]
#[repr(u8)]
enum PlayerPreFlopState {
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
    range_string: String,
    //results: Results,
    hole_cards: Option<HoleCards>,
    range_set: InRangeType,
    state: PlayerPreFlopState,
}

#[wasm_bindgen]
//doing this to stop warnings in vs code about camel case in the wasm function names
#[allow(non_camel_case_types)]
pub struct flop_analyzer {
    board_cards: Vec<Card>,
    player_info: Vec<PreflopPlayerInfo>,
}


#[wasm_bindgen]
pub struct FlopSimulationResults {
    //Because of https://stackoverflow.com/questions/68243940/rust-wasm-bindgen-struct-with-string
    //we don't want copy but we have accessors
    all_villians: PlayerFlopResults,
    flop_results: Vec<PlayerFlopResults>,
    
}

#[wasm_bindgen]
impl FlopSimulationResults {

    //note these player_indexes are the index of active players

    // The rust struct can't get passed via the worker interface, so we need primitive accessors
    pub fn get_perc_family(&self, active_player_index: Option<usize>, street_index: usize, family_index: usize) -> f64 {
        let r = if let Some(p_idx) = active_player_index {
             &self.flop_results[p_idx].street_rank_results[street_index]
        } else {
            &self.all_villians.street_rank_results[street_index]
        };
          
        r.rank_family_count[family_index] as f64 / r.num_iterations as f64
    }
    pub fn get_perc_family_or_better(&self, active_player_index: Option<usize>, street_index: usize, family_index: usize) -> f64 {
        let mut total = 0.0;
        for i in family_index..NUM_RANK_FAMILIES {
            total += self.get_perc_family(active_player_index, street_index, i)
        }
        total
    }
    pub fn get_equity(&self, active_player_index: Option<usize>, street_index: usize) -> f64 {
        
        let r = if let Some(p_idx) = active_player_index {
             &self.flop_results[p_idx].street_rank_results[street_index]
        } else {
            &self.all_villians.street_rank_results[street_index]
        };

        (r.win_eq + r.tie_eq) / r.num_iterations as f64
    }

    //This guy has no arrays, so we can just convert it to json
    pub fn get_street_draw(&self, active_player_index: Option<usize>, draw_index: usize) -> Result<JsValue, JsValue> {
        info!(
            "get_street_draw: {} ",
            draw_index
        );
        
        Ok(serde_wasm_bindgen::to_value(
        if let Some(p_idx) = active_player_index {            
                &self.flop_results[p_idx].street_draws[draw_index]            
        } else {
            &self.all_villians.street_draws[draw_index]
        })?)
        
    }

    pub fn get_num_players(&self) -> usize {
        self.flop_results.len()
    }

    //Convert from active_player_index to original
    pub fn get_player_index(&self, player_index: usize) -> usize {
        info!(
            "get_player_index: {} ",
            player_index
        );
        
        self.flop_results[player_index].player_index
    }
}

//hero is 0
const MAX_PLAYERS: usize = 5;

const MAX_RAND_NUMBER_ATTEMPS: usize = 1000;

#[wasm_bindgen]
impl flop_analyzer {
    pub fn new() -> Self {
        if cfg!(any(target_family = "wasm")) {
            console_error_panic_hook::set_once();
            wasm_logger::init(wasm_logger::Config::default());
        }

        debug!("debug");
        warn!("warn");
        trace!("trace");
        error!("error");

        info!("Initializing FlopAnalyzer ");

        Self {
            board_cards: Vec::with_capacity(7),
            player_info: Vec::with_capacity(MAX_PLAYERS),
        }
    }

    pub fn set_board_cards(&mut self, cards: &[u8]) -> Result<(), PokerError> {
        self.board_cards.clear();
        info!("set_board_cards: len {}", cards.len());
        
        for c in cards.iter() {
            let card = Card::try_from(*c)?;
            self.check_if_used(card)?;
            debug!("card: {:?}", card);
            self.board_cards.push(card);
        }

        Ok(())
    }

    fn check_if_used(&self, card: Card) -> Result<(), PokerError> {
        
        if self.board_cards.contains(&card) {
            return Err(PokerError::from_string(format!(
                "set_board_cards: card {} already in board",
                card.to_string()
            )));
        }

        for p in self.player_info.iter() {
            if p.state != PlayerPreFlopState::UseHoleCards {
                continue;
            } 
            
            if let Some(hc) = p.hole_cards {
                if hc.get_hi_card() == card || hc.get_lo_card() == card {
                    return Err(PokerError::from_string(format!(
                        "set_board_cards: card {} already in hole cards",
                        card.to_string()
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn set_player_cards(&mut self, player_idx: usize, cards: &[u8]) -> Result<(), PokerError> {
        info!("set_player_cards idx {} with {} cards", player_idx, cards.len());

        if player_idx >= self.player_info.len() {
            return Err(PokerError::from_string(format!(
                "set_player_cards: player_idx {} >= self.player_info.len() {}",
                player_idx,
                self.player_info.len()
            )));
        }

        if 2 != cards.len() {
            return Err(PokerError::from_string(format!(
                "set_player_cards: cards.len() {} != 2",
                cards.len()
            )));
        }

        let card1 = Card::try_from(cards[0])?;
        self.check_if_used(card1)?;
        let card2 = Card::try_from(cards[1])?;
        self.check_if_used(card2)?;

        self.player_info[player_idx].hole_cards = Some(HoleCards::new(card1, card2)?);
        
        Ok(())
    }

    pub fn set_player_range(&mut self, player_idx: usize, range_str: &str) {
        info!("set_player_range: {} [{}]", player_idx, range_str);

        if range_str.is_empty() {
            warn!("set_player_range: empty string");
            return;
        }
        let range_set = range_string_to_set(range_str);

        info!("% is {}", range_set.count_ones() as f64 / 2652.0);

        self.player_info[player_idx].range_set = range_set;
        self.player_info[player_idx].range_string = range_str.to_string();
    }

    pub fn set_player_state(&mut self, player_idx: usize, state: u8) {
        info!("set_player_state: {} {}", player_idx, state);
        self.player_info[player_idx].state = state.into();
    }

    pub fn clear_player_cards(&mut self, player_idx: usize) {
        self.player_info[player_idx].hole_cards = None;
    }

    pub fn reset(&mut self) {
        self.board_cards.clear();
        self.player_info.clear();
        info!("reset to {} players", MAX_PLAYERS);
        for _ in 0..MAX_PLAYERS {
            let p_info = PreflopPlayerInfo::default();
            //p_info.results.rank_family_count = vec![0; 9];
            self.player_info.push(p_info);
        }
    }

    fn init_cards_used(&self) -> Result<CardUsedType, PokerError> {
        let mut cards_used = CardUsedType::default();

        
        for c in self.board_cards.iter() {
            set_used_card((*c).into(), &mut cards_used)?;
        }

        for p in self.player_info.iter() {
            if p.state != PlayerPreFlopState::UseHoleCards {
                continue;
            } 
            
            let hc = p.hole_cards.ok_or(PokerError::from_string(format!("Player missing hole cards")))?;

            set_used_card(hc.get_hi_card().into(), &mut cards_used)?;
            set_used_card(hc.get_lo_card().into(), &mut cards_used)?;
        }

        Ok(cards_used)
    }

    //Because wasm doesn't like refs, we use move semantics
    pub fn build_results(&self) -> FlopSimulationResults {
        let active_players = self
            .player_info
            .iter()
            .enumerate()
            .filter(|(_p_idx, p)| p.state != PlayerPreFlopState::Disabled)
            .collect_vec();

        let mut results = Vec::with_capacity(MAX_PLAYERS);
        for (p_idx, _p) in active_players.iter() {
            let mut player_results = PlayerFlopResults::new();
            player_results.player_index = *p_idx;
            results.push(player_results);
        }
        FlopSimulationResults {
            flop_results: results,
            all_villians: PlayerFlopResults::new(),
        }
    }

    pub fn simulate_flop(
        &self,
        num_iterations: u32,
        all_flop_results: FlopSimulationResults,        
    ) -> Result<FlopSimulationResults, PokerError> {
        //let n_players = self.player_info.len();
        let mut rng = StdRng::seed_from_u64(42);

        let active_players = self
            .player_info
            .iter()
            .enumerate()
            .filter(|(_p_idx, p)| p.state != PlayerPreFlopState::Disabled)
            .collect_vec();

        if active_players.len() < 2 {
            return Err(PokerError::from_string(format!(
                "simulate_flop: n_active_players {} < 2",
                active_players.len()
            )));
        }

        let mut flop_results = all_flop_results.flop_results;
        let mut villian_results = all_flop_results.all_villians;

        if flop_results.len() != active_players.len() {
            return Err(PokerError::from_string(format!(
                "simulate_flop: flop_results.len() {} != active_players.len() {}",
                flop_results.len(),
                active_players.len()
            )));
        }

        info!("simulate_flop: num_iterations {} for {} players", num_iterations, active_players.len());

        let base_cards_used = self.init_cards_used()?;

        for _it_num in 0..num_iterations {
            //debug!("simulate_flop: iteration {}", it_num);

            //let mut deck = self.prepare_deck();

            //with flop, players with hole cards
            let mut eval_cards = Vec::with_capacity(15);
            let mut cards_used = base_cards_used.clone();
            let num_added = self.add_flop(&mut rng, &mut eval_cards, &mut cards_used)?;

            assert_eq!(3, eval_cards.len());

            //First we choose hole cards for players that are using a range
            let player_cards =
                get_all_player_hole_cards(&active_players, &mut rng, &mut cards_used)?;

            assert_eq!(player_cards.len(), active_players.len());

            assert_eq!(num_added+self.board_cards.len() + 2 * active_players.len(), cards_used.count_ones());

            eval_current_draws(
                &active_players,
                &player_cards, &eval_cards, 
                &mut flop_results, 
                &mut villian_results,
                0)?;

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                0,
            )?;

            assert_eq!(3, eval_cards.len());

            //Turn

            //Do we have a 4th card on our board?
            if self.board_cards.len() < 4 {
                //choose one
                add_eval_card(
                    get_unused_card(&mut rng, &cards_used).unwrap(),
                    &mut eval_cards,
                    &mut cards_used,
                )?;

                assert_eq!(3, self.board_cards.len()+num_added);
                assert_eq!(4 + 2 * active_players.len(), cards_used.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let turn_card_index: usize = self.board_cards[3].into();
                assert!(cards_used[turn_card_index]);
                eval_cards.push(self.board_cards[3].into());
                assert_eq!(num_added, 0);

                assert_eq!(self.board_cards.len() + 2 * active_players.len(), cards_used.count_ones());
            }

            assert_eq!(4, eval_cards.len());
            

            eval_current_draws(
                &active_players,
                &player_cards, &eval_cards, 
                &mut flop_results, &mut villian_results, 1)?;

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                1,
            )?;

            //River
            //Perhaps iterate on the remaining cards instead of each eval round doing flop/turn/river
            if self.board_cards.len() < 5 {
                add_eval_card(
                    get_unused_card(&mut rng, &mut cards_used).unwrap(),
                    &mut eval_cards,
                    &mut cards_used,
                )?;
                
                assert_eq!(5 + 2 * active_players.len(), cards_used.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let river_card_index: usize = self.board_cards[4].into();
                assert!(cards_used[river_card_index]);
                eval_cards.push(self.board_cards[4]);

                assert_eq!(num_added, 0);

                assert_eq!(self.board_cards.len() + 2 * active_players.len(), cards_used.count_ones());
            }

            assert_eq!(5, eval_cards.len());
            

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                2,
            )?;
        }

        Ok(FlopSimulationResults{
            flop_results: flop_results,
            all_villians: villian_results,
        })
    }

    fn add_flop(
        &self,
        rng: &mut StdRng,
        eval_cards: &mut Vec<Card>,
        cards_used: &mut CardUsedType,
    ) -> Result<usize, PokerError> {
        
        assert!(eval_cards.is_empty());

        let num_cards_needed_for_flop = 3;

        //We add all the board cards to used so they don't get selected again
        //But only add the num we need to eval
        for (c_idx, c) in self.board_cards.iter().enumerate() {
            //Should have been initialized already in init_cards_used
            let card_as_usize: usize = (*c).into();
            assert!(cards_used[card_as_usize]);
            
            if c_idx < num_cards_needed_for_flop {
                eval_cards.push(*c);
            }
            
        }

        assert!(eval_cards.len() <= num_cards_needed_for_flop);


        //Choose any cards up until the flop has been chosen

        let mut num_chosen = 0;
        for _ in eval_cards.len()..num_cards_needed_for_flop {
            add_eval_card(
                get_unused_card(rng, &cards_used).unwrap(),
                eval_cards,
                cards_used,
            )?;
            num_chosen += 1;
        }


        Ok(num_chosen)
    }

}

/*
Assumes all players have either hole cards or their ranges chosen
*/
pub fn eval_current(
    active_players: &[(usize, &PreflopPlayerInfo)],
    player_cards: &[HoleCards],
    eval_cards: &mut Vec<Card>,
    flop_results: &mut Vec<PlayerFlopResults>,
    //treat first active player as the hero, all others as villians
    villian_results: &mut PlayerFlopResults,
    street_index: usize,
) -> Result<(), PokerError> {
    if eval_cards.len() < 3 {
        return Err(PokerError::from_string(format!(
            "eval_current: eval_cards needs at least 3 cards, but had {} cards",
            eval_cards.len()
        )));
    }
    if eval_cards.len() > 5 {
        return Err(PokerError::from_string(format!(
            "eval_current: too many eval_cards, should be 5 max, but had {} cards",
            eval_cards.len()
        )));
    }

    let n_players = active_players.len();
    assert!(n_players > 1);
    assert_eq!(player_cards.len(), n_players);

    let mut hand_evals: Vec<Rank> = Vec::with_capacity(n_players);

    for (active_index, (_p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        player_cards[active_index].add_to_eval(eval_cards);
        
        let rank = rank_cards(&eval_cards);

        update_results_from_rank(
            &mut flop_results[active_index].street_rank_results[street_index],
            rank,
        );

        hand_evals.push(rank);

        player_cards[active_index].remove_from_eval(eval_cards)?;
    }

    //Best villian hand
    let best_villian_rank = hand_evals[1..]
        .iter()
        .fold(Rank::HighCard(0), |acc, &x| acc.max(x));
    update_results_from_rank(&mut villian_results.street_rank_results[street_index], best_villian_rank);

    let winner_indexes = indices_of_max_values(&hand_evals);

    assert!(winner_indexes.len() > 0);

    for winner_idx in winner_indexes.iter() {
        let results = &mut flop_results[*winner_idx].street_rank_results[street_index];
        if winner_indexes.len() == 1 {
            results.win_eq += 1.0;
            if *winner_idx > 0 {
                villian_results.street_rank_results[street_index].tie_eq += 1.0;
            }
        } else {
            results.tie_eq += 1.0 / winner_indexes.len() as f64;

            if *winner_idx > 0 {
                villian_results.street_rank_results[street_index].tie_eq += 1.0;
            }
        }
    }

    Ok(())
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
        let prc = partial_rank_cards(
            &hc, 
            &eval_cards);

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


fn get_all_player_hole_cards(
    active_players: &[(usize, &PreflopPlayerInfo)],
    rng: &mut StdRng,
    cards_used: &mut CardUsedType,
) -> Result<Vec<HoleCards>, PokerError> {
    let mut player_cards: Vec<HoleCards> = Vec::with_capacity(active_players.len());

    
    for (p_idx, p) in active_players.iter() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        if p.state == PlayerPreFlopState::UseHoleCards {
            let pc = p.hole_cards.ok_or(PokerError::from_string(format!("Player missing hole cards")))?;
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

            let range_index = card_pair_to_index(card1_index as u8, card2_index as u8);

            if !p.range_set[range_index] {
                continue;
            }

            break;
        }

        //we set their cards
        let pc = HoleCards::new(
            Card::try_from(card1_index)?, Card::try_from(card2_index)?)?;
        pc.set_used(cards_used)?;
        player_cards.push(pc);
    }

    Ok(player_cards)
}


fn update_results_from_rank(results: &mut RankResults, rank: Rank) {
    results.num_iterations += 1;
    results.rank_family_count[rank.get_family_index()] += 1;
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
        match fd.flush_draw_type
        {
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

    if let Some (hi) = prc.hi_card.as_ref() {
        if hi.number_above == 0 {
            num_overcards += 1;

            //Only count lower card as overcard if the higher one did too
            if let Some (lo) = prc.lo_card.as_ref() {
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

//returns winners and how many players were considered (non None rank)
fn indices_of_max_values(arr: &[Rank]) -> Vec<usize> {
    
    let mut max_indices = Vec::with_capacity(MAX_PLAYERS);
    let mut max_value = Rank::HighCard(0);

    for (index, &value) in arr.iter().enumerate() {
        if value > max_value {
            max_value = value;
            max_indices.clear();
            max_indices.push(index);
        } else if value == max_value {
            max_indices.push(index);
        }
    }

    max_indices
}


#[cfg(test)]
mod tests {
    use crate::{card_u8s_from_string, web::analyzer::PlayerPreFlopState, Rank};

    fn assert_equity(equity: f64, target: f64, tolerance: f64) {
        let passed = (equity - target).abs() < tolerance;
        if !passed {
            println!("assert_equity failed: {} != {}", equity, target);
        }
        assert!(passed);
    }

    #[test]
    fn test_heads_up_both_with_hole_cards() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();
        let tolerance = 0.1;

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("7h 6s").as_slice()).unwrap();

        analyzer.set_player_cards(3, card_u8s_from_string("Th 9h").as_slice()).unwrap();

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice()).unwrap();

        let num_it = 10_000;
        let f_results = analyzer.build_results();
        let f_results = analyzer.simulate_flop(num_it, f_results).unwrap();

        let results = &f_results.flop_results;

        // let results = analyzer
        //     .player_info
        //     .iter()
        //     .map(|p| &p.results)
        //     .collect_vec();

        assert_eq!(results[0].street_rank_results[2].num_iterations, num_it);
        assert_eq!(0, results[0].player_index);
        assert_equity(
            100.0 * results[0].street_rank_results[2].win_eq / num_it as f64,
            21.92,
            tolerance,
        );

        assert_eq!(results[1].street_rank_results[2].num_iterations, num_it);
        assert_eq!(3, results[1].player_index);

        assert_equity(
            100.0 * results[1].street_rank_results[2].win_eq / num_it as f64,
            78.08,
            tolerance,
        );
    }

    #[test]
    fn test_3way_with_ranges() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseRange as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("8d 7s").as_slice()).unwrap();

        analyzer.set_player_cards(3, card_u8s_from_string("Qd 5c").as_slice()).unwrap();

        analyzer.set_player_range(
            2,
            "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o",
        );

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice()).unwrap();

        let num_it = 4_000;

        let tolerance = 0.5;
        //let tolerance = 0.1;

        let f_results = analyzer.build_results();
        let f_results = analyzer.simulate_flop(num_it, f_results).unwrap();

        let results = &f_results.flop_results;

        assert_eq!(results[0].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[0].player_index, 0);

        assert_equity(
            100.0 * results[0].street_rank_results[2].win_eq
                / results[0].street_rank_results[2].num_iterations as f64,
            21.03,
            tolerance,
        );
        assert_equity(
            100.0 * results[0].street_rank_results[2].tie_eq
                / results[0].street_rank_results[2].num_iterations as f64,
            0.12,
            0.05,
        );

        assert_eq!(results[2].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[2].player_index, 3);

        // assert_equity(
        //     100.0 * results[3].eq_not_folded / not_folded as f64,
        //     50.93 + 0.82,
        //     0.7,
        // );
        assert_equity(
            100.0 * results[2].street_rank_results[2].win_eq / num_it as f64,
            50.93,
            tolerance,
        );
        assert_equity(
            100.0 * results[2].street_rank_results[2].tie_eq / num_it as f64,
            0.82,
            tolerance,
        );

        assert_eq!(results[1].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[1].player_index, 2);
        //let not_folded = results[3].num_iterations;

        assert_equity(
            100.0 * results[1].street_rank_results[2].win_eq / num_it as f64,
            26.14,
            tolerance,
        );
        assert_equity(
            100.0 * results[1].street_rank_results[2].tie_eq / num_it as f64,
            0.95,
            tolerance,
        );
    }

    #[test]
    fn test_villian_draws() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(4, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("Td 8s").as_slice()).unwrap();

        analyzer.set_player_cards(3, card_u8s_from_string("Ad Kc").as_slice()).unwrap();
        analyzer.set_player_cards(4, card_u8s_from_string("5s 5c").as_slice()).unwrap();
        analyzer.set_player_cards(2, card_u8s_from_string("Qd 7d").as_slice()).unwrap();

        analyzer.set_board_cards(card_u8s_from_string("9s 8c Ah 5h 6h").as_slice()).unwrap();

        let num_it = 1;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

        let v_r = &results.all_villians;
        assert_eq!(1, v_r.street_rank_results[0].rank_family_count[Rank::OnePair(0).get_family_index()]);
        assert_eq!(1u32, v_r.street_rank_results[0].rank_family_count.iter().sum());
        assert_eq!(0, v_r.street_draws[0].gut_shot);
        assert_eq!(0, v_r.street_draws[0].two_overcards);
        assert_eq!(0, v_r.street_draws[0].one_overcard);

        //Turn villian picks up gut shot
        assert_eq!(1, v_r.street_rank_results[1].rank_family_count[Rank::ThreeOfAKind(0).get_family_index()]);
        assert_eq!(1u32, v_r.street_rank_results[1].rank_family_count.iter().sum());
        assert_eq!(1, v_r.street_draws[1].gut_shot);
        assert_eq!(0, v_r.street_draws[1].two_overcards);
        assert_eq!(0, v_r.street_draws[1].one_overcard);

        assert_eq!(0, v_r.street_rank_results[2].rank_family_count[Rank::OnePair(0).get_family_index()]);
        assert_eq!(1, v_r.street_rank_results[2].rank_family_count[Rank::Straight(0).get_family_index()]);
        assert_eq!(1u32, v_r.street_rank_results[2].rank_family_count.iter().sum());
        assert_eq!(2, v_r.street_draws.len());
        
    }

    #[test]
    fn test_villian_overcards() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(4, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("Tc 8s").as_slice()).unwrap();

        analyzer.set_player_cards(3, card_u8s_from_string("Ad Jc").as_slice()).unwrap();
        analyzer.set_player_cards(4, card_u8s_from_string("Ks Qc").as_slice()).unwrap();
        analyzer.set_player_cards(2, card_u8s_from_string("Jd Td").as_slice()).unwrap();

        analyzer.set_board_cards(card_u8s_from_string("2s 4c 7h Qh Ah").as_slice()).unwrap();

        let num_it = 1;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

        let v_r = &results.all_villians;
        assert_eq!(1, v_r.street_rank_results[0].rank_family_count[Rank::HighCard(0).get_family_index()]);
        assert_eq!(1, v_r.street_rank_results[0].rank_family_count.iter().sum::<u32>());
        assert_eq!(1, v_r.street_draws[0].two_overcards);
        assert_eq!(0, v_r.street_draws[0].one_overcard);

        assert_eq!(1, v_r.street_rank_results[1].rank_family_count[Rank::OnePair(0).get_family_index()]);
        assert_eq!(1, v_r.street_rank_results[1].rank_family_count.iter().sum::<u32>());
        assert_eq!(0, v_r.street_draws[1].two_overcards);
        assert_eq!(1, v_r.street_draws[1].one_overcard);

        assert_eq!(1, v_r.street_rank_results[2].rank_family_count[Rank::OnePair(0).get_family_index()]);
        assert_eq!(1, v_r.street_rank_results[2].rank_family_count.iter().sum::<u32>());
        assert_eq!(2, v_r.street_draws.len());


    }
}
