use log::info;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{PlayerFlopResults, NUM_RANK_FAMILIES};

#[wasm_bindgen]
pub struct FlopSimulationResults {
    //Because of https://stackoverflow.com/questions/68243940/rust-wasm-bindgen-struct-with-string
    //we don't want copy but we have accessors
    pub(crate) all_villians: PlayerFlopResults,
    pub(crate) flop_results: Vec<PlayerFlopResults>,
}

#[wasm_bindgen]
impl FlopSimulationResults {
    //note these player_indexes are the index of active players

    // The rust struct can't get passed via the worker interface, so we need primitive accessors
    pub fn get_perc_family(
        &self,
        active_player_index: Option<usize>,
        street_index: usize,
        family_index: usize,
    ) -> f64 {
        let r = if let Some(p_idx) = active_player_index {
            &self.flop_results[p_idx].street_rank_results[street_index]
        } else {
            &self.all_villians.street_rank_results[street_index]
        };

        r.rank_family_count[family_index] as f64 / r.num_iterations as f64
    }
    pub fn get_perc_family_or_better(
        &self,
        active_player_index: Option<usize>,
        street_index: usize,
        family_index: usize,
    ) -> f64 {
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
    pub fn get_street_draw(
        &self,
        active_player_index: Option<usize>,
        draw_index: usize,
    ) -> Result<JsValue, JsValue> {
        info!("get_street_draw: {} ", draw_index);

        Ok(serde_wasm_bindgen::to_value(
            if let Some(p_idx) = active_player_index {
                &self.flop_results[p_idx].street_draws[draw_index]
            } else {
                &self.all_villians.street_draws[draw_index]
            },
        )?)
    }

    pub fn get_num_players(&self) -> usize {
        self.flop_results.len()
    }

    //Convert from active_player_index to original
    pub fn get_player_index(&self, player_index: usize) -> usize {
        info!("get_player_index: {} ", player_index);

        self.flop_results[player_index].player_index
    }
}
