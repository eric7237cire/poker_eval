use crate::flop_ranges::narrow_range_by_equity;
use crate::pre_calc::perfect_hash::load_boomperfect_hash;
use crate::web::player_results::PlayerFlopResults;
use crate::web::{
    eval_current, eval_current_draws, get_all_player_hole_cards, FlopSimulationResults,
    PlayerPreFlopState, PreflopPlayerInfo,
};
use crate::{set_used_card, BoolRange, HoleCards, PokerError, Board, ALL_CARDS, Deck};
use boomphf::Mphf;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};

use rand::seq::SliceRandom;
#[cfg(not(test))]
use rand::thread_rng;
use rand::{rngs::StdRng, SeedableRng};

use crate::{Card, CardUsedType};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
//doing this to stop warnings in vs code about camel case in the wasm function names
#[allow(non_camel_case_types)]
pub struct flop_analyzer {
    board_cards: Vec<Card>,
    player_info: Vec<PreflopPlayerInfo>,

    hash_func: Mphf<u32>,
}

//hero is 0
pub const MAX_PLAYERS: usize = 10;

pub const MAX_RAND_NUMBER_ATTEMPS: usize = 1000;

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

        let hash_func = load_boomperfect_hash();

        Self {
            board_cards: Vec::with_capacity(7),
            player_info: Vec::with_capacity(MAX_PLAYERS),
            hash_func,
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
        info!(
            "set_player_cards idx {} with {} cards",
            player_idx,
            cards.len()
        );

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

    pub fn set_player_range(
        &mut self,
        player_idx: usize,
        range_str: &str,
    ) -> Result<(), PokerError> {
        info!("set_player_range: {} [{}]", player_idx, range_str);

        if range_str.is_empty() {
            return Err(PokerError::from_str("set_player_range: empty string"));
        }
        let range: BoolRange = range_str.parse()?;

        info!("% is {}", range.data.count_ones() as f64 / 2652.0);

        self.player_info[player_idx].range = range;
        self.player_info[player_idx].range_string = range_str.to_string();

        Ok(())
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

    /*
    Creates bit set with all board cards & chosen hole cards set
    */
    fn init_cards_used(&self) -> Result<CardUsedType, PokerError> {
        let mut cards_used = CardUsedType::default();

        for c in self.board_cards.iter() {
            set_used_card((*c).into(), &mut cards_used)?;
        }

        for p in self.player_info.iter() {
            if p.state != PlayerPreFlopState::UseHoleCards {
                continue;
            }

            let hc = p.hole_cards.ok_or(PokerError::from_string(format!(
                "Player missing hole cards"
            )))?;

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
        equity_only: bool,
    ) -> Result<FlopSimulationResults, PokerError> {
        //let n_players = self.player_info.len();
        #[cfg(test)]
        let rng = StdRng::seed_from_u64(42);

        #[cfg(not(test))]
        let rng = StdRng::from_rng(thread_rng())
            .ok()
            .ok_or(PokerError::from_str("Failed to create rng"))?;

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

        //Store the active player index because we want to change the order in which we choose the ranges
        //Ignores players that already have hole cards set
        let mut possible_hole_cards: Vec<(usize, Vec<HoleCards>)> = {
            let mut pv = active_players
            .iter().enumerate()
            .filter(|(_idx, p)| p.1.state == PlayerPreFlopState::UseRange )
            //These are irrespective of used cards, we try later on which hole cards are still valid
            .map(|(active_player_index, r)| (active_player_index, r.1.range.get_all_enabled_holecards()))
            .collect_vec();
    
            //pv.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
            //pv.shuffle(&mut rng);
            pv
        };

        let mut flop_results = all_flop_results.flop_results;
        let mut villian_results = all_flop_results.all_villians;

        if flop_results.len() != active_players.len() {
            return Err(PokerError::from_string(format!(
                "simulate_flop: flop_results.len() {} != active_players.len() {}",
                flop_results.len(),
                active_players.len()
            )));
        }

        info!(
            "simulate_flop: num_iterations {} for {} players.  Equity Only? {}",
            num_iterations,
            active_players.len(),
            equity_only
        );

        let base_cards_used = self.init_cards_used()?;

        let mut deck = Deck {
            rng,
            used_cards: base_cards_used.clone(),
        };

        for _it_num in 0..num_iterations {
            //debug!("simulate_flop: iteration {}", it_num);

            possible_hole_cards.shuffle(&mut deck.rng);

            deck.used_cards = base_cards_used.clone();

            //with flop, players with hole cards
            let mut eval_cards = Vec::with_capacity(15);            
            
            //let mut cards_used = base_cards_used.clone();
            let num_added = self.add_flop(&mut deck, &mut eval_cards)?;

            assert_eq!(3, eval_cards.len());

            //First we choose hole cards for players that are using a range
            let player_cards =
                get_all_player_hole_cards(&active_players, &mut deck, &possible_hole_cards)?;

            assert_eq!(player_cards.len(), active_players.len());

            assert_eq!(
                num_added + self.board_cards.len() + 2 * active_players.len(),
                deck.used_cards.count_ones()
            );

            if !equity_only {
                eval_current_draws(
                    &active_players,
                    &player_cards,
                    &eval_cards,
                    &mut flop_results,
                    &mut villian_results,
                    0,
                )?;
            }

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                0,
                &self.hash_func,
            )?;

            assert_eq!(3, eval_cards.len());

            //Turn

            //Do we have a 4th card on our board?
            if self.board_cards.len() < 4 {
                let turn_card = deck.get_unused_card()?;
                eval_cards.push(turn_card);
                
                assert_eq!(3, self.board_cards.len() + num_added);
                assert_eq!(4 + 2 * active_players.len(), deck.used_cards.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let turn_card_index: usize = self.board_cards[3].into();
                assert!(deck.used_cards[turn_card_index]);
                eval_cards.push(self.board_cards[3].into());
                assert_eq!(num_added, 0);

                assert_eq!(
                    self.board_cards.len() + 2 * active_players.len(),
                    deck.used_cards.count_ones()
                );
            }

            assert_eq!(4, eval_cards.len());

            if !equity_only {
                eval_current_draws(
                    &active_players,
                    &player_cards,
                    &eval_cards,
                    &mut flop_results,
                    &mut villian_results,
                    1,
                )?;
            }

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                1,
                &self.hash_func,
            )?;

            //River
            //Perhaps iterate on the remaining cards instead of each eval round doing flop/turn/river
            if self.board_cards.len() < 5 {
                let river_card = deck.get_unused_card()?;
                eval_cards.push(river_card);
                
                assert_eq!(5 + 2 * active_players.len(), deck.used_cards.count_ones());
            } else {
                //Just do a simple push since we already added it to used cards
                let river_card_index: usize = self.board_cards[4].into();
                assert!(deck.used_cards[river_card_index]);
                eval_cards.push(self.board_cards[4]);

                assert_eq!(num_added, 0);

                assert_eq!(
                    self.board_cards.len() + 2 * active_players.len(),
                    deck.used_cards.count_ones()
                );
            }

            assert_eq!(5, eval_cards.len());

            eval_current(
                &active_players,
                &player_cards,
                &mut eval_cards,
                &mut flop_results,
                &mut villian_results,
                2,
                &self.hash_func,
            )?;
        }

        Ok(FlopSimulationResults {
            flop_results: flop_results,
            all_villians: villian_results,
        })
    }

    fn add_flop(
        &self,
        deck: &mut Deck,
        eval_cards: &mut Vec<Card>,
    ) -> Result<usize, PokerError> {
        assert!(eval_cards.is_empty());

        let num_cards_needed_for_flop = 3;

        //We add all the board cards to used so they don't get selected again
        //But only add the num we need to eval
        for (c_idx, c) in self.board_cards.iter().enumerate() {
            //Should have been initialized already in init_cards_used
            let card_as_usize: usize = (*c).into();
            assert!(deck.used_cards[card_as_usize]);

            if c_idx < num_cards_needed_for_flop {
                eval_cards.push(*c);
            }
        }

        assert!(eval_cards.len() <= num_cards_needed_for_flop);

        //Choose any cards up until the flop has been chosen

        let mut num_chosen = 0;
        for _ in eval_cards.len()..num_cards_needed_for_flop {
            let unused_card = deck.get_unused_card()?;
            eval_cards.push(unused_card);            
            num_chosen += 1;
        }

        Ok(num_chosen)
    }

    pub fn narrow_range(&self,
        str_range_to_narrow: &str,
        //seperated by ;
        str_opponent_ranges: &str,
        min_equity: f64,
        cards: &[u8],
        //This is per hole card
        num_simulations: usize,
    ) -> Result<String, PokerError> {
        info!("Starting narrow range {} simulations per hole card, min equity {:.2}", num_simulations, min_equity);

        let range_to_narrow : BoolRange = str_range_to_narrow.parse()?;
        info!("range_to_narrow {} hands", range_to_narrow.data.count_ones());

        let mut opponent_ranges = Vec::with_capacity(str_opponent_ranges.len());
        for r in str_opponent_ranges.split(';') {
            opponent_ranges.push(r.parse()?);
        }
        info!("opponent_ranges.len() {}", opponent_ranges.len());

        let mut board = Board::new();
        for c in cards.iter() {
            board.add_card(ALL_CARDS[*c as usize])?;
        }
        info!("board {}", board.to_string());

        let narrowed_range = narrow_range_by_equity(
            &range_to_narrow,
            &opponent_ranges,
            min_equity,
            &board,
            num_simulations,
        );

        info!("narrowed range {} hands", narrowed_range.data.count_ones());

        Ok(narrowed_range.to_string())
    }
}
