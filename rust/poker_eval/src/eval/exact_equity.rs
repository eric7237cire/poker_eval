/*

A flop

Take all hole cards (52*51/2)

rank them

vs 1 player

prob is num below / num possible hole cards

low => hi
o o o H x x 

prob winning is 3 / 5 * 2/ 4


First thing we need is 

given hole cards + flop

or just flop + t + r
rank all the hole cards

*/

use std::{cell::RefCell, rc::Rc};

use crate::{board_eval_cache_redb::{EvalCacheReDb, ProduceRank}, Board};


fn rank_all_hole_cards(board: Board, rank_db: Rc<RefCell<EvalCacheReDb<ProduceRank>>>,) {
    //returns array [52*51/2] = none for impossible or
    // num above / below / equal & total
}