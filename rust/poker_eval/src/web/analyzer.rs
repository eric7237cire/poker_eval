use std::{cmp, fmt::Display, mem};

use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use postflop_solver::card_pair_to_index;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    get_filtered_range_set, range_string_to_set, rank_cards, Card, CardUsedType, InRangeType, Rank,
    NUM_RANK_FAMILIES, partial_rank_cards, PartialRankContainer, StraightDrawType, FlushDrawType,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
//extern crate wasm_bindgen;
//extern crate console_error_panic_hook;
type ResultType = u32;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Default)]
pub struct RankResults {
    num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    win_eq: f64,
    tie_eq: f64,

    rank_family_count: [ResultType; 9],
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PlayerFlopResults {
    /*
    This is when evaluating the flop vs the players
    */
    //pub num_iterations: ResultType,
    pub player_index: usize,

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

    // The rust struct can't get passed via the worker interface, so we need primitive accessors
    pub fn get_perc_family(&self, street_index: usize, family_index: usize) -> f64 {
        let r = &self.street_rank_results[street_index];
        r.rank_family_count[family_index] as f64 / r.num_iterations as f64
    }
    pub fn get_perc_family_or_better(&self, street_index: usize, family_index: usize) -> f64 {
        let mut total = 0.0;
        for i in family_index..NUM_RANK_FAMILIES {
            total += self.get_perc_family(street_index, i)
        }
        total
    }
    pub fn get_equity(&self, street_index: usize) -> f64 {
        let r = &self.street_rank_results[street_index];
        (r.win_eq + r.tie_eq) / r.num_iterations as f64
    }

    //This guy has no arrays, so we can just convert it to json
    pub fn get_street_draw(&self, draw_index: usize) -> Result<JsValue, JsValue> {
        info!(
            "get_street_draw: {} ",
            draw_index
        );
        
        Ok(serde_wasm_bindgen::to_value(
            &self.street_draws[draw_index]
        )?)
        // return Draws {
        //     num_iterations: self.street_draws[draw_index].num_iterations,
        //     gut_shot: self.street_draws[draw_index].gut_shot,
        //     str8_draw: self.street_draws[draw_index].str8_draw,
        //     flush_draw: self.street_draws[draw_index].flush_draw,
        //     backdoor_flush_draw: self.street_draws[draw_index].backdoor_flush_draw,
        //     one_overcard: self.street_draws[draw_index].one_overcard,
        //     two_overcards: self.street_draws[draw_index].two_overcards,
        //     lo_paired: self.street_draws[draw_index].lo_paired,
        //     hi_paired: self.street_draws[draw_index].hi_paired,
        //     pp_paired: self.street_draws[draw_index].pp_paired,
        // };
    }
}

// #[wasm_bindgen]
// #[derive(Default, Serialize, Clone)]
// pub struct TurnResults {
//     /*
//     This is when evaluating the flop vs the players
//     */
//     pub num_iterations: ResultType,

//     turn_results: RankResults,
//     river_results: RankResults,

//     turn_draws: Draws,
// }

// #[wasm_bindgen]
// #[derive(Default, Serialize, Clone)]
// pub struct RiverResults {
//     /*
//     This is when evaluating the river vs the players
//     */
//     pub num_iterations: ResultType,

//     river_results: RankResults,

// }

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

#[derive(Debug)]
pub struct MyError {
    details: String,
}

impl MyError {
    fn from_str(msg: &str) -> MyError {
        MyError {
            details: msg.to_string(),
        }
    }
    fn from_string(msg: String) -> MyError {
        MyError { details: msg }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl From<MyError> for JsValue {
    fn from(failure: MyError) -> Self {
        js_sys::Error::new(&failure.to_string()).into()
    }
}

#[derive(Clone, Eq, PartialEq)]
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
    hole_cards: Vec<Card>,
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

        info!("FlopAnalyzer::new()");
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

    pub fn set_board_cards(&mut self, cards: &[u8]) {
        self.board_cards.clear();
        info!("set_board_cards: len {}", cards.len());
        let bc = &mut self.board_cards;
        for c in cards.iter() {
            let card = Card::from(*c);
            debug!("card: {:?}", card);
            bc.push(card);
        }
    }

    pub fn set_player_cards(&mut self, player_idx: usize, cards: &[u8]) {
        info!("set_player_cards: {} len {}", player_idx, cards.len());
        let pc = &mut self.player_info[player_idx].hole_cards;
        pc.clear();
        for c in cards.iter() {
            let card = Card::from(*c);
            debug!("card: {:?}", card);
            pc.push(card);
        }
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
        self.player_info[player_idx].hole_cards.clear();
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

    pub fn build_results(&self) -> Vec<PlayerFlopResults> {
        let active_players = self
            .player_info
            .iter()
            .enumerate()
            .filter(|(p_idx, p)| p.state != PlayerPreFlopState::Disabled)
            .collect_vec();

        let mut results = Vec::with_capacity(MAX_PLAYERS);
        for (p_idx, p) in active_players.iter() {
            let mut player_results = PlayerFlopResults::new();
            player_results.player_index = *p_idx;
            results.push(player_results);
        }
        results
    }

    pub fn simulate_flop(
        &self,
        num_iterations: u32,
        mut flop_results: Vec<PlayerFlopResults>,
    ) -> Result<Vec<PlayerFlopResults>, MyError> {
        //let n_players = self.player_info.len();
        let mut rng = StdRng::seed_from_u64(42);

        let active_players = self
            .player_info
            .iter()
            .enumerate()
            .filter(|(p_idx, p)| p.state != PlayerPreFlopState::Disabled)
            .collect_vec();

        if active_players.len() < 2 {
            return Err(MyError::from_string(format!(
                "simulate_flop: n_active_players {} < 2",
                active_players.len()
            )));
        }

        if flop_results.len() != active_players.len() {
            return Err(MyError::from_string(format!(
                "simulate_flop: flop_results.len() {} != active_players.len() {}",
                flop_results.len(),
                active_players.len()
            )));
        }

        info!("simulate_flop: num_iterations {} for {} players", num_iterations, active_players.len());

        for _it_num in 0..num_iterations {
            //debug!("simulate_flop: iteration {}", it_num);

            //let mut deck = self.prepare_deck();

            //with flop, players with hole cards
            let mut eval_cards = Vec::with_capacity(15);
            let mut used_cards = self.init_cards_used(&mut eval_cards, 3)?;

            //even though we are doing the flop, if we have turn/river cards specified, we add them too
            assert_eq!(used_cards.count_ones(), self.board_cards.len());
            assert!(self.board_cards.len() >= 3);
            
            assert_eq!(3, eval_cards.len());

            //First we choose hole cards for players that are using a range
            let player_cards =
                get_all_player_hole_cards(&active_players, &mut rng, &mut used_cards)?;

            assert_eq!(player_cards.len(), active_players.len());

            assert_eq!(self.board_cards.len() + 2 * active_players.len(), used_cards.count_ones());

            eval_current_draws(
                &active_players,
                &player_cards, &eval_cards, 
                &mut flop_results, 0)?;

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                0,
            )?;

            assert_eq!(3, eval_cards.len());

            //Turn

            //Do we have a 4th card on our board?
            if self.board_cards.len() < 4 {
                //choose one
                add_eval_card(
                    get_unused_card(&mut rng, &used_cards).unwrap(),
                    &mut eval_cards,
                    &mut used_cards,
                )?;

                assert_eq!(3, self.board_cards.len());
                assert_eq!(4 + 2 * active_players.len(), used_cards.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let turn_card_index: usize = self.board_cards[3].into();
                assert!(used_cards[turn_card_index]);
                eval_cards.push(self.board_cards[3].into());

                assert_eq!(self.board_cards.len() + 2 * active_players.len(), used_cards.count_ones());
            }

            assert_eq!(4, eval_cards.len());
            

            eval_current_draws(
                &active_players,
                &player_cards, &eval_cards, 
                &mut flop_results, 1)?;

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                1,
            )?;

            //River
            //Perhaps iterate on the remaining cards instead of each eval round doing flop/turn/river
            if self.board_cards.len() < 5 {
                add_eval_card(
                    get_unused_card(&mut rng, &mut used_cards).unwrap(),
                    &mut eval_cards,
                    &mut used_cards,
                )?;
                
                assert_eq!(5 + 2 * active_players.len(), used_cards.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let river_card_index: usize = self.board_cards[4].into();
                assert!(used_cards[river_card_index]);
                eval_cards.push(self.board_cards[4]);

                assert_eq!(5 + 2 * active_players.len(), used_cards.count_ones());
            }

            assert_eq!(5, eval_cards.len());
            

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                2,
            )?;
        }

        Ok(flop_results)
    }

    fn init_cards_used(
        &self,
        eval_cards: &mut Vec<Card>,
        num_board_cards: usize,
    ) -> Result<CardUsedType, MyError> {
        let mut cards_used = CardUsedType::default();

        if self.board_cards.len() < num_board_cards {
            return Err(MyError::from_string(format!(
                "init_cards_used: not enough board cards.  board_cards.len() {}, needed num_board_cards {}",
                self.board_cards.len(),
                num_board_cards
            )));
        }

        //We add all the board cards to used so they don't get selected again
        //But only add the num we need to eval
        for (c_idx, c) in self.board_cards.iter().enumerate() {
            set_used_card((*c).into(), &mut cards_used)?;
            
            if c_idx < num_board_cards {
                eval_cards.push(*c);
            }
            
        }


        Ok(cards_used)
    }

}

/*
Assumes all players have either hole cards or their ranges chosen
*/
pub fn eval_current(
    active_players: &[(usize, &PreflopPlayerInfo)],
    player_cards: &[(Card, Card)],
    eval_cards: &mut Vec<Card>,
    flop_results: &mut Vec<PlayerFlopResults>,
    street_index: usize,
) -> Result<(), MyError> {
    if eval_cards.len() < 3 {
        return Err(MyError::from_string(format!(
            "eval_current: eval_cards needs at least 3 cards, but had {} cards",
            eval_cards.len()
        )));
    }
    if eval_cards.len() > 5 {
        return Err(MyError::from_string(format!(
            "eval_current: too many eval_cards, should be 5 max, but had {} cards",
            eval_cards.len()
        )));
    }

    let n_players = active_players.len();
    assert!(n_players > 1);
    assert_eq!(player_cards.len(), n_players);

    let mut hand_evals: Vec<Rank> = Vec::with_capacity(n_players);

    for (active_index, (p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        eval_cards.push(player_cards[active_index].0);
        eval_cards.push(player_cards[active_index].1);

        flop_results[active_index].street_rank_results[street_index].num_iterations += 1;

        let rank = rank_cards(&eval_cards);

        update_results_from_rank(
            &mut flop_results[active_index].street_rank_results[street_index],
            rank,
        );

        hand_evals.push(rank);

        eval_cards.pop();
        eval_cards.pop();
    }

    let winner_indexes = indices_of_max_values(&hand_evals);

    assert!(winner_indexes.len() > 0);

    for winner_idx in winner_indexes.iter() {
        let results = &mut flop_results[*winner_idx].street_rank_results[street_index];
        if winner_indexes.len() == 1 {
            results.win_eq += 1.0;
        } else {
            results.tie_eq += 1.0 / winner_indexes.len() as f64;
        }
    }

    Ok(())
}

pub fn eval_current_draws(
    active_players: &[(usize, &PreflopPlayerInfo)],
    player_cards: &[(Card, Card)],
    eval_cards: &Vec<Card>,
    flop_results: &mut Vec<PlayerFlopResults>,
    draw_index: usize,
) -> Result<(), MyError> {
    if eval_cards.len() < 3 {
        return Err(MyError::from_string(format!(
            "eval_current: eval_cards needs at least 3 cards, but had {} cards",
            eval_cards.len()
        )));
    }
    if eval_cards.len() >= 5 {
        return Err(MyError::from_string(format!(
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

    for (active_index, (p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        let prc = partial_rank_cards(
            &[player_cards[active_index].0, player_cards[active_index].1], 
            &eval_cards);

        update_draw(
            &mut flop_results[active_index].street_draws[draw_index],
            prc,
        );

    }

    Ok(())
}

fn add_hole_cards_to_used(
    player_cards: &(Card, Card),
    cards_used: &mut CardUsedType,
) -> Result<(), MyError> {
    let count_before = cards_used.count_ones();
    cards_used.set(player_cards.0.into(), true);
    cards_used.set(player_cards.1.into(), true);
    let count_after = cards_used.count_ones();

    if count_before + 2 != count_after {
        return Err(MyError::from_string(format!(
            "Card already used {} {} in board",
            player_cards.0.to_string(),
            player_cards.1.to_string()
        )));
    }

    Ok(())
}

fn get_all_player_hole_cards(
    active_players: &[(usize, &PreflopPlayerInfo)],
    rng: &mut StdRng,
    cards_used: &mut CardUsedType,
) -> Result<Vec<(Card, Card)>, MyError> {
    let mut player_cards = Vec::with_capacity(active_players.len());

    //Add all the hole cards to used cards first
    for (p_idx, p) in active_players.iter() {
        if p.state != PlayerPreFlopState::UseHoleCards {
            continue;
        }
        if p.hole_cards.len() != 2 {
            return Err(MyError::from_string(format!(
                "get_all_player_hole_cards: player {} has {} hole cards",
                p_idx,
                p.hole_cards.len()
            )));
        }
        add_hole_cards_to_used(&(p.hole_cards[0], p.hole_cards[1]), cards_used)?;
    }

    //Now add them in order
    for (p_idx, p) in active_players.iter() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        if p.state == PlayerPreFlopState::UseHoleCards {
            //we already updated used cards above
            let pc = (p.hole_cards[0], p.hole_cards[1]);
            player_cards.push(pc);
            continue;
        }

        //Now deal with ranges
        let mut attempts = 0;
        let mut card1_index;
        let mut card2_index;

        loop {
            attempts += 1;

            if attempts > MAX_RAND_NUMBER_ATTEMPS {
                return Err(MyError::from_string(
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
        let pc = (Card::from(card1_index), Card::from(card2_index));
        add_hole_cards_to_used(&pc, cards_used)?;
        player_cards.push(pc);
    }

    Ok(player_cards)
}

fn set_used_card(
    c_index: usize, 
    cards_used: &mut CardUsedType,
) -> Result<(), MyError> {
    let count_before = cards_used.count_ones();
    cards_used.set(c_index, true);
    let count_after = cards_used.count_ones();

    if count_before + 1 != count_after {
        return Err(MyError::from_string(format!(
            "Card already used {} in board",
            Card::from(c_index).to_string()
        )));
    }

    Ok(())
}

fn add_eval_card(
    c_index: usize,
    eval_cards: &mut Vec<Card>,
    cards_used: &mut CardUsedType,
) -> Result<(), MyError> {
    set_used_card(c_index, cards_used)?;

    eval_cards.push(Card::from(c_index));

    Ok(())
}

fn update_results_from_rank(results: &mut RankResults, rank: Rank) {
    results.rank_family_count[rank.get_family_index()] += 1;
}

fn update_draw(results: &mut Draws, prc: PartialRankContainer) {

    results.num_iterations += 1;

    if let Some(sd) = prc.straight_draw {
        match sd.straight_draw_type {
            StraightDrawType::GutShot(_cv) => results.gut_shot += 1,
            StraightDrawType::OpenEnded => results.str8_draw += 1,
            StraightDrawType::DoubleGutShot => results.str8_draw += 1,
        }
    }

    if let Some(fd) = prc.flush_draw {
        match fd.flush_draw_type
        {
            FlushDrawType::FlushDraw => results.flush_draw += 1,
            FlushDrawType::BackdoorFlushDraw => results.backdoor_flush_draw += 1,
        }
    }

    if let Some(_pp) = prc.pocket_pair {
        results.pp_paired += 1;
    }

    if let Some(_hi) = prc.hi_pair {
        results.hi_paired += 1;
    }

    if let Some(_lo) = prc.lo_pair {
        results.lo_paired += 1;
    }

    let mut num_overcards = 0;

    if let Some (hi) = prc.hi_card {
        if hi.number_above == 0 {
            num_overcards += 1;
        }
    }

    if let Some (lo) = prc.lo_card {
        if lo.number_above == 0 {
            num_overcards += 1;
        }
    }

    if num_overcards == 1 {
        results.one_overcard += 1;
    } else if num_overcards == 2 {
        results.two_overcards += 1;
    }
}

//returns winners and how many players were considered (non None rank)
fn indices_of_max_values(arr: &[Rank]) -> Vec<usize> {
    let mut non_none_count = 0;
    let mut max_indices = Vec::with_capacity(MAX_PLAYERS);
    let mut max_value = Rank::HighCard(0);

    for (index, &value) in arr.iter().enumerate() {
        non_none_count += 1;
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

fn get_unused_card(rng: &mut StdRng, cards_used: &CardUsedType) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{card_u8s_from_string, web::analyzer::PlayerPreFlopState};

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

        analyzer.set_player_cards(0, card_u8s_from_string("7h 6s").as_slice());

        analyzer.set_player_cards(3, card_u8s_from_string("Th 9h").as_slice());

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice());

        let num_it = 10_000;
        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

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

        analyzer.set_player_cards(0, card_u8s_from_string("8d 7s").as_slice());

        analyzer.set_player_cards(3, card_u8s_from_string("Qd 5c").as_slice());

        analyzer.set_player_range(
            2,
            "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o",
        );

        analyzer.set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice());

        let num_it = 4_000;

        let tolerance = 0.5;
        //let tolerance = 0.1;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

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
    fn test_heads_up_with_cards() {
        let mut analyzer = super::flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer.set_player_cards(0, card_u8s_from_string("2d 7s").as_slice());
        analyzer.set_player_cards(3, card_u8s_from_string("Ad Ac").as_slice());

        analyzer.set_board_cards(card_u8s_from_string("3s 4s Ac 6h 5c").as_slice());

        let num_it = 400_000;

        let tolerance = 0.1;

        //let mut results = analyzer.build_results();
        //analyzer.simulate_flop(num_it, &mut results).unwrap();
    }
}
