use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;
//use poker_rs::{core::Hand as CoreHand, holdem::MonteCarloGame};


// const GAMES_COUNT: i32 = 3_000_000;
// const STARTING_HANDS: [&str; 2] = ["Adkh", "8c8s"];

use postflop_solver::Card;
use postflop_solver::Range;
use postflop_solver::card_from_str;
use postflop_solver::Hand;
use postflop_solver::card_pair_to_index;
use postflop_solver::card_to_string;
use postflop_solver::flop_from_str;
use postflop_solver::index_to_card_pair;
use postflop_solver::rank_to_char;

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
    
    try_evaluate();

    try_ranges();
}

//returns AA, AK, 76s
fn cards_to_simple_string(card1: Card, card2: Card) -> Result<String, String> {
    let rank1 = card1 >> 2;
    let rank2 = card2 >> 2;

    let suit1 = card1 & 3;
    let suit2 = card2 & 3;

    if suit1 == suit2 {
        return Ok(format!("{}{}s", rank_to_char(rank1)?, rank_to_char(rank2)?));
    } else {
        return Ok(format!("{}{}", rank_to_char(rank1)?, rank_to_char(rank2)?));
    
    }
}

fn try_ranges() {
    let range1 = "QQ+,AKs".parse::<Range>().unwrap();
    let range2 = "22+,A2+,K2+,Q2s+,Q4o+,J2s+,J5o+,T2s+,T6o+,94s+,97o+,85s+,87o,75s+,76o,65,54,43,32".parse::<Range>().unwrap();
    //all
    let range3: Range = "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap();

    let (range1_weights, _) = range1.get_hands_weights(0);
    let (range2_weights, _) = range2.get_hands_weights(0);
    let (range3_weights, _) = range3.get_hands_weights(0);

    println!("Range1 num hands = {}", range1_weights.len());
    println!("Range2 num hands = {}", range2_weights.len());
    println!("Range3 num hands = {}", range3_weights.len());
    println!("Total num hands = {}\n\n", 52*51 / 2);

    //flop_from_str
    
    let my_hand = Hand::new();
    let my_hand = my_hand.add_card(card_from_str("Kd").unwrap() as usize);
    let my_hand = my_hand.add_card(card_from_str("Th").unwrap() as usize);

    let flop_str = "Ah Ts 5d";
    let flop = flop_from_str(&flop_str).unwrap();

    let dead_cards_mask: u64 =  (1 << flop[0]) | (1 << flop[1]) | (1 << flop[2]) | (1 << my_hand.cards[0]) | (1 << my_hand.cards[1]);

    assert_eq!(5, dead_cards_mask.count_ones());

    let (range1_weights, _) = range1.get_hands_weights(dead_cards_mask);
    let (range2_weights, _) = range2.get_hands_weights(dead_cards_mask);
    let (range3_weights, _) = range3.get_hands_weights(dead_cards_mask);

    println!("Range1 num hands = {}", range1_weights.len());
    println!("Range2 num hands = {}", range2_weights.len());
    println!("Range3 num hands = {}", range3_weights.len());
    println!("Total num hands = {}", 52*51 / 2);

    println!("range 3 {}", range3.to_string());

    //let mut range1 = range1.clone();
    //let mut range2 = range2.clone();
    let mut range3 = range3.clone();

    //range3.set_weight(&[my_hand.cards[0], my_hand.cards[1]], 0.0);
    //range3.set_weight(&[flop[0] as usize, flop[1] as usize, flop[2] as usize], 0.0);

    // for card1 in 0..52 {
    //     for card2 in 0..52 {
            
    //     }
    // }
    // for index in 0..20 {
    //     let (card1, card2) = index_to_card_pair(index);
    //     println!("card1 = {} {} card2 = {} {}", 
    //     card1,
    //     card_to_string(card1 as u8).unwrap(), 
    //     card2,
    //     card_to_string(card2 as u8).unwrap());
    // }

    let mut range3_strings : HashMap<String, u8> = HashMap::new();

    let mut range_index = 0;
    for card1 in 0..=51 {
        for card2 in card1+1..=51 {
            
            let check_index = card_pair_to_index(card1, card2);

            /*println!("card1 = {} {} card2 = {} {}  index = {}, check index = {}", 
            card1, card_to_string(card1 as u8).unwrap(), card2, card_to_string(card2 as u8).unwrap(),
            range_index, check_index);*/

            assert_eq!(check_index, range_index);

            let mut taken = false;

            if card1 == my_hand.cards[0] as u8 || card1 == my_hand.cards[1] as u8 || card2 == my_hand.cards[0] as u8 || card2 == my_hand.cards[1] as u8 {
                taken = true;
            } 

            //same with flop
            if card1 == flop[0] || card1 == flop[1] || card1 == flop[2] || card2 == flop[0] || card2 == flop[1] || card2 == flop[2] {
                taken = true;
                
            }

            if taken {
                range3.set_weight(&[range_index], 0.0);
                
            } else {
                let ss = cards_to_simple_string(card2, card1).unwrap();
                range3_strings.entry(ss).and_modify(|count| *count+=1).or_insert(1);
            }

            range_index += 1;
        }
    }

    assert_eq!(range_index, 1326);
    assert_eq!(range_index, 52*51/2);

    //let flop_range: Range = flop_str.replace(" ", ", ").parse().unwrap();
    // for (hand, _weight) in flop_range.get_hands_weights(dead_cards_mask).0 {
    //     range3.set_weight(&[hand as usize], 0.0);
    // }

    //range3.update_with_singleton(&card_to_string(my_hand.cards[0] as u8).unwrap(), 0.0).unwrap();
    
    let (range3_weights, _) = range3.get_hands_weights(0);

    println!("Range3 num hands = {} and should be {}", range3_weights.len(), 47*46/2);
    println!("range 3 {}", range3.to_string());

    // for s in range3_strings.keys() {
    //     println!("{} {}", s, range3_strings.get(s).unwrap());
    // }

    let new_range_string = range3_strings.keys().join(",");
    //println!("new range string {}", new_range_string);
    //let range3_again = new_range_string.parse::<Range>().unwrap();
    //will be everything again
    //println!("range 3 again {}\n{}", range3_again.to_string(), range3_again.get_hands_weights(0).0.len());
    //# of hands


    //print hole cards and flop again
    println!("Hole Cards: {} {}", card_to_string(my_hand.cards[0] as u8).unwrap(), card_to_string(my_hand.cards[1] as u8).unwrap());
    println!("Flop: {} {} {}", card_to_string(flop[0] as u8).unwrap(), card_to_string(flop[1] as u8).unwrap(), card_to_string(flop[2] as u8).unwrap());
    //go through all the cards and tally stats based on category

    //store for each player 
    let mut category_totals: Vec< RangeEval > = Vec::new();
    category_totals.push( RangeEval::new() );
    category_totals.push( RangeEval::new() );

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

    let mut vil_range_results = RangeEval::new();

    for (card1, card2) in range3.get_hands_weights(0).0 {
        let hand_villian = hand_base.add_card(card1 as usize);
        let hand_villian = hand_villian.add_card(card2 as usize);

        assert_eq!(hand_villian.num_cards, 5);

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

    //now print out the results

    println!("Hero category {}", category_to_string(hero_rank));

    for cat in 0..9 {
        println!("Category {:<20} => Win {:>3} => {:>4.1}% | Lose {:>3} {:>4.1}% Tie {:>3} {:.1}%", 
        category_to_string(cat), 
        vil_range_results.category_winning_hands[cat as usize], 
        vil_range_results.category_winning_hands[cat as usize] as f64 / range3_weights.len() as f64 * 100.0,
        vil_range_results.category_losing_hands[cat as usize],
        vil_range_results.category_losing_hands[cat as usize] as f64 / range3_weights.len() as f64 * 100.0,
        vil_range_results.category_tie_hands[cat as usize], 
        vil_range_results.category_tie_hands[cat as usize] as f64 / range3_weights.len() as f64 * 100.0,
    );

    }

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

fn try_evaluate() {
    //trying code from postflop solver

    println!("Hello {}", card_from_str("Ad").unwrap().to_string());

    let mut hand1 = Hand::new();
    for chunk in &"AdAcTd9d2h3c4d".chars().chunks(2) {
        let sub_str = chunk.collect::<String>();
        println!("sub_str = {}", sub_str);

        hand1 = hand1.add_card(card_from_str(&sub_str).unwrap() as usize);
    }

    println!("\n\n {}  rank {}", hand1.evaluate(), category_to_string(get_hand_category_rank(&hand1)));

    let mut hand2 = Hand::new();
    for chunk in &"AdAcTd9d2h3c5h".chars().chunks(2) {
        let sub_str = chunk.collect::<String>();
        println!("sub_str = {}", sub_str);

        hand2 = hand2.add_card(card_from_str(&sub_str).unwrap() as usize);
    }

    assert!(hand1.evaluate() < hand2.evaluate());

}

// fn old_main() {
//     println!("Hello, world!");

//     let hands = STARTING_HANDS
//         .iter()
//         .map(|s| CoreHand::new_from_str(s).expect("Should be able to create a hand."))
//         .collect();
//     let mut g = MonteCarloGame::new(hands).expect("Should be able to create a game.");
//     let mut wins: [u64; 2] = [0, 0];
//     for _ in 0..GAMES_COUNT {
//         let r = g.simulate();
//         g.reset();
//         wins[r.0.ones().next().unwrap()] += 1
//     }

//     let normalized: Vec<f64> = wins
//         .iter()
//         .map(|cnt| *cnt as f64 / GAMES_COUNT as f64)
//         .collect();

//     println!("Starting Hands =\t{:?}", STARTING_HANDS);
//     println!("Wins =\t\t\t{:?}", wins);
//     println!("Normalized Wins =\t{:?}", normalized);
// }

#[cfg(test)]
mod tests {
    //use std::cmp::Ordering;

    //use poker_rs::core::{Hand, Rankable, Rank};

    // #[test]
    // fn test_3rd_kicker() {
    //     //AA pair, ten, nine, four vs 5
    //     let hand1 =  Hand::new_from_str("AdAcTd9d2h3c4d").unwrap().rank();
    //     let hand2 =  Hand::new_from_str("AdAcTd9d2h3c5h").unwrap().rank();

    //     assert_eq!(hand2.cmp(&hand1), Ordering::Greater);
    //     if let Rank::OnePair(_val) = hand2 {
    //         assert!(true);
    //     } else {
    //         assert!(false);
    //     }

    //     //Test 4th kicker doesn't matter

    //     //K Q T 9 8 7 vs 6
    //     let hand1 = Hand::new_from_str("KdQcTd9d8h7c3d2c").unwrap().rank();

    //     let hand2 = Hand::new_from_str("KhQcTd9d8h6c3d5c").unwrap().rank();

    //     assert_eq!(hand2.cmp(&hand1), Ordering::Equal);

    //     assert_eq!(1, 2);


    // }
}