use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use log::info;

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

fn test_has_unique_value(weights: &[u64]) {
    //Go through values 1 (2) to 13 (Ace) and check that the weight give a unique value
    //Reason is that when a hand is not a flush, the card values alone determine the evaluation rank

    //The ugliest but fastest of the checkers

    let mut seen: HashSet<u64> = HashSet::new();

    let mut num_unique = 0;
    let mut max_key = 0;
    for num_val0 in 0..=4 {
        
        for num_val1 in 0..=4 {
            let val_sum = num_val0 + num_val1;
            if val_sum > 7 {
                break;
            }
            for num_val2 in 0..=4 {
                let val_sum = num_val0 + num_val1 + num_val2;
                if val_sum > 7 {
                    break;
                }
                for num_val3 in 0..=4 {
                    let val_sum = num_val0 + num_val1 + num_val2 + num_val3;
                    if val_sum > 7 {
                        break;
                    }
                    for num_val4 in 0..=4 {
                        let val_sum = num_val0 + num_val1 + num_val2 + num_val3 + num_val4;
                        if val_sum > 7 {
                            break;
                        }
                        for num_val5 in 0..=4 {
                            let val_sum =
                                num_val0 + num_val1 + num_val2 + num_val3 + num_val4 + num_val5;
                            if val_sum > 7 {
                                break;
                            }
                            for num_val6 in 0..=4 {
                                let val_sum = num_val0
                                    + num_val1
                                    + num_val2
                                    + num_val3
                                    + num_val4
                                    + num_val5
                                    + num_val6;
                                let val_0to6 = val_sum;
                                if val_sum > 7 {
                                    break;
                                }
                                for num_val7 in 0..=4 {
                                    let val_sum = val_0to6 + num_val7;
                                    if val_sum > 7 {
                                        break;
                                    }
                                    for num_val8 in 0..=4 {
                                        let val_sum = val_0to6 + num_val7 + num_val8;
                                        if val_sum > 7 {
                                            break;
                                        }
                                        for num_val9 in 0..=4 {
                                            let val_sum = val_0to6 + num_val7 + num_val8 + num_val9;
                                            if val_sum > 7 {
                                                break;
                                            }
                                            for num_val10 in 0..=4 {
                                                let val_sum = val_0to6
                                                    + num_val7
                                                    + num_val8
                                                    + num_val9
                                                    + num_val10;
                                                if val_sum > 7 {
                                                    break;
                                                }
                                                for num_val11 in 0..=4 {
                                                    let val_sum = val_0to6
                                                        + num_val7
                                                        + num_val8
                                                        + num_val9
                                                        + num_val10
                                                        + num_val11;
                                                    if val_sum > 7 {
                                                        break;
                                                    }
                                                    for num_val12 in 0..=4 {
                                                        let val_sum = val_0to6
                                                            + num_val7
                                                            + num_val8
                                                            + num_val9
                                                            + num_val10
                                                            + num_val11
                                                            + num_val12;
                                                        if val_sum < 5 || val_sum > 7 {
                                                            continue;
                                                        }
                                                        let mut counts = [0; 13];
                                                        counts[0] = num_val0;
                                                        counts[1] = num_val1;
                                                        counts[2] = num_val2;
                                                        counts[3] = num_val3;
                                                        counts[4] = num_val4;
                                                        counts[5] = num_val5;
                                                        counts[6] = num_val6;
                                                        counts[7] = num_val7;
                                                        counts[8] = num_val8;
                                                        counts[9] = num_val9;
                                                        counts[10] = num_val10;
                                                        counts[11] = num_val11;
                                                        counts[12] = num_val12;

                                                        //max count is 4
                                                        if *counts.iter().max().unwrap() > 4 {
                                                            panic!("Max count is 4");
                                                        }

                                                        let mut key = 0;
                                                        for i in 0..13 {
                                                            key += weights[i] * counts[i];
                                                        }
                                                        if seen.contains(&key) {
                                                            panic!(
                                                                "Duplicate key: {} for counts {:?}",
                                                                key, counts
                                                            );
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

#[cfg(test)]
mod tests {
    use crate::init_test_logger;

    use super::*;

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

        test_has_unique_value(&rank_base1);
        //https://github.com/zekyll/OMPEval/blob/master/omp/HandEvaluator.cpp

        let rank_base2: [u64; 13] = [
            0x2000, 0x8001, 0x11000, 0x3a000, 0x91000, 0x176005, 0x366000, 0x41a013, 0x47802e,
            0x479068, 0x48c0e4, 0x48f211, 0x494493,
        ];

        test_has_unique_value(&rank_base2);

        //0x2000, 0x8001, 0x11000, 0x3a000, 0x91000, 0x176005, 0x366000, 0x41a013, 0x47802e, 0x479068, 0x48c0e4, 0x48f211, 0x494493
        //];

        let mut rank_base3: Vec<u64> = vec![1, 5, 25];

        while rank_base3.len() < 13 {
            let mut next = rank_base3[rank_base3.len() - 1] * 5;
            rank_base3.push(next);
        }

        test_has_unique_value(&rank_base3);

        assert!(false);
    }
}
