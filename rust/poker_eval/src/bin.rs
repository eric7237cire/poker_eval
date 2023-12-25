//use poker_rs::{core::Hand as CoreHand, holdem::MonteCarloGame};

// const GAMES_COUNT: i32 = 3_000_000;
// const STARTING_HANDS: [&str; 2] = ["Adkh", "8c8s"];

use postflop_solver::card_from_str;
use postflop_solver::card_pair_to_index;
use postflop_solver::card_to_string;
use postflop_solver::flop_from_str;

use postflop_solver::Hand;
use postflop_solver::Range;

mod game;
pub use game::*;
mod eval;
pub use eval::*;
mod core;
pub use core::*;

fn category_to_string(rank: i32) -> &'static str {
    match rank {
        0 => "High Card",
        1 => "One Pair",
        2 => "Two Pair",
        3 => "Three of a Kind",
        4 => "Straight",
        5 => "Flush",
        6 => "Full House",
        7 => "Four of a Kind",
        8 => "Straight Flush",
        _ => "Unknown",
    }
}

fn get_hand_category_rank(hand: &Hand) -> i32 {
    let eval = hand.evaluate_internal();
    let rank = eval >> 26;

    rank
}

//1 is a way to detech flush draws (with strength indicators)
//over pairs
//str8 draw, & gut shot
//mid pair, top pair, lowest pair

//then simualte using 2, 3, 4 players the frequency

//but first do a simulator of n runs with a flop of you vs 2 other players range

fn main() {
    //try_evaluate();

    //try_ranges();

    run_simul();
}

fn run_simul() {
    //flop_from_str

    let my_hand = Hand::new();
    let my_hand = my_hand.add_card(card_from_str("Kd").unwrap() as usize);
    let my_hand = my_hand.add_card(card_from_str("Th").unwrap() as usize);

    let flop_str = "Ah Ts 5d";
    let flop = flop_from_str(&flop_str).unwrap();

    let mut villain_ranges: Vec<Range> = vec![
        //all
        //"22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap(),

        //75%
        //"22+, A2s+, K2s+, Q2s+, J2s+, T2s+, 92s+, 82s+, 72s+, 62s+, 52s+, 42s+, A2o+, K2o+, Q2o+, J4o+, T6o+, 96o+, 86o+, 76o".parse().unwrap(),

        //50%
        "22+, A2s+, K2s+, Q2s+, J2s+, T5s+, 96s+, 86s+, 75s+, A2o+, K5o+, Q7o+, J8o+, T8o+"
            .parse()
            .unwrap(),
        //25%
        "55+, A2s+, K5s+, Q8s+, J8s+, T9s, A8o+, K9o+, QTo+, JTo"
            .parse()
            .unwrap(),
    ];

    let dead_cards_mask: u64 = (1 << flop[0])
        | (1 << flop[1])
        | (1 << flop[2])
        | (1 << my_hand.cards[0])
        | (1 << my_hand.cards[1]);

    assert_eq!(5, dead_cards_mask.count_ones());

    for range in villain_ranges.iter() {
        let (range_weights, _) = range.get_hands_weights(dead_cards_mask);
        println!(
            "Range num hands = {} {:.1}%",
            range_weights.len(),
            range_weights.len() as f64 / 1326.0 * 100.0
        );
    }

    //print hole cards and flop again
    println!(
        "Hole Cards: {} {}",
        card_to_string(my_hand.cards[0] as u8).unwrap(),
        card_to_string(my_hand.cards[1] as u8).unwrap()
    );
    println!(
        "Flop: {} {} {}",
        card_to_string(flop[0] as u8).unwrap(),
        card_to_string(flop[1] as u8).unwrap(),
        card_to_string(flop[2] as u8).unwrap()
    );
    //go through all the cards and tally stats based on category

    //store for each player
    let mut category_totals: Vec<RangeEval> = Vec::new();
    category_totals.push(RangeEval::new());
    category_totals.push(RangeEval::new());

    //let mut hero_range: Range = format!("{}{}", card_to_string(my_hand.cards[0] as u8).unwrap(), card_to_string(my_hand.cards[1] as u8).unwrap()).parse().unwrap();

    //now evaluate each hand in the range
    let hand_base = Hand::new();
    let hand_base = hand_base.add_card(flop[0] as usize);
    let hand_base = hand_base.add_card(flop[1] as usize);
    let hand_base = hand_base.add_card(flop[2] as usize);

    let hand_hero = hand_base.add_card(my_hand.cards[0] as usize);
    let hand_hero = hand_hero.add_card(my_hand.cards[1] as usize);

    let hero_rank = get_hand_category_rank(&hand_hero);
    let hero_eval = hand_hero.evaluate_internal();

    let cards_to_remove: Vec<u8> = vec![
        my_hand.cards[0] as u8,
        my_hand.cards[1] as u8,
        flop[0],
        flop[1],
        flop[2],
    ];
    //let cards_to_remove_usize = cards_to_remove.iter().map(|c| *c as usize).collect::<Vec<usize>>();
    let flop_card_range_indexes = get_indexes_for_cards(&cards_to_remove);
    for range in villain_ranges.iter_mut() {
        remove_cards_from_range(range, &flop_card_range_indexes);
    }

    //now print out the results

    println!("Hero category {}", category_to_string(hero_rank));

    for (vil_idx, vil_range) in villain_ranges.iter().enumerate() {
        let mut vil_results = RangeEval::new();
        eval_villian_range(&mut vil_results, hand_base, vil_range, hero_eval);
        let num_hands = vil_range.get_hands_weights(0).0.len();
        println!("\n\n*Flop* Villian {} with {}\n", vil_idx + 1, num_hands);
        print_villian_range_results(&vil_results, num_hands);
    }

    //Now simulate all remaining turn cards
    //let mut turn_cards: Vec<Card> = Vec::new();

    let mut villian_results_turn = villain_ranges
        .iter()
        .map(|_| RangeEval::new())
        .collect::<Vec<RangeEval>>();
    let mut villian_results_river = villain_ranges
        .iter()
        .map(|_| RangeEval::new())
        .collect::<Vec<RangeEval>>();

    for turn_card in 0..=51 {
        if cards_to_remove.contains(&turn_card) {
            continue;
        }
        //println!("Turn card {}", card_to_string(turn_card as u8).unwrap());

        let turn_card_indexes = get_indexes_for_cards(&vec![turn_card]);

        let turn_hero_hand = hand_hero.add_card(turn_card as usize);
        let turn_hero_eval = turn_hero_hand.evaluate_internal();

        for (villian_index, villian_range) in villain_ranges.iter_mut().enumerate() {
            let removed_indexes = remove_cards_from_range(villian_range, &turn_card_indexes);
            //assert!(removed_indexes.len() > 0);

            eval_villian_range(
                &mut villian_results_turn[villian_index],
                hand_base.add_card(turn_card as usize),
                &villian_range,
                turn_hero_eval,
            );

            //add them back
            add_cards_from_range(villian_range, &removed_indexes);
        }

        for river_card in 0..=51 {
            if cards_to_remove.contains(&river_card) {
                continue;
            }
            if river_card == turn_card {
                continue;
            }
            //println!("River card {}", card_to_string(river_card as u8).unwrap());

            let river_card_indexes = get_indexes_for_cards(&vec![river_card]);

            let river_hero_hand = turn_hero_hand.add_card(river_card as usize);
            assert_eq!(7, river_hero_hand.num_cards);

            let river_hero_eval = river_hero_hand.evaluate_internal();

            for (villian_index, villian_range) in villain_ranges.iter_mut().enumerate() {
                let removed_indexes_turn =
                    remove_cards_from_range(villian_range, &turn_card_indexes);
                //assert!(removed_indexes_turn.len() > 0);

                let removed_indexes_river =
                    remove_cards_from_range(villian_range, &river_card_indexes);
                //Sometimes with pocket pairs we already removed it
                //assert!(removed_indexes_river.len() > 0);

                let villian_hand = hand_base
                    .add_card(turn_card as usize)
                    .add_card(river_card as usize);
                assert_eq!(5, villian_hand.num_cards);

                eval_villian_range(
                    &mut villian_results_river[villian_index],
                    villian_hand,
                    &villian_range,
                    river_hero_eval,
                );

                //add them back
                add_cards_from_range(villian_range, &removed_indexes_river);
                add_cards_from_range(villian_range, &removed_indexes_turn);
            }
        }
    }

    for (villian_index, villian_results) in villian_results_turn.iter().enumerate() {
        //let num_hands = villain_ranges[villian_index].get_hands_weights(0).0.len();
        println!("\n\n*Turn* Villian {}\n", villian_index + 1);
        print_villian_range_results(
            &villian_results,
            villian_results.category_winning_hands.iter().sum::<u32>() as usize
                + villian_results.category_losing_hands.iter().sum::<u32>() as usize
                + villian_results.category_tie_hands.iter().sum::<u32>() as usize,
        );
    }

    for (villian_index, villian_results) in villian_results_river.iter().enumerate() {
        //let num_hands = villain_ranges[villian_index].get_hands_weights(0).0.len();
        println!("\n\n*River* Villian {}\n", villian_index + 1);
        print_villian_range_results(
            &villian_results,
            villian_results.category_winning_hands.iter().sum::<u32>() as usize
                + villian_results.category_losing_hands.iter().sum::<u32>() as usize
                + villian_results.category_tie_hands.iter().sum::<u32>() as usize,
        );
    }
}

fn get_indexes_for_cards(cards: &Vec<u8>) -> Vec<usize> {
    let mut indexes: Vec<usize> = Vec::new();

    for card1 in 0..=51 {
        for card2 in card1 + 1..=51 {
            if cards.contains(&card1) || cards.contains(&card2) {
                indexes.push(card_pair_to_index(card1, card2));
            }
        }
    }

    indexes
}

//Returns indexes actually removed
fn remove_cards_from_range(range: &mut Range, card_indexes: &Vec<usize>) -> Vec<usize> {
    let mut removed_indexes: Vec<usize> = Vec::with_capacity(7);

    for card_index in card_indexes {
        if range.data[*card_index] <= 0.0 {
            continue;
        }

        removed_indexes.push(*card_index);
        range.data[*card_index] = 0.0;
    }

    removed_indexes
}

fn add_cards_from_range(range: &mut Range, card_indexes: &Vec<usize>) {
    for card_index in card_indexes {
        assert_eq!(range.data[*card_index], 0.0);

        range.data[*card_index] = 1.0;
    }
}
/*
fn remove_cards_from_range(range: &mut Range, cards: &Vec<Card>) {

    let mut range_index = 0;
    for card1 in 0..=51 {
        for card2 in card1+1..=51 {

            let check_index = card_pair_to_index(card1, card2);

            /*println!("card1 = {} {} card2 = {} {}  index = {}, check index = {}",
            card1, card_to_string(card1 as u8).unwrap(), card2, card_to_string(card2 as u8).unwrap(),
            range_index, check_index);*/

            assert_eq!(check_index, range_index);

            let taken = cards.contains(&card1) || cards.contains(&card2);


            if taken {
                range.set_weight(&[range_index], 0.0);

            }

            range_index += 1;
        }
    }

}*/

fn eval_villian_range(
    vil_range_results: &mut RangeEval,
    hand_base: Hand,
    vil_range: &Range,
    hero_eval: i32,
) {
    for (card1, card2) in vil_range.get_hands_weights(0).0 {
        let hand_villian = hand_base.add_card(card1 as usize);
        let hand_villian = hand_villian.add_card(card2 as usize);

        assert!(hand_villian.num_cards >= 5 && hand_villian.num_cards <= 7);

        let vil_rank = get_hand_category_rank(&hand_villian);

        assert!(vil_rank >= 0);
        assert!(vil_rank <= 8);
        let vil_eval = hand_villian.evaluate_internal();

        let is_winning = vil_eval > hero_eval;
        let is_losing = vil_eval < hero_eval;
        let is_tied = vil_eval == hero_eval;

        if is_winning {
            vil_range_results.category_winning_hands[vil_rank as usize] += 1;
        } else if is_losing {
            vil_range_results.category_losing_hands[vil_rank as usize] += 1;
        } else if is_tied {
            vil_range_results.category_tie_hands[vil_rank as usize] += 1;
        }
    }
}

fn print_villian_range_results(vil_range_results: &RangeEval, range_total: usize) {
    let mut win_sum = 0;
    let mut lose_sum = 0;
    let mut tie_sum = 0;

    for cat in 0..9 {
        println!(
            "Category {:<20} => Win {:>5} => {:>4.1}% | Lose {:>5} {:>4.1}% Tie {:>4} {:.1}%",
            category_to_string(cat),
            vil_range_results.category_winning_hands[cat as usize],
            vil_range_results.category_winning_hands[cat as usize] as f64 / range_total as f64
                * 100.0,
            vil_range_results.category_losing_hands[cat as usize],
            vil_range_results.category_losing_hands[cat as usize] as f64 / range_total as f64
                * 100.0,
            vil_range_results.category_tie_hands[cat as usize],
            vil_range_results.category_tie_hands[cat as usize] as f64 / range_total as f64 * 100.0,
        );

        win_sum += vil_range_results.category_winning_hands[cat as usize];
        lose_sum += vil_range_results.category_losing_hands[cat as usize];
        tie_sum += vil_range_results.category_tie_hands[cat as usize];
    }

    println!("Total                         => Win {:>5} => {:>4.1}% | Lose {:>5} {:>4.1}% Tie {:>4} {:.1}%", 
        win_sum, 
        win_sum as f64 / range_total as f64 * 100.0,
        lose_sum,
        lose_sum as f64 / range_total as f64 * 100.0,
        tie_sum, 
        tie_sum as f64 / range_total as f64 * 100.0,
        );
}

struct RangeEval {
    category_winning_hands: [u32; 9],
    category_tie_hands: [u32; 9],
    category_losing_hands: [u32; 9],
}

impl RangeEval {
    fn new() -> RangeEval {
        RangeEval {
            category_winning_hands: [0; 9],
            category_losing_hands: [0; 9],
            category_tie_hands: [0; 9],
        }
    }
}

#[cfg(test)]
mod tests {}
