

use wasm_bindgen::prelude::wasm_bindgen;
type ResultType = u32;

use crate::Draws;

#[derive(Default)]
pub struct RankResults {
    pub(crate) num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    pub(crate) win_eq: f64,
    pub(crate) tie_eq: f64,

    pub(crate) rank_family_count: [ResultType; 9],
}

#[derive(Default)]
#[wasm_bindgen]
pub struct PlayerFlopResults {
    /*
    This is when evaluating the flop vs the players
    */
    //pub num_iterations: ResultType,
    pub(crate) player_index: usize,

    //hand rankings as of the flop
    //3 of them, flop turn river
    pub(crate) street_rank_results: [RankResults; 3],

    //turn & river
    pub(crate) street_draws: [Draws; 2],
}

#[wasm_bindgen]
impl PlayerFlopResults {
    pub fn new() -> Self {
        let d = Self::default();
        //d.num_iterations = 0;
        d
    }

    
}
