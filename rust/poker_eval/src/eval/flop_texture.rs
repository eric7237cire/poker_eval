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

fn combinatorial_index(cards: &[usize]) -> usize {

    //we want the smallest index 1st
    let mut cards = cards.to_vec();
    cards.sort();

    
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
        let ncr = binomial(num_possible_before, dim);
        //trace!("Adding {} choose {} == {} to index", num_possible_before, dim, ncr);
        index += ncr;
    }

    // https://en.wikipedia.org/wiki/Combinatorial_number_system
    // https://math.stackexchange.com/questions/1227409/indexing-all-combinations-without-making-list
    

    index
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
