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

use crate::game::agents::info_state::{InfoState, InfoStateDb, InfoStateMemory, InfoStateActionValueType, info_state_actions};

#[enum_dispatch]
pub enum InfoStateDbEnum {
    InfoStateDb,
    InfoStateMemory
}

#[enum_dispatch(InfoStateDbEnum)]
pub trait InfoStateDbTrait {
    fn get(
        &self,
        infostate: &InfoState,
    ) -> Result<Option<[InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>, PokerError>;

    fn put(
        &mut self,
        infostate: &InfoState,
        result: [InfoStateActionValueType; info_state_actions::NUM_ACTIONS],
    ) -> Result<(), PokerError>;
}
