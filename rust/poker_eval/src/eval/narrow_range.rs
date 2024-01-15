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

use crate::{
    calc_board_texture,
    core::BoolRange,
    likes_hands::{likes_hand, LikesHandLevel},
    monte_carlo_equity::calc_equity,
    partial_rank_cards,
    pre_calc::fast_eval::fast_hand_eval,
    Board, PokerError, ALL_HOLE_CARDS,
};

use boomphf::Mphf;
use itertools::Itertools;
use log::{debug, trace};

pub fn narrow_range_by_equity(
    range_to_narrow: &BoolRange,
    opponent_ranges: &[BoolRange],
    min_equity: f64,
    board: &Board,
    num_simulations: usize,
) -> BoolRange {
    let mut narrowed_range = BoolRange::default();

    //we'll calc equity on every hole card against the opponent ranges

    let mut all_ranges: Vec<BoolRange> = Vec::with_capacity(opponent_ranges.len() + 1);
    all_ranges.push(BoolRange::default());
    all_ranges.extend(opponent_ranges.iter().cloned());

    let hole_card_indexes = range_to_narrow.data.iter_ones().collect_vec();

    for hci in hole_card_indexes.iter() {
        if *hci >= ALL_HOLE_CARDS.len() {
            break;
        }

        //if these hole cards are impossible given the board, skip
        if board.intersects_holecards(&ALL_HOLE_CARDS[*hci]) {
            continue;
        }

        all_ranges[0].data.fill(false);
        all_ranges[0].data.set(*hci, true);
        let results = calc_equity(board, &all_ranges, num_simulations);

        match results {
            Err(e) => {
                let hc = ALL_HOLE_CARDS[*hci];
                debug!("Unable to calculate {}, error: {}", &hc, e);
                continue;
            }
            Ok(results) => {
                trace!(
                    "Equity was {:.2} for {} in board {}",
                    results[0],
                    ALL_HOLE_CARDS[*hci],
                    &board
                );
                if results[0] >= min_equity {
                    narrowed_range.data.set(*hci, true);
                }
            }
        }
    }

    narrowed_range
}

pub fn narrow_range_by_pref(
    range_to_narrow: &BoolRange,
    min_likes_hand_level: LikesHandLevel,
    board: &Board,
    num_players: u8,
    hash_func: &Mphf<u32>,
) -> Result<BoolRange, PokerError> {
    let mut narrowed_range = BoolRange::default();

    let hole_card_indexes = range_to_narrow.data.iter_ones().collect_vec();

    for hci in hole_card_indexes.iter() {
        if *hci >= ALL_HOLE_CARDS.len() {
            break;
        }

        //if these hole cards are impossible given the board, skip
        if board.intersects_holecards(&ALL_HOLE_CARDS[*hci]) {
            continue;
        }

        let hc = ALL_HOLE_CARDS[*hci];

        let prc = partial_rank_cards(&hc, board.as_slice_card());

        let board_texture = calc_board_texture(board.as_slice_card());

        let rank = fast_hand_eval(board.get_iter().chain(hc.iter()), hash_func);

        let likes_hand_response = likes_hand(&prc, &board_texture, &rank, board, &hc, num_players)?;

        if likes_hand_response.likes_hand >= min_likes_hand_level {
            narrowed_range.data.set(*hci, true);
        }
    }

    Ok(narrowed_range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_test_logger, Board, BoolRange};

    #[test]
    fn test_narrow_range() {
        /*
        cargo test --lib test_narrow_range --release -- --nocapture
         */
        let board: Board = "Jh Td 7c".parse().unwrap();
        init_test_logger();

        //let hero_range: BoolRange = "Jd9s".parse().unwrap();
        let hero_range = "22+,A2+,K2+,Q3s+,Q5o+,J7s+,J8o+,T7s+,T8o+,97s+,98o,87s,76s,65s,54s"
            .parse()
            .unwrap();
        let other_guy: BoolRange = "22+,A2+,K2+,Q2s+,Q3o+,J3s+,J6o+,T5s+,T7o+,97s+,98o,87s"
            .parse()
            .unwrap();
        let to_narrow: BoolRange = "22+,A2+,K2+,Q2+,J2+,T2s+,T3o+,92s+,95o+,84s+,86o+,74s+,76o,65s"
            .parse()
            .unwrap();

        let narrowed_range =
            narrow_range_by_equity(&to_narrow, &[hero_range, other_guy], 0.25, &board, 1);

        println!("Narrowed range:\n{}", narrowed_range.to_string());

        // let board: Board = "Jh 6h 5d".parse().unwrap();
        // let to_narrow : BoolRange = "22+,A2+,K2+,Q2+,J2+,T2s+,T5o+,95s+,96o+,85s+,87o,76s".parse().unwrap();
        // let op1 : BoolRange = "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap();
        // let op2 : BoolRange = "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap();

        // let narrowed_range = narrow_range_by_equity(&to_narrow, &[op1, op2], 0.35, &board, 3_000);

        // println!("Narrowed range:\n{}", narrowed_range.to_string());
    }
}
