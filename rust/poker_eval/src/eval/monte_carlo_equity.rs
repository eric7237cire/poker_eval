/*

*/

use std::iter::once;

use itertools::Itertools;
use log::trace;

use crate::{
    pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash, rank::Rank},
    Board, BoolRange, Deck, HoleCards, PokerError, ALL_CARDS, ALL_HOLE_CARDS,
};

/*

Returns equity between 0 and 1 for each range given

For each simulation, a win counts as 1
A tie with 2 players counts as 0.5 each
A tie with 3 players counts as 0.33 each, etc.

*/
pub fn calc_equity(
    board: &Board,
    //The player ranges we are calculating equity for
    ranges: &Vec<BoolRange>,
    num_simulations: usize,
) -> Result<Vec<f64>, PokerError> {
    let hash_func = load_boomperfect_hash();

    let mut deck = Deck::new();

    let mut out = vec![0.0; ranges.len()];

    //Store the player index because we want to pick the most restrictive hole cards first
    let possible_hole_cards: Vec<(usize, Vec<HoleCards>)> = {
        let mut pv = ranges
            .iter()
            .enumerate()
            //These are irrespective of used cards, we try later on which hole cards are still valid
            .map(|(player_index, r)| (player_index, r.get_all_enabled_holecards()))
            .collect_vec();

        pv.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        pv
    };
    //trace!("Get possible hole cards done ");

    let mut player_hole_cards = vec![ALL_HOLE_CARDS[0]; ranges.len()];

    let mut player_ranks: Vec<Rank> = vec![Rank::lowest_rank(); ranges.len()];

    let mut board_cards = board.as_slice_card().iter().map(|c| *c).collect_vec();

    while board_cards.len() < 5 {
        //just a place holder
        board_cards.push(ALL_CARDS[0]);
    }

    for it in 0..num_simulations {
        if it % 10_000 == 0 && it > 0 {
            trace!("it {}", it);
        }
        deck.reset();

        for c in board.as_slice_card().iter() {
            deck.set_used_card(*c);
        }

        //We need to deal hole cards to each player
        for (player_index, player_possible_hold_cards) in possible_hole_cards.iter() {
            // trace!(
            //     "Choosing range for {} with {} possibilities",
            //     p,
            //     possible_hole_cards[p].len()
            // );

            //This takes into account the used cards
            player_hole_cards[*player_index] =
                deck.choose_available_in_range(player_possible_hold_cards)?;
        }

        for board_index in board.get_num_cards()..5 {
            let card = deck.get_unused_card().unwrap();
            board_cards[board_index] = card;
        }

        assert_eq!(
            2 * player_hole_cards.len() + 5,
            deck.get_number_of_used_cards()
        );

        //do eval

        let mut max_rank: Option<Rank> = None;
        let mut count_at_max = 0;

        for player_index in 0..player_hole_cards.len() {
            let hole_cards = &player_hole_cards[player_index];

            let h1 = once(hole_cards.get_hi_card()).chain(once(hole_cards.get_lo_card()));
            let c_it = board_cards.iter().map(|c| *c).chain(h1);

            let rank = fast_hand_eval(c_it, &hash_func);
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

        let results = calc_equity(&board, &ranges, 1_000).unwrap();

        for i in 0..ranges.len() {
            println!("{}\n{:.2}", ranges[i].to_string(), results[i] * 100.0);
        }

        println!("time {:?}", start.elapsed());
    }

    #[test]
    fn test_single_card_ranges() {
        let board: Board = "9d 8h 9c".parse().unwrap();

        let start = Instant::now();

        let ranges: Vec<BoolRange> = vec![
            "Ac7s,Ac6s,72".parse().unwrap(),
            "Ks6s".parse().unwrap(),
            "As7s".parse().unwrap(),
        ];

        //let rank_db: EvalCacheReDb<ProduceRank> = EvalCacheReDb::new().unwrap();

        //let shared = Rc::new(RefCell::new(rank_db));

        let results = calc_equity(&board, &ranges, 100).unwrap();

        for i in 0..ranges.len() {
            println!("{}\n{:.2}", ranges[i].to_string(), results[i] * 100.0);
        }

        println!("time {:?}", start.elapsed());

        assert!(results[2] > results[1]);
        assert!(results[1] > results[0]);
    }
}
