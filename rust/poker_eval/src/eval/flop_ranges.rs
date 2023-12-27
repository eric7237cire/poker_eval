/*
T2o:0.45,72o

Basically this takes a flop / turn / river
and returns a range of hands that may have callen
a bet / strong bet on the flop / turn / river

strong bet would be top pair, or a good draw, or a decent
poket pair

weak bet would be really anything interesting,
gut shot, overcards, etc.

So each street we have 3 possibilities:
no bet, small bet, large bet

we ignore preflop since people call with anything

on turn we have 3 possibilities frop flop bet
on river we have 6

Flop bet; Turn bet
sb, lb
sb, sb,
sb, no bet
lb, lb
lb, sb
lb, no bet 
no bet, sb
no bet, lb
no bet, no bet

So we need a 9 thing array for river ranges
and a 3 thing array for turn ranges

we do this for the some preconfigured ranges

all
top 75%
top 50%

So we are determining narrowed down turn & river ranges

The cache would then be
the board  (5 cards, maybe 4)


Maybe have this take a decision profile, but we'll start with what
seems reasonable and the most 'fishy'
*/

use std::{rc::Rc, cell::RefCell};

use postflop_solver::{Range, Hand};
use serde::{Deserialize, Serialize};
use crate::{core::BoolRange, Card, board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards}, PartialRankContainer};
const BET_SIZE_COUNT : usize = 3;

const BET_ZERO : usize = 0;
const BET_SMALL : usize = 1;
const BET_LARGE : usize = 2;

#[derive(Serialize, Deserialize)]
pub struct FlopRangeContainer {
    //This is what we start with
    pub flop_range: BoolRange,
    pub turn_ranges: [BoolRange; BET_SIZE_COUNT],
    pub river_ranges: [BoolRange; BET_SIZE_COUNT * BET_SIZE_COUNT],
}

#[derive(Serialize, Deserialize)]
pub struct FlopRanges {
    pub all: FlopRangeContainer,

    pub top_75: FlopRangeContainer,

    pub top_50: FlopRangeContainer,
}

fn narrow_range(board: &[Card], in_range: &BoolRange, bet_size: usize, 
    partial_rank_db: Rc<
    RefCell<EvalCacheWithHcReDb<ProducePartialRankCards, PartialRankContainer>>,
>,) -> BoolRange {

    //bet 0 is nothing, 1 is small bet
    

}