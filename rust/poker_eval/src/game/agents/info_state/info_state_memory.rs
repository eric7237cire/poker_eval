use std::{
    cell::RefCell,
    cmp::min,
    collections::HashMap,
    fmt::{Display, Formatter},
    fs, mem,
    rc::Rc,
};
use enum_dispatch::enum_dispatch;
use log::info;
use once_cell::sync::Lazy;
use redb::{Database, Error as ReDbError, ReadTransaction, ReadableTable, TableDefinition};

use crate::{
    board_eval_cache_redb::{get_data_path, EvalCacheEnum},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::core::{ActionEnum, GameState, PlayerAction, PlayerState, Round, ChipType},
    monte_carlo_equity::get_equivalent_hole_board,
    pre_calc::NUMBER_OF_SIMPLE_HOLE_CARDS,
    HoleCards, ALL_HOLE_CARDS, Card, PokerError,
};

use crate::game::agents::info_state::{InfoState, InfoStateActionValueType, info_state_actions, InfoStateDbTrait};

//For testing, to have the infostate action values
pub struct InfoStateMemory {
    hash_map: HashMap<InfoState, [InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>,
}

impl InfoStateDbTrait for InfoStateMemory {
    fn get(&self,infostate: &InfoState,) -> Result<Option<[InfoStateActionValueType;
info_state_actions::NUM_ACTIONS]>,PokerError> {
        let v = self.hash_map.get(infostate);
        if v.is_some() {
            Ok(Some(*v.unwrap()))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self,infostate: &InfoState,result:[InfoStateActionValueType;
info_state_actions::NUM_ACTIONS],) -> Result<(),PokerError> {
        let info_state: InfoState = infostate.clone();
        self.hash_map.insert(info_state, result);

        Ok(())
    }
}

impl InfoStateMemory {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }
}