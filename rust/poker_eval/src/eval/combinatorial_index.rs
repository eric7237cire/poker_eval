use std::collections::HashMap;

use num_integer::binomial;

use crate::Card;

pub struct CombinatorialIndex {
    //Our key is like 0-52 and 0-7 so it fits in a u16
    //Value max is 133_784_560 so it fits in a u32
    cache: HashMap<u16, u32>,
}

impl CombinatorialIndex {
    pub fn new() -> Self {
        CombinatorialIndex {
            cache: HashMap::new(),
        }
    }

    fn get_binomial(&mut self, n: u16, k: u16) -> u32 {
        let key = (n << 8) | k;
        if let Some(val) = self.cache.get(&key) {
            return *val;
        }

        let val = binomial(n as usize, k as usize) as u32;
        self.cache.insert(key, val);
        val
    }

    pub fn get_index(&mut self, cards: &[Card]) -> u32 {
        let mut cards = cards.to_vec();
        cards.sort();

        let mut index = 0;

        for i in 0..cards.len() {
            let num_possible_before: u8 = cards[i].into(); // 0 to card[i] - 1
            let dim = i + 1;
            let ncr = self.get_binomial(num_possible_before as u16, dim as u16);
            index += ncr;
        }

        index
    }

    // fn combinatorial_index(cards: &[usize]) -> usize {

    //     //we want the smallest index 1st
    //     let mut cards = cards.to_vec();
    //     cards.sort();

    //     //so if I have 7 x, I want to add
    //     //how many combinations of C(6, 2) to add

    //     let mut index = 0;

    //     for i in 0..cards.len() {
    //         //trace!("Card {} is {}", i, cards[i]);
    //         let num_possible_before = cards[i]; // 0 to card[i] - 1
    //         let dim = i+1;
    //         //Example cards 50 12 5
    //         // (50 C 3) + (12 C 2) + (5 C 1)
    //         // # of ways to choose cards 0-49 in  3 cards
    //         // # of ways to choose cards 0-11 in  2 cards
    //         // # of ways to choose cards 0-4  in  1 card
    //         let ncr = binomial(num_possible_before, dim);
    //         //trace!("Adding {} choose {} == {} to index", num_possible_before, dim, ncr);
    //         index += ncr;
    //     }

    //     // https://en.wikipedia.org/wiki/Combinatorial_number_system
    //     // https://math.stackexchange.com/questions/1227409/indexing-all-combinations-without-making-list

    //     index
    // }
}

#[cfg(test)]
mod tests {

    use crate::{init_test_logger, Card};

    use super::*;

    //#[test]
    //It's a bit slow
    #[allow(dead_code)]
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
        let mut ci = CombinatorialIndex::new();
        for card1 in 0..52u8 {
            let card1_obj: Card = card1.try_into().unwrap();
            for card2 in 0..card1 {
                let card2_obj: Card = card2.try_into().unwrap();
                for card3 in 0..card2 {
                    let card3_obj: Card = card3.try_into().unwrap();
                    let idx = ci.get_index(&vec![card1_obj, card2_obj, card3_obj]);
                    // println!("{} {} {} ==> Idx is {} but should be {}",
                    //     card1, card2, card3,
                    //     idx, index_check);
                    assert_eq!(idx, index_check);
                    index_check += 1;
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        let mut ci = CombinatorialIndex::new();
        let mut index_check = 0;
        for card1 in 0..52u8 {
            let card1_obj: Card = card1.try_into().unwrap();
            for card2 in 0..card1 {
                let card2_obj: Card = card2.try_into().unwrap();
                for card3 in 0..card2 {
                    let card3_obj: Card = card3.try_into().unwrap();
                    for card4 in 0..card3 {
                        let card4_obj: Card = card4.try_into().unwrap();
                        for card5 in 0..card4 {
                            let card5_obj: Card = card5.try_into().unwrap();
                            for card6 in 0..card5 {
                                let card6_obj: Card = card6.try_into().unwrap();
                                for card7 in 0..card6 {
                                    let card7_obj: Card = card7.try_into().unwrap();

                                    // let idx = combinatorial_index(&vec![card1 as usize, card2 as usize, card3 as usize,
                                    //     card4 as usize, card5 as usize, card6 as usize, card7 as usize]);
                                    let idx = ci.get_index(&vec![
                                        card1_obj, card2_obj, card3_obj, card4_obj, card5_obj,
                                        card6_obj, card7_obj,
                                    ]);
                                    assert_eq!(idx, index_check);
                                    index_check += 1;

                                    if index_check % 1_000_000 == 0 {
                                        println!(
                                            "Index is {} cache size {}",
                                            index_check,
                                            ci.cache.len()
                                        );
                                    }

                                    assert!(index_check < 400_000_000);
                                }
                            }
                        }
                    }
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        //52 choose 5 == 2_598_960 ; 47 choose 2 == 1081 ; produce = 2_810_503_360
        //52 choose 7 -- 133_784_560
        //7 choose 5 == 21
        assert_eq!(133_784_560, index_check);
    }

    // cargo test cache_perf --lib --release -- --nocapture
}
