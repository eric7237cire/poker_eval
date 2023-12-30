
use boomphf::Mphf;
//use ph::fmph;
use std::borrow::Borrow;

use crate::Card;


use super::{
    lookup::{LOOKUP, LOOKUP_FLUSH},
    perfect_hash::get_value_bits_for_flush,
    rank::Rank,
    CARDS, GLOBAL_SUIT_SHIFT, INITIAL_SUIT_COUNT,
};

/*


*/

// fn check_perfect_hash(rank_bases: &[u64]) {
//     let weights_u32 = rank_bases.iter().map(|x| *x as u32).collect::<Vec<u32>>();

//     let unique_sets = enumerate_all_unique_sets(&weights_u32);

//     info!("Unique sets: {}", unique_sets.len());

//     let f = fmph::Function::from(unique_sets.as_ref());

//     //open a file hash1.dat in data dir
//     let path = get_perfect_hash_path();
//     let path = path.parent().unwrap().join("test.dat");
//     let mut file = File::create(&path).unwrap();

//     f.write(&mut file).unwrap();
//     file.flush().unwrap();

//     let start = std::time::Instant::now();
//     let mut file = File::open(&path).unwrap();
//     let f2 = fmph::Function::read(&mut file).unwrap();

//     let mut seen: SeenBitSet = SeenBitSet::default();

//     for us in unique_sets.iter() {
//         let hash = f2.get(us).unwrap();
//         if *seen.get(hash as usize).unwrap() {
//             panic!("Duplicate hash: {}", hash);
//         }
//         seen.set(hash as usize, true);
//     }

//     info!("Time to read and check: {:?} {}", start.elapsed(), seen.count_ones());

// }

pub fn calc_lookup_key_and_mask<I, B>(cards: I) -> (u64, u64)
where
    I: Iterator<Item = B>,
    B: Borrow<Card>,
{
    let mut lookup_key_sum = INITIAL_SUIT_COUNT << GLOBAL_SUIT_SHIFT;
    let mut card_mask = 0;
    for card in cards {
        let card_index: usize = (*card.borrow()).into();
        let (lookup_key, card_bit) = CARDS[card_index];
        lookup_key_sum += lookup_key;
        card_mask |= card_bit;
    }
    (lookup_key_sum, card_mask)
}

//pub fn fast_hand_eval<I, B>(cards: I, hash_func: &fmph::Function) -> Rank
pub fn fast_hand_eval<I, B>(cards: I, hash_func: &Mphf<u32>) -> Rank
where
    I: Iterator<Item = B>,
    B: Borrow<Card>,
{
    let (lookup_key_sum, card_mask) = calc_lookup_key_and_mask(cards);

    let flush_lookup_key = get_value_bits_for_flush(lookup_key_sum, card_mask);

    let raw_rank = if let Some(flush_lookup) = flush_lookup_key {
        LOOKUP_FLUSH[flush_lookup as usize]
    } else {
        //hash it first
        let raw_lookup_key_without_suits = lookup_key_sum as u32;
        //let lookup_key_without_suits = hash_func.get(&raw_lookup_key_without_suits as _).unwrap();
        let lookup_key_without_suits = hash_func.hash(&raw_lookup_key_without_suits as _);
        LOOKUP[lookup_key_without_suits as usize]
    };

    Rank::from(raw_rank)
}

#[cfg(test)]
mod tests {
    use log::info;

    use crate::{eval::pre_calc::{rank::RankEnum, perfect_hash::load_boomperfect_hash}, init_test_logger, Board};

    use super::*;

    #[test]
    fn test_lookups() {
        //create_perfect_hash();
        let f = load_boomperfect_hash();

        let board = Board::try_from("7d 5s 2h 3s 4c").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::HighCard(0));

        let board = Board::try_from("Ad 5s Qd Tc Kh Js").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::Straight(9));

        let board = Board::try_from("9d 5s Qd Tc Kh Js").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::Straight(8));

        let board = Board::try_from("2d 5s Qd 3h Kh 4s Ac").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::Straight(0));

        let board = Board::try_from("2c 5c Qd 3c Kh 4c Ac").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::StraightFlush(0));

        //Even though there is a flush on the board
        let board = Board::try_from("3c 3d Qd 5d 4h 4c 4d").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::FullHouse(25));

        let board = Board::try_from("3c 3d 3h Qd 5d 4c 4d").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::FullHouse(13));

        let board = Board::try_from("2c 2d 2h Qd 5d 4c 4d").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::FullHouse(1));

        let board = Board::try_from("Ad 3d Qd 5d 4h 4c 4d").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::Flush(996));

        let board = Board::try_from("Ad 9d Qd Jd 4h 4c Td").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        assert_eq!(rank.get_rank_enum(), RankEnum::Flush(1112));

        let board = Board::try_from("Ad Kd Qd Jd 4h 4c 9d").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        //the best flush without it being a str8
        assert_eq!(rank.get_rank_enum(), RankEnum::Flush(1276));

        let board = Board::try_from("2h 7h 4d Jd 4h 3h 5h").unwrap();
        let rank = fast_hand_eval(board.as_slice_card().iter(), &f);

        //the worst flush without it being a str8
        assert_eq!(rank.get_rank_enum(), RankEnum::Flush(0));
    }

    #[test]
    fn test_abc() {
        //  cargo test --lib --release abc -- --nocapture

        // 7 cards -- 49 205 unique keys
        // 6 cards -- 18 395
        // 5 cards --  6 175

        // 73,775  unique key sums for 5 to 7 cards

        init_test_logger();

        //https://github.com/b-inary/holdem-hand-evaluator/blob/main/assets/src/constants.rs
        let rank_base1: [u64; 13] = [
            0x000800, 0x002000, 0x024800, 0x025005, 0x03102e, 0x05f0e4, 0x13dc93, 0x344211,
            0x35a068, 0x377813, 0x378001, 0x378800, 0x380000,
        ];

        //check_perfect_hash(&rank_base1);

        // for i in 0..13 {
        //     //also print in binary
        //     for j in 1..=4 {
        //         info!("{} x {}: {:>40b} {}", i, j, rank_base1[i]*j, rank_base1[i]*j);
        //     }
        // }
        //assert!(false);

        //test_has_unique_value(&rank_base1);
        //https://github.com/zekyll/OMPEval/blob/master/omp/HandEvaluator.cpp

        let rank_base2: [u64; 13] = [
            0x2000, 0x8001, 0x11000, 0x3a000, 0x91000, 0x176005, 0x366000, 0x41a013, 0x47802e,
            0x479068, 0x48c0e4, 0x48f211, 0x494493,
        ];

        //check_perfect_hash(&rank_base2, );

        //test_has_unique_value(&rank_base2);

        //0x2000, 0x8001, 0x11000, 0x3a000, 0x91000, 0x176005, 0x366000, 0x41a013, 0x47802e, 0x479068, 0x48c0e4, 0x48f211, 0x494493
        //];

        let mut rank_base3: Vec<u64> = vec![1, 5, 25];

        while rank_base3.len() < 13 {
            let mut next = rank_base3[rank_base3.len() - 1] * 5;
            rank_base3.push(next);
        }

        //check_perfect_hash(&rank_base3, );

        let suit_bases: [u64; 6] = [0x1000, 0x0100, 0x0010, 0x0001, 0x8888, 0x3333];
        for s in 0..suit_bases.len() {
            //Print in binary (width 40, hex, and dec)
            info!(
                "{} : {:>40b} {:>10x} {}",
                s, suit_bases[s], suit_bases[s], suit_bases[s]
            );
        }

        // let valid_counts = test_has_unique_value(&rank_base3);

        // //Now counts given 0, 1, 2, 3, 4 aces
        // info!("Valid counts: {}", valid_counts.len());
        // let mut tally_ace_counts = [0; 5];

        // //now for king to 9, we want to tally them only when all the higher values are 0
        // let mut tally_king_counts = [0; 5];

        // let mut tally_x_counts = [0; 5];
        // //11 king, 10 queen, 9 jack, 8 ten, 7 nine, 6 eight, 5 seven, 4 six, 3 five, 2 four, 1 three, 0 two
        // let x = 1;

        // for count in valid_counts.iter() {
        //     let mut counts = [0; 13];
        //     for i in 0..13 {
        //         counts[i] = (*count >> (i * 3)) & 0x7;
        //     }
        //     //info!("Counts: {:?}", counts);
        //     assert!(counts[12] < 5);
        //     tally_ace_counts[counts[12] as usize] += 1;

        //     if counts[12] == 0 {
        //         tally_king_counts[counts[11] as usize] += 1;
        //     }

        //     let mut ok = true;
        //     for check in x+1..=12 {
        //         if counts[check] != 0 {
        //             ok = false;
        //             break;
        //         }
        //     }
        //     if ok {
        //         tally_x_counts[counts[x] as usize] += 1;
        //     }
        // }

        // info!("Tally ace counts: {:?}", tally_ace_counts);
        // info!("Tally king counts: {:?}", tally_king_counts);
        // info!("Tally x {} counts: {:?}", x, tally_x_counts);

        // let my_bases = [
        //     3, //3
        //     13, //4
        //     55, //5
        //     200, //6
        //     561, //7
        //     1338, //8
        //     2850, //9
        //     5580, //10
        //     10230, //jack
        //     17787, //queen
        //     29601, //king
        //     //Calculated counts having 0, 1, 2, 3, or 4 aces, largest size was this
        //     //meaning adding it should be enough to garauntee a unique key
        //     47476 // Ace
        // ];

        assert_eq!(1, 3);
    }
}

/*
Unused code while figuring it out


fn test_has_unique_value2(weights: &[u64]) {
    //Go through values 1 (2) to 13 (Ace) and check that the weight give a unique value
    //Reason is that when a hand is not a flush, the card values alone determine the evaluation rank
    let mut seen: HashSet<u64> = HashSet::new();
    //let mut seen_map: HashMap<u64, [u16;13]> = HashMap::new();

    let mut seen_count: HashSet<u64> = HashSet::new();

    let mut num_unique = 0;
    let mut max_key = 0;
    for v1 in 1..14 {
        for v2 in 1..14 {
            for v3 in 1..14 {
                for v4 in 1..14 {
                    for v5 in 1..14 {
                        for v6 in 1..14 {
                            for v7 in 1..14 {
                                let mut counts = [0; 13];
                                counts[v1 - 1] += 1;
                                counts[v2 - 1] += 1;
                                counts[v3 - 1] += 1;
                                counts[v4 - 1] += 1;
                                counts[v5 - 1] += 1;
                                counts[v6 - 1] += 1;
                                counts[v7 - 1] += 1;

                                //max count is 4
                                if *counts.iter().max().unwrap() > 4 {
                                    continue;
                                }

                                //counts hash is 3 bits per count
                                let mut counts_hash = 0;
                                for i in 0..13 {
                                    counts_hash |= counts[i] << (i * 3);
                                }
                                if seen_count.contains(&counts_hash) {
                                    //This is ok
                                    continue;
                                }
                                seen_count.insert(counts_hash);

                                let mut key = 0;
                                for i in 0..13 {
                                    key += weights[i] * counts[i];
                                }
                                if seen.contains(&key) {
                                    info!("Duplicate key: {} for counts {:?}", key, counts);
                                }
                                seen.insert(key);
                                max_key = max(max_key, key);

                                num_unique += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Max key: {}", max_key);
    info!("Number of unique keys: {} for {}", seen.len(), num_unique);
    info!("Bits needed for max key: {}", 64 - max_key.leading_zeros());
}

fn test_has_unique_value3(weights: &[u64]) {
    //Go through values 1 (2) to 13 (Ace) and check that the weight give a unique value
    //Reason is that when a hand is not a flush, the card values alone determine the evaluation rank
    let mut seen: HashSet<u64> = HashSet::new();
    //let mut seen_map: HashMap<u64, [u16;13]> = HashMap::new();

    let mut seen_count: HashSet<u64> = HashSet::new();

    let mut num_unique = 0;
    let mut max_key = 0;
    for card1 in 0..52 {
        for card2 in (card1 + 1)..52 {
            for card3 in (card2 + 1)..52 {
                for card4 in (card3 + 1)..52 {
                    for card5 in (card4 + 1)..52 {
                        for card6 in (card5 + 1)..52 {
                            for card7 in (card6 + 1)..52 {
                                //card values == card1 >> 2
                                let v1 = card1 >> 2;
                                let v2 = card2 >> 2;
                                let v3 = card3 >> 2;
                                let v4 = card4 >> 2;
                                let v5 = card5 >> 2;
                                let v6 = card6 >> 2;
                                let v7 = card7 >> 2;

                                let mut counts = [0; 13];
                                counts[v1] += 1;
                                counts[v2] += 1;
                                counts[v3] += 1;
                                counts[v4] += 1;
                                counts[v5] += 1;
                                counts[v6] += 1;
                                counts[v7] += 1;

                                //max count is 4
                                if *counts.iter().max().unwrap() > 4 {
                                    panic!("Max count is 4");
                                }

                                //counts hash is 3 bits per count
                                let mut counts_hash = 0;
                                for i in 0..13 {
                                    counts_hash |= counts[i] << (i * 3);
                                }
                                if seen_count.contains(&counts_hash) {
                                    //This is ok
                                    continue;
                                }
                                seen_count.insert(counts_hash);

                                let mut key = 0;
                                for i in 0..13 {
                                    key += weights[i] * counts[i];
                                }
                                if seen.contains(&key) {
                                    info!("Duplicate key: {} for counts {:?}", key, counts);
                                }
                                seen.insert(key);
                                max_key = max(max_key, key);

                                num_unique += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Max key: {}", max_key);
    info!("Number of unique keys: {} for {}", seen.len(), num_unique);
    info!("Bits needed for max key: {}", 64 - max_key.leading_zeros());
}


// fn test_has_unique_value(weights: &[u64]) -> HashSet<u64> {
//     //Go through values 1 (2) to 13 (Ace) and check that the weight give a unique value
//     //Reason is that when a hand is not a flush, the card values alone determine the evaluation rank

//     //The ugliest but fastest of the checkers

//     let mut seen: HashSet<u64> = HashSet::new();

//     //using 3 bits per count
//     let mut valid_counts: HashSet<u64> = HashSet::new();

//     let mut num_unique = 0;
//     let mut max_key = 0;
//     for num_val0 in 0..=4 {

//         for num_val1 in 0..=4 {
//             let val_sum = num_val0 + num_val1;
//             if val_sum > 7 {
//                 break;
//             }
//             for num_val2 in 0..=4 {
//                 let val_sum = num_val0 + num_val1 + num_val2;
//                 if val_sum > 7 {
//                     break;
//                 }
//                 for num_val3 in 0..=4 {
//                     let val_sum = num_val0 + num_val1 + num_val2 + num_val3;
//                     if val_sum > 7 {
//                         break;
//                     }
//                     for num_val4 in 0..=4 {
//                         let val_sum = num_val0 + num_val1 + num_val2 + num_val3 + num_val4;
//                         if val_sum > 7 {
//                             break;
//                         }
//                         for num_val5 in 0..=4 {
//                             let val_sum =
//                                 num_val0 + num_val1 + num_val2 + num_val3 + num_val4 + num_val5;
//                             if val_sum > 7 {
//                                 break;
//                             }
//                             for num_val6 in 0..=4 {
//                                 let val_sum = num_val0
//                                     + num_val1
//                                     + num_val2
//                                     + num_val3
//                                     + num_val4
//                                     + num_val5
//                                     + num_val6;
//                                 let val_0to6 = val_sum;
//                                 if val_sum > 7 {
//                                     break;
//                                 }
//                                 for num_val7 in 0..=4 {
//                                     let val_sum = val_0to6 + num_val7;
//                                     if val_sum > 7 {
//                                         break;
//                                     }
//                                     for num_val8 in 0..=4 {
//                                         let val_sum = val_0to6 + num_val7 + num_val8;
//                                         if val_sum > 7 {
//                                             break;
//                                         }
//                                         for num_val9 in 0..=4 {
//                                             let val_sum = val_0to6 + num_val7 + num_val8 + num_val9;
//                                             if val_sum > 7 {
//                                                 break;
//                                             }
//                                             for num_val10 in 0..=4 {
//                                                 let val_sum = val_0to6
//                                                     + num_val7
//                                                     + num_val8
//                                                     + num_val9
//                                                     + num_val10;
//                                                 if val_sum > 7 {
//                                                     break;
//                                                 }
//                                                 for num_val11 in 0..=4 {
//                                                     let val_sum = val_0to6
//                                                         + num_val7
//                                                         + num_val8
//                                                         + num_val9
//                                                         + num_val10
//                                                         + num_val11;
//                                                     if val_sum > 7 {
//                                                         break;
//                                                     }
//                                                     for num_val12 in 0..=4 {
//                                                         let val_sum = val_0to6
//                                                             + num_val7
//                                                             + num_val8
//                                                             + num_val9
//                                                             + num_val10
//                                                             + num_val11
//                                                             + num_val12;
//                                                         if val_sum < 5 || val_sum > 7 {
//                                                             continue;
//                                                         }
//                                                         let mut counts = [0; 13];
//                                                         counts[0] = num_val0;
//                                                         counts[1] = num_val1;
//                                                         counts[2] = num_val2;
//                                                         counts[3] = num_val3;
//                                                         counts[4] = num_val4;
//                                                         counts[5] = num_val5;
//                                                         counts[6] = num_val6;
//                                                         counts[7] = num_val7;
//                                                         counts[8] = num_val8;
//                                                         counts[9] = num_val9;
//                                                         counts[10] = num_val10;
//                                                         counts[11] = num_val11;
//                                                         counts[12] = num_val12;

//                                                         //max count is 4
//                                                         if *counts.iter().max().unwrap() > 4 {
//                                                             panic!("Max count is 4");
//                                                         }

//                                                         let mut counts_hash = 0;
//                                                         for i in 0..13 {
//                                                             counts_hash |= counts[i] << (i * 3);
//                                                         }
//                                                         if valid_counts.contains(&counts_hash) {
//                                                             //This is ok
//                                                             continue;
//                                                         }
//                                                         valid_counts.insert(counts_hash);

//                                                         let mut key = 0;
//                                                         for i in 0..13 {
//                                                             key += weights[i] * counts[i];
//                                                         }
//                                                         if seen.contains(&key) {
//                                                             panic!(
//                                                                 "Duplicate key: {} for counts {:?}",
//                                                                 key, counts
//                                                             );
//                                                         }
//                                                         seen.insert(key);
//                                                         max_key = max(max_key, key);

//                                                         num_unique += 1;
//                                                     }
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     info!("Max key: {}", max_key);
//     info!("Number of unique keys: {} for {}", seen.len(), num_unique);
//     info!("Bits needed for max key: {}", 64 - max_key.leading_zeros());

//     valid_counts
// }

// const NUMBER_OF_RANKS : usize = 13;

//

*/
