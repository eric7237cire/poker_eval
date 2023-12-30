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

use std::{cell::RefCell, rc::Rc, iter::once};

use itertools::Itertools;

use crate::{
    board_eval_cache_redb::{EvalCacheReDb, ProduceRank},
    Board, BoolRange, Deck, OldRank, PokerError, pre_calc::{perfect_hash::load_boomperfect_hash, fast_eval::fast_hand_eval, rank::Rank}, ALL_HOLE_CARDS, ALL_CARDS,
};

//A more direct version of the flop analyze code
// fn calc_equity_slow(
//     board: Board,
//     ranges: &Vec<BoolRange>,
//     rank_db: Rc<RefCell<EvalCacheReDb<ProduceRank>>>,
//     num_simulations: usize,
// ) -> Result<Vec<f64>, PokerError> {
//     //returns array [52*51/2] = none for impossible or
//     // num above / below / equal & total

//     let _hash_func = load_boomperfect_hash();
    
//     let mut deck = Deck::new();

//     let mut out = vec![0.0; ranges.len()];

//     for it in 0..num_simulations {
//         if it % 1000 == 0 {
//             println!("it {}", it);
//         }
//         deck.reset();

//         for c in board.as_slice_card().iter() {
//             deck.set_used_card(*c);
//         }

//         //We need to deal hole cards to each player
//         let player_hole_cards = ranges
//             .iter()
//             .map(|range| {
//                 //let usable_range = deck.get_available_in_range(range)?;
//                 //trace!("usable range {}", usable_range.data.count_ones());
//                 let hole_cards = deck.choose_available_in_range(range).unwrap();

//                 hole_cards
//             })
//             .collect_vec();


//         let mut extra_board_cards = Vec::with_capacity(5);

//         for _ in 0..5 - board.get_num_cards() {
//             let card = deck.get_unused_card().unwrap();
//             extra_board_cards.push(card);
//         }

//         assert_eq!(2 * player_hole_cards.len() + 5, deck.get_number_of_used_cards());

//         let mut rank_instance = rank_db.borrow_mut();

//         //do eval
//         let ranks = player_hole_cards
//             .iter()
//             .enumerate()
//             .map(|(player_index, hole_cards)| {
//                 let mut eval_board = Board::new_from_cards(board.as_slice_card());
//                 for ec in extra_board_cards.iter() {
//                     eval_board.add_card(*ec).unwrap();
//                 }
//                 eval_board.add_card(hole_cards.get_hi_card()).unwrap();
//                 eval_board.add_card(hole_cards.get_lo_card()).unwrap();

//                 eval_board.get_index();

//                 let rank = rank_instance.get_put(&eval_board).unwrap();

//                 (rank, player_index)
//             })
//             .collect_vec();

//         let mut count_at_max = 0;
//         let mut max_rank: Option<OldRank> = None;

//         for (rank, _player_index) in ranks.iter() {
//             if max_rank.is_none() || rank > max_rank.as_ref().unwrap() {
//                 max_rank = Some(*rank);
//                 count_at_max = 1;
//             } else if rank == max_rank.as_ref().unwrap() {
//                 count_at_max += 1;
//             }
//         }

//         for (rank, player_index) in ranks.iter() {
//             if rank == max_rank.as_ref().unwrap() {
//                 out[*player_index] += 1.0 / count_at_max as f64;
//             }
//         }
//     }

//     for i in 0..out.len() {
//         out[i] /= num_simulations as f64;
//     }

//     Ok(out)
// }

pub fn calc_equity(
    board: Board,
    ranges: &Vec<BoolRange>,
    num_simulations: usize,
) -> Result<Vec<f64>, PokerError> {
    //returns array [52*51/2] = none for impossible or
    // num above / below / equal & total

    let hash_func = load_boomperfect_hash();
    
    let mut deck = Deck::new();

    let mut out = vec![0.0; ranges.len()];

    let possible_hole_cards = ranges.iter().map(|r| {
        r.get_all_enabled_holecards()
    }).collect_vec();

    let mut player_hole_cards = vec![ALL_HOLE_CARDS[0]; ranges.len()];

    let mut player_ranks: Vec<Rank> = vec![Rank::lowest_rank(); ranges.len()];

    let mut board_cards = board.as_slice_card().iter().map(|c| *c).collect_vec();

    while board_cards.len() < 5 {
        //just a place holder
        board_cards.push(ALL_CARDS[0]);
    }

    for it in 0..num_simulations {
        if it % 100000 == 0 {
            println!("it {}", it);
        }
        deck.reset();

        for c in board.as_slice_card().iter() {
            deck.set_used_card(*c);
        }

        //We need to deal hole cards to each player
        for p in 0..player_hole_cards.len() {
            player_hole_cards[p] = deck.choose_available_in_range(&possible_hole_cards[p]).unwrap();
        }
        // let player_hole_cards = ranges
        //     .iter()
        //     .map(|range| {
        //         //let usable_range = deck.get_available_in_range(range)?;
        //         //trace!("usable range {}", usable_range.data.count_ones());
        //         let hole_cards = deck.choose_available_in_range(range).unwrap();

        //         hole_cards
        //     })
        //     .collect_vec();

        for board_index in board.get_num_cards()..5 {
            let card = deck.get_unused_card().unwrap();
            board_cards[board_index] = card;
        }

        assert_eq!(2 * player_hole_cards.len() + 5, deck.get_number_of_used_cards());

        //do eval

        let mut max_rank: Option<Rank> = None;
        let mut count_at_max = 0;

        for player_index in 0..player_hole_cards.len() {
            let hole_cards = &player_hole_cards[player_index];
                
            let h1 = once(hole_cards.get_hi_card()).chain(once(hole_cards.get_lo_card()));
            let c_it = board_cards.iter().map(|c| *c).chain(h1);

            let rank = fast_hand_eval(c_it, &hash_func );
            player_ranks[player_index] = rank;

            if max_rank.is_none() || &rank > max_rank.as_ref().unwrap() {
                max_rank = Some(rank);
                count_at_max = 1;
            } else if &rank == max_rank.as_ref().unwrap() {
                count_at_max += 1;
            }
        }

        for player_index in 0..player_hole_cards.len() {
            if &player_ranks[player_index] == max_rank.as_ref().unwrap() {
                out[player_index] += 1.0 / count_at_max as f64;
            }
        }
    }

    for i in 0..out.len() {
        out[i] /= num_simulations as f64;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{Board, board_eval_cache_redb::{EvalCacheReDb, ProduceEvalResult, ProduceRank}};

    use super::*;


    #[test]
    fn test_equity() {

        /*
        cargo test test_equity --release -- --nocapture 
         */
        
        let board: Board = "9d 8h 9c".parse().unwrap();

        let start = Instant::now();

        let ranges: Vec<BoolRange> = vec![
            "Ks6s".parse().unwrap(),
            "22+,A2s+,K2s+,Q2s+,J2s+,T2s+,92s+,82s+,74s+,64s+,54s,A2o+,K2o+,Q2o+,J2o+,T2o+,94o+,85o+,75o+".parse().unwrap(),
            "33+,A2s+,K3s+,Q6s+,J8s+,T9s,A2o+,K6o+,Q8o+,JTo".parse().unwrap(),
        ];

        //let rank_db: EvalCacheReDb<ProduceRank> = EvalCacheReDb::new().unwrap();

        //let shared = Rc::new(RefCell::new(rank_db));

        let results = calc_equity(board, &ranges, 10_000_000).unwrap();

        for i in 0..ranges.len() {
            println!("{}\n{:.2}", ranges[i].to_string(), results[i]*100.0);
        }

        println!("time {:?}", start.elapsed());
        
    }
}
