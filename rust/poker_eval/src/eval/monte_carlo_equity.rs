/*

*/

use std::{cmp::Ordering, iter::once};

use itertools::Itertools;
use log::trace;

use crate::{
    pre_calc::{
        fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash, rank::Rank, NUMBER_OF_SUITS,
    },
    Board, BoolRange, Card, Deck, HoleCards, PokerError, Suit, ALL_CARDS, ALL_HOLE_CARDS,
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

pub fn calc_equity_vs_random(
    board: &Board,
    //The player ranges we are calculating equity for
    hole_cards: &HoleCards,
    num_players: usize,
    num_simulations: usize,
) -> Result<f64, PokerError> {
    assert!(num_players >= 2 && num_players <= 10);
    let hash_func = load_boomperfect_hash();

    let mut deck = Deck::new();

    let mut out = 0.0;

    let mut player_hole_cards = vec![ALL_HOLE_CARDS[0]; num_players];

    let mut player_ranks: Vec<Rank> = vec![Rank::lowest_rank(); num_players];

    let mut board_cards = board.as_slice_card().iter().map(|c| *c).collect_vec();

    while board_cards.len() < 5 {
        //just a place holder
        board_cards.push(ALL_CARDS[0]);
    }

    player_hole_cards[0] = hole_cards.clone();

    for it in 0..num_simulations {
        if it % 10_000 == 0 && it > 0 {
            trace!("it {}", it);
        }

        deck.reset();

        for c in board.as_slice_card().iter() {
            deck.set_used_card(*c);
        }
        deck.set_used_card(player_hole_cards[0].get_hi_card());
        deck.set_used_card(player_hole_cards[0].get_lo_card());

        //We need to deal hole cards to each player
        for player_index in 1..num_players {
            // trace!(
            //     "Choosing range for {} with {} possibilities",
            //     p,
            //     possible_hole_cards[p].len()
            // );

            //This takes into account the used cards
            let card1 = deck.get_unused_card().unwrap();
            let card2 = deck.get_unused_card().unwrap();
            player_hole_cards[player_index] = HoleCards::new(card1, card2)?;
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

        if player_ranks[0] == max_rank.unwrap() {
            out += 1.0 / count_at_max as f64;
        }
    }

    out /= num_simulations as f64;

    Ok(out)
}

//If we are doing hole cards + board vs pure random ranges we can reduce the search space
pub fn get_equivalent_hole_board(hole_cards: &HoleCards, board: &[Card]) -> (HoleCards, Board) {
    //Mapping

    /*
    Lo card => club
    if hi card different suit => diamond

    In the flop we have; for 5 cards
    5 0
    4 1
    3 2
    3 1 1
    2 1 1 1
    2 2 1

    Starting from most numerous, suit value breaks ties
    Non matching suit 1 => Heart
    Non matching suit 2 => Spades

    (if suited)
    Non matching suit 3 => Diamonds

    Simpler is we sort the suits, 1st by is lo card, is hi card, then by frequency
    */

    let mut suit_frequency = vec![0; NUMBER_OF_SUITS];

    for c in board.iter() {
        suit_frequency[c.suit as usize] += 1;
    }

    //to tweak the sorting, set freq really hi for hole card1 and 2
    suit_frequency[hole_cards.get_lo_card().suit as usize] = 10;
    suit_frequency[hole_cards.get_hi_card().suit as usize] = 9;

    let mut suits = Suit::suits();
    suits.sort_by(|suit1, suit2| {
        if suit1 == suit2 {
            return Ordering::Equal;
        }

        let freq1 = suit_frequency[*suit1 as usize];
        let freq2 = suit_frequency[*suit2 as usize];

        if freq1 != freq2 {
            freq2.cmp(&freq1)
        } else {
            suit1.cmp(suit2)
        }
    });

    //Ex: suits = Heart, Spade, Diamond, Club
    //Mapping it to clubs/diamonds/heart/spade
    //mapping[club] = Spade

    //Now we need an array where
    //mapping[suit] = the index in suits, which is the new mapping
    let mut mapping = vec![Suit::Club; NUMBER_OF_SUITS];

    for index in 0..NUMBER_OF_SUITS {
        let map_target: Suit = Suit::try_from(index as u8).unwrap();
        let map_source = suits[index];
        mapping[map_source as usize] = map_target;
    }

    let mut board_cards = Vec::with_capacity(board.len());
    for c in board.iter() {
        board_cards.push(Card::new(c.value, mapping[c.suit as usize]));
    }

    let new_hole_cards = HoleCards::new(
        swap_suit(
            hole_cards.get_hi_card(),
            mapping[hole_cards.get_hi_card().suit as usize],
        ),
        swap_suit(
            hole_cards.get_lo_card(),
            mapping[hole_cards.get_lo_card().suit as usize],
        ),
    )
    .unwrap();

    (new_hole_cards, Board::new_from_cards(&board_cards))
}

fn swap_suit(card: Card, to_suit: Suit) -> Card {
    Card::new(card.value, to_suit)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use log::info;

    use crate::init_test_logger;

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

    fn compare_expected_actual(
        hole_cards: &HoleCards,
        board: &Board,
        expected_hole_cards: &HoleCards,
        expected_board: &Board,
    ) {
        let (actual_hole_cards, actual_board) =
            get_equivalent_hole_board(&hole_cards, board.as_slice_card());

        info!(
            "hole cards {} should be {} and are {}",
            hole_cards, expected_hole_cards, actual_hole_cards
        );
        info!(
            "board {} should be {} and is {}",
            board, expected_board, actual_board
        );

        assert_eq!(*expected_hole_cards, actual_hole_cards);
        assert_eq!(expected_board.as_slice_card(), actual_board.as_slice_card());
    }

    #[test]
    fn test_get_equivalent_hole_board() {
        //cargo test --lib test_get_equivalent_hole_board
        //clubs/diamonds/heart/spade

        init_test_logger();

        let hole_cards: HoleCards = "Ac Th".parse().unwrap();
        let board: Board = "Js Qh 2h 3c 4c".parse().unwrap();

        let expected_hole_cards: HoleCards = "Ad Tc".parse().unwrap();
        let expected_board: Board = "Jh Qc 2c 3d 4d".parse().unwrap();

        compare_expected_actual(&hole_cards, &board, &expected_hole_cards, &expected_board);

        let hole_cards: HoleCards = "Ac Ad".parse().unwrap();
        let board: Board = "Kc Qc Jc".parse().unwrap();

        let expected_hole_cards: HoleCards = "Ac Ad".parse().unwrap();
        let expected_board: Board = "Kc Qc Jc".parse().unwrap();

        assert_eq!(hole_cards.get_lo_card().suit, Suit::Club);

        compare_expected_actual(&hole_cards, &board, &expected_hole_cards, &expected_board);

        let board: Board = "Kd Qd Jd".parse().unwrap();
        let expected_board: Board = "Kd Qd Jd".parse().unwrap();

        compare_expected_actual(&hole_cards, &board, &expected_hole_cards, &expected_board);

        let hole_cards: HoleCards = "Ac Ad".parse().unwrap();
        let board: Board = "Ks Qs Js".parse().unwrap();

        let expected_hole_cards: HoleCards = "Ac Ad".parse().unwrap();
        let expected_board: Board = "Kh Qh Jh".parse().unwrap();

        compare_expected_actual(&hole_cards, &board, &expected_hole_cards, &expected_board);

        let hole_cards: HoleCards = "7h 2s".parse().unwrap();
        let board: Board = "Kc Js Qc Ts 3d".parse().unwrap();

        let expected_hole_cards: HoleCards = "2c 7d".parse().unwrap();
        let expected_board: Board = "Kh Jc Qh Tc 3s".parse().unwrap();

        compare_expected_actual(&hole_cards, &board, &expected_hole_cards, &expected_board);
    }
}
