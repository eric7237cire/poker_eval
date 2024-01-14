extern crate wasm_bindgen;
use poker_eval::{pre_calc::NUMBER_OF_HOLE_CARDS, BoolRange, CardValue, PokerError};
//use postflop_solver::*;
use log::debug;
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RangeManager {
    range: BoolRange,
}

#[wasm_bindgen]
impl RangeManager {
    pub fn new() -> Self {
        if cfg!(any(target_family = "wasm")) {
            console_error_panic_hook::set_once();
            wasm_logger::init(wasm_logger::Config::default());
        }

        debug!("RangeManager::new()");

        Self {
            range: BoolRange::default(),
        }
    }

    pub fn clear(&mut self) {
        self.range.data.fill(false)
    }

    //row/col are 1 based
    pub fn update(&mut self, row: u8, col: u8, is_enabled: bool) -> Result<(), PokerError> {
        let rank1: CardValue = (13 - row).try_into()?;
        let rank2: CardValue = (13 - col).try_into()?;
        debug!(
            "update: row: {}, col: {}, rank1: {}, rank2: {} ==> {}",
            row, col, rank1, rank2, is_enabled
        );
        match row.cmp(&col) {
            Ordering::Equal => self.range.set_enabled_pair(rank1, is_enabled),
            Ordering::Less => self.range.set_enabled_suited(rank1, rank2, is_enabled),
            Ordering::Greater => self.range.set_enabled_offsuit(rank1, rank2, is_enabled),
        }
        debug!("update: range: {}", self.range.to_string());
        Ok(())
    }

    pub fn from_string(&mut self, s: &str) -> Result<(), PokerError> {
        //info!("from_string: {}", s);
        let result: BoolRange = s.parse()?;
        self.range = result;
        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.range.to_string()
    }

    //This is for the 'simplified' range view, with row 0 being AA AKs ...
    //
    pub fn get_weights(&self) -> Result<Box<[f32]>, PokerError> {
        let mut weights = vec![0.0; 13 * 13];

        for row in 0..13 {
            for col in 0..13 {
                let rank1: CardValue = (12 - row as u8).try_into()?;
                let rank2: CardValue = (12 - col as u8).try_into()?;
                weights[row * 13 + col] = match row.cmp(&col) {
                    Ordering::Equal => self.range.get_weight_pair(rank1),
                    Ordering::Less => self.range.get_weight_suited(rank1, rank2),
                    Ordering::Greater => self.range.get_weight_offsuit(rank1, rank2),
                };
            }
        }

        Ok(weights.into())
    }

    /*
    If a row/col has >0 but <100%, this returns what's excluded if %>50
    and what's included if %<=50

    row/col are 0 based
    */
    pub fn get_partial_comment(&self, row: u8, col: u8) -> Result<String, PokerError> {
        let rank1: CardValue = (12 - row).try_into()?;
        let rank2: CardValue = (12 - col).try_into()?;

        let st = match row.cmp(&col) {
            Ordering::Equal => self.range.get_weight_pair_comment(rank1),
            Ordering::Less => self.range.get_weight_suited_comment(rank1, rank2),
            Ordering::Greater => self.range.get_weight_offsuit_comment(rank1, rank2),
        };
        Ok(st)
    }

    pub fn raw_data(&self) -> Box<[u8]> {
        let mut data = vec![0u8; NUMBER_OF_HOLE_CARDS];
        for i in 0..NUMBER_OF_HOLE_CARDS {
            data[i] = self.range.data[i] as u8;
        }
        data.into()
    }
}
