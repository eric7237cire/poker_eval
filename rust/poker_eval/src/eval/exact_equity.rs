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

use itertools::Itertools;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceRank},
    Board, BoolRange, Deck, OldRank, PokerError,
};

//A more direct version of the flop analyze code
fn calc_equity(
    board: Board,
    ranges: &Vec<BoolRange>,
    rank_db: Rc<RefCell<EvalCacheReDb<ProduceRank>>>,
    num_simulations: usize,
) -> Result<Vec<f64>, PokerError> {
    //returns array [52*51/2] = none for impossible or
    // num above / below / equal & total

    let mut deck = Deck::new();

    let mut out = vec![0.0; ranges.len()];

    for _it in 0..num_simulations {
        deck.reset();

        //We need to deal hole cards to each player
        let player_hole_cards = ranges
            .iter()
            .map(|range| {
                //let usable_range = deck.get_available_in_range(range)?;
                //trace!("usable range {}", usable_range.data.count_ones());
                let hole_cards = deck.choose_available_in_range(range).unwrap();

                hole_cards
            })
            .collect_vec();

        let mut extra_board_cards = Vec::with_capacity(5);

        for _ in 0..5 - board.get_num_cards() {
            let card = deck.get_unused_card().unwrap();
            extra_board_cards.push(card);
        }

        let mut rank_instance = rank_db.borrow_mut();

        //do eval
        let ranks = player_hole_cards
            .iter()
            .enumerate()
            .map(|(player_index, hole_cards)| {
                let mut eval_board = Board::new_from_cards(board.as_slice_card());
                for ec in extra_board_cards.iter() {
                    eval_board.add_card(*ec).unwrap();
                }
                eval_board.add_card(hole_cards.get_hi_card()).unwrap();
                eval_board.add_card(hole_cards.get_lo_card()).unwrap();

                eval_board.get_index();

                let rank = rank_instance.get_put(&eval_board).unwrap();

                (rank, player_index)
            })
            .collect_vec();

        let mut count_at_max = 0;
        let mut max_rank: Option<OldRank> = None;

        for (rank, _player_index) in ranks.iter() {
            if max_rank.is_none() || rank > max_rank.as_ref().unwrap() {
                max_rank = Some(*rank);
                count_at_max = 1;
            } else if rank == max_rank.as_ref().unwrap() {
                count_at_max += 1;
            }
        }

        for (rank, player_index) in ranks.iter() {
            if rank == max_rank.as_ref().unwrap() {
                out[*player_index] += 1.0 / count_at_max as f64;
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_equity() {
        //let mut range_1 : BoolRange = "TT+".parse();
    }
}
