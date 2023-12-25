//Flop Texture: has

// 3 same suit
// 2 same suit, 1 other
// 0, 1 trips
// 0, 1 pair
// Ordering ranks, what are the gaps

//'Partial' rank / draws

// Flush draws
// 2 board, 2 hand
// 3 board, 1 hand

// Straight draws

// hole connected draw -- high
// 8 9 on board 6 7

// hole connected draw -- low
// 8 9 on board T J

// 1gap hole interleaved
// 8 T on board 9 J -- med   targetting 789TJ or 89TJQ
// 8 T on board 7 9 -- low

// 2gap Hole
// 8 J on board 9 T

// Top pair, 2nd pair, 3rd pair, 4th pair etc.

// Best 2 pair, 2nd best, etc.

use std::cmp::max;

use log::trace;
use num_integer::binomial;

use crate::{calc_cards_metrics, Card, CardValue};

pub struct BoardTexture {
    // Highest same suited count, 1 is a raindbow board
    pub same_suited_max_count: u8,

    pub gaps: Vec<u8>, // a flop will have 2

    //56 on board gives straight draw to
    //43 and 78 and 4 7
    //5 7 gives straight draw to 46, 68
    //5 8 gives straight draw to 67

    pub has_trips: bool,
    pub has_pair: bool,
    pub has_two_pair: bool,

    //group values into 3 chunks
    //[A K Q J] [T 9 8 7] [6 5 4 3 2]
    pub high_value_count: u8,
    pub med_value_count: u8,
    pub low_value_count: u8,
}

pub fn calc_board_texture(cards: &[Card]) -> BoardTexture {
    let mut texture = BoardTexture {
        same_suited_max_count: 0,
        gaps: Vec::with_capacity(cards.len() - 1),
        has_trips: false,
        has_pair: false,
        has_two_pair: false,
        high_value_count: 0,
        med_value_count: 0,
        low_value_count: 0,
    };

    let cards_metrics = calc_cards_metrics(cards);

    let mut card_values: Vec<CardValue> = cards.iter().map(|c| c.value).collect();
    //highest value 1st
    card_values.sort_by(|a, b| b.cmp(a));

    //Gap is the difference between the values of the cards
    for i in 1..cards.len() {
        texture.gaps.push(card_values[i].gap(card_values[i - 1]));
    }

    //If highest card is an Ace then also add the gap between it and the lowest card value
    if card_values[0] == CardValue::Ace {
        //2 is == 0 so the distance is the lowest value + 1
        texture.gaps.push(card_values[cards.len() - 1] as u8 + 1);
    }

    //filter out 0 gaps, these don't matter for straights, then return lowest order first

    //T 9 -- 1
    //T 8 -- 2
    //T 7 -- 3  // T [9 8] 7
    //T 6 -- 4 // T [9 8 7] 6

    //The lowest gap distance we care about is 4

    texture.gaps.retain(|&x| x > 0 && x <= 4);
    texture.gaps.sort_by(|a, b| a.cmp(b));

    for card_value in card_values.iter() {
        if *card_value as u8 >= CardValue::Jack as u8 {
            texture.high_value_count += 1;
        } else if *card_value as u8 >= CardValue::Seven as u8 {
            texture.med_value_count += 1;
        } else {
            texture.low_value_count += 1;
        }
    }

    // Find out if there's a flush
    for svs in cards_metrics.suit_value_sets.iter() {
        texture.same_suited_max_count = max(texture.same_suited_max_count, svs.count_ones() as u8);
    }

    if cards_metrics.count_to_value[3] != 0 {
        texture.has_trips = true;
    }
    let pair_count = cards_metrics.count_to_value[2].count_ones();
    if pair_count >= 2 {
        texture.has_two_pair = true;
    } else if pair_count == 1 {
        texture.has_pair = true;
    }

    texture
}

fn compute_cum_sum(dimension: usize, total_dimensions: usize) -> Vec<usize> {

    assert!(total_dimensions >= 2 && total_dimensions <= 7);

    assert!(dimension <= total_dimensions);
    assert!(dimension >= 2);

    //For dimension 1 of 5, the minimum card is 4 because order is increasing
    // 0   1   2   3   4 ; the mins/max values are
    //[0-47]    [1-48]     [2-49]    [3-50]    [4-51]
    //Dim5 Dim4  Dim3  Dim2  Dim1

    //For dimension 1 of 2, the minimum card is 1
    // 0 1

    let min_card_value = total_dimensions - dimension;
    let max_card_value = 52 - dimension;


    let mut cumul_sum = Vec::with_capacity(52);
    //cumul_sum.push(num_this_one);

    for card_value in 0..=min_card_value {
        cumul_sum.push(0);
    }

    //Lets say total dimension = 3, dimension = 2
    // So we have 3 cards total -- X card_value Y
    // 0 1 2 is the index == 0, where card_value == minuvalue
    // 0 1 3 is 1, so @ min value we add nothing

    for card_value in min_card_value+1..=max_card_value {
        let n = 52 - card_value;
        let count = binomial(n, dimension-1);
        let prev_sum = cumul_sum[card_value-1];
        // trace!("Calculating dim {} of {}.  Count of {} x == {} choose {} == {}.  Adding to prev value {}",
        // dimension, total_dimensions, card_value, n, dimension-1, count, prev_sum
        // );
        cumul_sum.push(prev_sum + count);
    }

    for cs in cumul_sum.iter_mut() {
        if *cs > 0 {
           // *cs -= 1;
        }
    }

    cumul_sum
}

fn combinatorial_index(cards: &[usize]) -> usize {

    //we want the smallest index 1st
    let mut cards = cards.to_vec();
    cards.sort();

    //index = Σ ( card[i] * C(remaining_cards - 1, n - i - 1) ) for i = 0 to n-1

    //so if I have 7 x, I want to add
    //how many combinations of C(6, 2) to add 

    let mut index = 0;

    for i in 0..cards.len() {
        //trace!("Card {} is {}", i, cards[i]);
        let num_possible_before = cards[i]; // 0 to card[i] - 1 
        let dim = i+1;
        //Example cards 50 12 5
        // (50 C 3) + (12 C 2) + (5 C 1)
        // # of ways to choose cards 0-49 in  3 cards
        // # of ways to choose cards 0-11 in  2 cards
        // # of ways to choose cards 0-4  in  1 card
        let ncr = combinations(num_possible_before, dim);
        //trace!("Adding {} choose {} == {} to index", num_possible_before, dim, ncr);
        index += ncr;
    }

    // https://en.wikipedia.org/wiki/Combinatorial_number_system
    

    index
}

fn combinations(n: usize, r: usize) -> usize {
    (0..r).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

fn main() {
    let cards_3 = [10, 22, 3];
    println!("Index for 3 cards: {}", combinatorial_index(&cards_3));

    let cards_4 = [10, 22, 3, 45];
    println!("Index for 4 cards: {}", combinatorial_index(&cards_4));

    let cards_5 = [10, 22, 3, 45, 11];
    println!("Index for 5 cards: {}", combinatorial_index(&cards_5));

    let cards_6 = [10, 22, 3, 45, 11, 0];
    println!("Index for 6 cards: {}", combinatorial_index(&cards_6));

    let cards_7 = [10, 22, 3, 45, 11, 0, 51];
    println!("Index for 7 cards: {}", combinatorial_index(&cards_7));
}

#[cfg(test)]
mod tests {

    use postflop_solver::card_pair_to_index;

    use crate::{CardVec, init_test_logger};

    use super::*;

    #[test]
    fn test_cache_indexing() {

        init_test_logger();

        // let mut index_check = 0;
        // for card1 in 0..52 {
        //     for card2 in card1 + 1.. 52 {
                

        //         let idx = combinatorial_index(&vec![card1 as usize, card2 as usize]);
        //         println!("card1: {} card 2: {} == {}, {} == {} + {}", 
        //         card1, card2, card_pair_to_index(card1, card2), index_check,
        //         //THis is the sum formula (51+50 / 2)
        //         card1 as usize * (101 - card1 as usize) / 2 ,
        //         card2
        //         );  
        //         println!("Idx is {} but should be {}", idx, index_check);
        //         assert_eq!(idx, index_check);
        //         index_check += 1;
        //     }
        // }

        //0 X Y has 51 Choose 2 -- 1275
        let cs = compute_cum_sum(3, 3);
        assert_eq!(cs[0], 0);
        assert_eq!(cs[1], 1275); //what's before 1 X Y
        //1 X Y has 50 Choose 2 -- 1225
        assert_eq!(cs[2], 1225+1275); //what's before 2 X Y
        //2 X Y has 49 Choose 2 -- 1176
        assert_eq!(cs[3], 1176+1225+1275); //what's before 3 X Y

        //- 1 X has 50 (52 - 2)
        //- 2 X has 49 (52 - 3)
        let cs = compute_cum_sum(2, 3);
        assert_eq!(cs[0], 0);
        assert_eq!(cs[1], 0);
        assert_eq!(cs[2], 50);
        assert_eq!(cs[3], 50+49);

        let mut index_check = 0;
        // 2 1 0
        // 3 1 0
        for card1 in 0..52 {            
            for card2 in 0 .. card1 {
                for card3 in 0..card2  {
                    let idx = combinatorial_index(&vec![card1 as usize, card2 as usize, card3 as usize]);
                    // println!("{} {} {} ==> Idx is {} but should be {}", 
                    //     card1, card2, card3,
                    //     idx, index_check);
                    assert_eq!(idx, index_check);
                    index_check += 1;
                    
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        let mut index_check = 0;
        for card1 in 0..52 {            
            for card2 in 0 .. card1 {
                for card3 in 0..card2  {
                    for card4 in 0..card3 {
                        for card5 in 0..card4 {
                            for card6 in 0..card5 {
                                for card7 in 0..card6 {

                                    let idx = combinatorial_index(&vec![card1 as usize, card2 as usize, card3 as usize,
                                        card4 as usize, card5 as usize, card6 as usize, card7 as usize]);
                                    assert_eq!(idx, index_check);
                                    index_check += 1;

                                    if index_check % 1_000_000 == 0 {
                                        println!("Index is {}", index_check);
                                    }

                                    assert!( index_check < 100_000_000);
                                }
                            }
                        }
                    }
                    
                    
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        assert_eq!(3000, index_check);


    }

    #[test]
    fn test_board_texture() {
        let cards = CardVec::try_from("3c 2s As").unwrap().0;
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 2);
        assert_eq!(texture.gaps.len(), 2);
        assert_eq!(texture.gaps[0], 1);
        assert_eq!(texture.gaps[1], 1);
        assert_eq!(texture.has_trips, false);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 1);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 2);

        let cards = CardVec::try_from("Ac Ah As").unwrap().0;
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 1);
        assert_eq!(texture.gaps.len(), 0);
        assert_eq!(texture.has_trips, true);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 3);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 0);

        let cards = CardVec::try_from("Qc Kh Qd As Ks").unwrap().0;
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 2);
        assert_eq!(texture.gaps.len(), 2);
        assert_eq!(texture.gaps[0], 1);
        assert_eq!(texture.gaps[1], 1);
        assert_eq!(texture.has_trips, false);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, true);
        assert_eq!(texture.high_value_count, 5);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 0);
    }
}
