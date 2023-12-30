// generate lookup tables.

use std::fs::File;
use std::io::Write;
use std::{cmp::max, collections::HashMap};

use crate::eval::pre_calc::perfect_hash::load_boomperfect_hash;
use crate::{Card, CardValue};
use boomphf::Mphf;
use log::info;
//use ph::fmph;

use crate::eval::pre_calc::fast_eval::calc_lookup_key_and_mask;
use crate::eval::pre_calc::RANK_BASES;
use crate::eval::{
    kev::{eval_5cards, eval_6cards, eval_7cards},
    pre_calc::{
        constants::{GLOBAL_SUIT_SHIFT, INITIAL_SUIT_COUNT, NUMBER_OF_CARDS},
        get_lookup_path,
        perfect_hash::{create_perfect_hash, load_perfect_hash},
        NUMBER_OF_RANKS,
    },
};
use crate::Suit;

use super::{
    constants::{CARDS, FLUSH_MASK, RANK_FAMILY_OFFEST},
    perfect_hash::get_value_bits_for_flush,
};

fn adjust_hand_rank(rank: u16) -> u16 {
    let reversed_rank = 7463 - rank; // now best hand = 7462
    match reversed_rank {
        1..=1277 => reversed_rank - 1, // 1277 high card
        1278..=4137 => (1 << RANK_FAMILY_OFFEST) + reversed_rank - 1278, // 2860 one pair
        4138..=4995 => (2 << RANK_FAMILY_OFFEST) + reversed_rank - 4138, //  858 two pair
        4996..=5853 => (3 << RANK_FAMILY_OFFEST) + reversed_rank - 4996, //  858 three-kind
        5854..=5863 => (4 << RANK_FAMILY_OFFEST) + reversed_rank - 5854, //   10 straights
        5864..=7140 => (5 << RANK_FAMILY_OFFEST) + reversed_rank - 5864, // 1277 flushes
        7141..=7296 => (6 << RANK_FAMILY_OFFEST) + reversed_rank - 7141, //  156 full house
        7297..=7452 => (7 << RANK_FAMILY_OFFEST) + reversed_rank - 7297, //  156 four-kind
        7453..=7462 => (8 << RANK_FAMILY_OFFEST) + reversed_rank - 7453, //   10 straight flushes
        _ => panic!("Invalid rank value {}, {}", rank, reversed_rank),
    }
}

#[inline]
fn add_card(key: u64, mask: u64, card: usize) -> (u64, u64) {
    let (k, m) = CARDS[card];
    (key + k, mask | m)
}

#[inline]
fn update(
    key: u64,
    mask: u64,
    //the kev evaluation result
    val: u16,
    lookup: &mut HashMap<u64, u16>,
    lookup_flush: &mut HashMap<usize, u16>,
    //mixed_key_perfect_hash_func: &fmph::Function,
    mixed_key_perfect_hash_func: &Mphf<u32>
) {
    let flush_key = get_value_bits_for_flush(key, mask);
    if let Some(flush_key) = flush_key {
        //let flush_key = (mask >> (4 * is_flush.leading_zeros())) as u16;
        match lookup_flush.insert(flush_key as usize, val) {
            Some(v) => assert_eq!(val, v),
            None => (),
        };
    } else {
        //we truncate the suited count info in the higher bits
        let mixed_key = key as u32;
        //let hash_key = mixed_key_perfect_hash_func.get(&mixed_key).unwrap();
        let hash_key = mixed_key_perfect_hash_func.hash(&mixed_key);
        assert!(hash_key < 73_775);
        match lookup.insert(hash_key, val) {
            //We should get same evaluation if we hash to the same value
            Some(v) => assert_eq!(val, v),
            None => (),
        }
    }
}

/*
Loops through all hands, but this generates lots of redundancy and takes a long time.

We only need to enumerate the distinct hand ranks
*/
pub fn generate_lookup_tables() {
    let mut lookup = HashMap::new();
    let mut lookup_flush = HashMap::new();

    info!("Loading perfect hash func");
    //create_perfect_hash();
    //let hash_func = load_perfect_hash();
    let hash_func = load_boomperfect_hash();

    info!("Running through all 5 card hands");
    // 5-cards
    for i in 0..(NUMBER_OF_CARDS - 4) {
        let (key, mask) = add_card(INITIAL_SUIT_COUNT << GLOBAL_SUIT_SHIFT, 0, i);
        for j in (i + 1)..(NUMBER_OF_CARDS - 3) {
            let (key, mask) = add_card(key, mask, j);
            for k in (j + 1)..(NUMBER_OF_CARDS - 2) {
                let (key, mask) = add_card(key, mask, k);
                for m in (k + 1)..(NUMBER_OF_CARDS - 1) {
                    let (key, mask) = add_card(key, mask, m);
                    for n in (m + 1)..NUMBER_OF_CARDS {
                        let (key, mask) = add_card(key, mask, n);
                        update(
                            key,
                            mask,
                            eval_5cards(i, j, k, m, n),
                            &mut lookup,
                            &mut lookup_flush,
                            &hash_func,
                        );
                    }
                }
            }
        }
    }

    info!("Running through all 6 card hands");
    // 6-cards
    for i in 0..(NUMBER_OF_CARDS - 5) {
        let (key, mask) = add_card(INITIAL_SUIT_COUNT << GLOBAL_SUIT_SHIFT, 0, i);
        for j in (i + 1)..(NUMBER_OF_CARDS - 4) {
            let (key, mask) = add_card(key, mask, j);
            for k in (j + 1)..(NUMBER_OF_CARDS - 3) {
                let (key, mask) = add_card(key, mask, k);
                for m in (k + 1)..(NUMBER_OF_CARDS - 2) {
                    let (key, mask) = add_card(key, mask, m);
                    for n in (m + 1)..(NUMBER_OF_CARDS - 1) {
                        let (key, mask) = add_card(key, mask, n);
                        for p in (n + 1)..NUMBER_OF_CARDS {
                            let (key, mask) = add_card(key, mask, p);
                            update(
                                key,
                                mask,
                                eval_6cards(i, j, k, m, n, p),
                                &mut lookup,
                                &mut lookup_flush,
                                &hash_func,
                            );
                        }
                    }
                }
            }
        }
    }

    // 7-cards
    info!("Running through all 7 card hands");
    for i in 0..(NUMBER_OF_CARDS - 6) {
        info!("7 card hands, i is {}", i);
        let (key, mask) = add_card(INITIAL_SUIT_COUNT << GLOBAL_SUIT_SHIFT, 0, i);
        for j in (i + 1)..(NUMBER_OF_CARDS - 5) {
            let (key, mask) = add_card(key, mask, j);
            for k in (j + 1)..(NUMBER_OF_CARDS - 4) {
                let (key, mask) = add_card(key, mask, k);
                for m in (k + 1)..(NUMBER_OF_CARDS - 3) {
                    let (key, mask) = add_card(key, mask, m);
                    for n in (m + 1)..(NUMBER_OF_CARDS - 2) {
                        let (key, mask) = add_card(key, mask, n);
                        for p in (n + 1)..(NUMBER_OF_CARDS - 1) {
                            let (key, mask) = add_card(key, mask, p);
                            for q in (p + 1)..NUMBER_OF_CARDS {
                                let (key, mask) = add_card(key, mask, q);
                                update(
                                    key,
                                    mask,
                                    eval_7cards(i, j, k, m, n, p, q),
                                    &mut lookup,
                                    &mut lookup_flush,
                                    &hash_func,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Sorting vecs");
    let mut lookup_vec = vec![0; *lookup.keys().max().unwrap() as usize + 1];
    let mut lookup_flush_vec = vec![0; *lookup_flush.keys().max().unwrap() + 1];

    for (key, value) in &lookup {
        lookup_vec[*key as usize] = adjust_hand_rank(*value);
    }

    for (key, value) in &lookup_flush {
        lookup_flush_vec[*key] = adjust_hand_rank(*value);
    }

    let lookup_path = get_lookup_path();
    let mut file = File::create(&lookup_path).unwrap();
    writeln!(
        file,
        "pub const LOOKUP: [u16; {}] = {:?};",
        lookup_vec.len(),
        lookup_vec
    )
    .unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "pub const LOOKUP_FLUSH: [u16; {}] = {:?};",
        lookup_flush_vec.len(),
        lookup_flush_vec
    )
    .unwrap();

    println!("wrote result to 'assets/src/lookup.rs'");
}

/*
Loops through all hands, but this generates lots of redundancy and takes a long time.

We only need to enumerate the distinct hand ranks
*/
pub fn generate_lookup_tables_fast() {
    let mut lookup = HashMap::new();
    let mut lookup_flush = HashMap::new();

    info!("Loading perfect hash func");
    //create_perfect_hash();
    //let hash_func = load_perfect_hash();
    let hash_func = load_boomperfect_hash();

    //Code from enumerate_all_unique_sets
    //we have card1 <= card2 <= card3 <= card4 <= card5 <= card6 <= card7
    //and card1 < card5 (to enforce only 4 of same card value)
    //and card2 < card6 (to enforce only 4 of same card value)
    //and card3 < card7 (to enforce only 4 of same card value)

    //We choose concrete cards that will never have a flush, we'll do the flush bits later
    for i in 0..(NUMBER_OF_RANKS - 1) {
        let cv_1: CardValue = (i as u8).try_into().unwrap();
        let card1 = Card::new(cv_1, Suit::Club);
        for j in i..NUMBER_OF_RANKS {
            let cv_2: CardValue = (j as u8).try_into().unwrap();
            let card2 = Card::new(cv_2, Suit::Spade);
            for k in j..NUMBER_OF_RANKS {
                let cv_3: CardValue = (k as u8).try_into().unwrap();
                let card3 = Card::new(cv_3, Suit::Heart);
                for m in k..NUMBER_OF_RANKS {
                    let cv_4: CardValue = (m as u8).try_into().unwrap();
                    let card4 = Card::new(cv_4, Suit::Diamond);
                    for n in max(m, i + 1)..NUMBER_OF_RANKS {
                        let cv_5: CardValue = (n as u8).try_into().unwrap();
                        let card5 = Card::new(cv_5, Suit::Club);
                        let x = RANK_BASES[i] + RANK_BASES[j] + RANK_BASES[k];
                        let x = x + RANK_BASES[m] + RANK_BASES[n];

                        //5 card hand sum
                        //x is the same as CARD[x], which we check here
                        let (sum_check, _) =
                            calc_lookup_key_and_mask([card1, card2, card3, card4, card5].iter());
                        //The suit sums are in the higher bits, so we need to mask them out
                        assert_eq!(x, sum_check & ((1 << 32) - 1));

                        let val = eval_5cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into());

                        assert!(val > 0);
                        assert!(val < 7463);

                        update(x, 0, val, &mut lookup, &mut lookup_flush, &hash_func);

                        for p in max(n, j + 1)..NUMBER_OF_RANKS {
                            let cv_6 = (p as u8).try_into().unwrap();
                            let card6 = Card::new(cv_6, Suit::Spade);
                            let x = x + RANK_BASES[p];

                            let (sum_check, _) = calc_lookup_key_and_mask(
                                [card1, card2, card3, card4, card5, card6].iter(),
                            );
                            //The suit sums are in the higher bits, so we need to mask them out
                            assert_eq!(x, sum_check & ((1 << 32) - 1));

                            let val = eval_6cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into(), card6.into());

                            assert!(val > 0);
                            assert!(val < 7463);

                            update(x, 0, val, &mut lookup, &mut lookup_flush, &hash_func);

                            for q in max(p, k + 1)..NUMBER_OF_RANKS {
                                let cv_7 = (q as u8).try_into().unwrap();
                                let card7 = Card::new(cv_7, Suit::Heart);
                                let x = x + RANK_BASES[q];

                                let (sum_check, mask_check) = calc_lookup_key_and_mask(
                                    [card1, card2, card3, card4, card5, card6, card7].iter(),
                                );
                                //The suit sums are in the higher bits, so we need to mask them out
                                assert_eq!(x, sum_check & ((1 << 32) - 1));
                                assert_eq!(7, mask_check.count_ones());

                                let val = eval_7cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into(), card6.into(), card7.into());

                                // if val==0 {
                                //     info!("Card 1 {} card 2 {} card 3 {} card 4 {} card 5 {} card 6 {} card 7 {}", card1, card2, card3, card4, card5, card6, card7);
                                //     info!("Card 1 [{}] card 2 [{}] card 3 [{}] card 4 [{}] card 5 [{}] card 6 [{}] card 7 [{}]", 
                                //     i, j, k, m, n, p, q
                                // );
                                // }
                                assert!(val > 0);
                                assert!(val < 7463);

                                update(x, 0, val, &mut lookup, &mut lookup_flush, &hash_func);
                            }
                        }
                    }
                }
            }
        }
    }

    //now we need to enumerate all 5 to 7 cards where there is a flush
    //when we evaluate a flush, only the bits of the flush set are available
    //this works because a full house XXXYY AB is impossible to have a flush

    //The main difference with the above code is we choose the same suit; and we don't need the value sum bits, only a bit mask
    //and the sequence needs to be strictly increasing so
    // card1 < card2 < card3 < card4 < card5 < card6 < card7
    for i in 0..(NUMBER_OF_RANKS - 1) {
        let cv_1: CardValue = (i as u8).try_into().unwrap();
        let card1 = Card::new(cv_1, Suit::Heart);
        for j in i + 1..NUMBER_OF_RANKS {
            let cv_2: CardValue = (j as u8).try_into().unwrap();
            let card2 = Card::new(cv_2, Suit::Heart);
            for k in j + 1..NUMBER_OF_RANKS {
                let cv_3: CardValue = (k as u8).try_into().unwrap();
                let card3 = Card::new(cv_3, Suit::Heart);
                for m in k + 1..NUMBER_OF_RANKS {
                    let cv_4: CardValue = (m as u8).try_into().unwrap();
                    let card4 = Card::new(cv_4, Suit::Heart);
                    for n in m + 1..NUMBER_OF_RANKS {
                        let cv_5: CardValue = (n as u8).try_into().unwrap();
                        let card5 = Card::new(cv_5, Suit::Heart);

                        //5 card hand sum
                        //x is the same as CARD[x], which we check here
                        {
                            let (card_sum, card_mask) = calc_lookup_key_and_mask(
                                [card1, card2, card3, card4, card5].iter(),
                            );
                            //The suit sums are in the higher bits, so we need to mask them out
                            assert_eq!(card_mask.count_ones(), 5);

                            assert_eq!(
                                5,
                                get_value_bits_for_flush(card_sum, card_mask)
                                    .unwrap()
                                    .count_ones()
                            );

                            let val = eval_5cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into());

                            assert!(val > 0);
                            assert!(val < 7463);

                            update(
                                card_sum,
                                card_mask,
                                val,
                                &mut lookup,
                                &mut lookup_flush,
                                &hash_func,
                            );
                        }
                        for p in n + 1..NUMBER_OF_RANKS {
                            let cv_6 = (p as u8).try_into().unwrap();
                            let card6 = Card::new(cv_6, Suit::Heart);

                            {
                                let (card_sum, card_mask) = calc_lookup_key_and_mask(
                                    [card1, card2, card3, card4, card5, card6].iter(),
                                );

                                assert_eq!(
                                    6,
                                    get_value_bits_for_flush(card_sum, card_mask)
                                        .unwrap()
                                        .count_ones()
                                );

                                let val = eval_6cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into(), card6.into());

                                assert!(val > 0);
                                assert!(val < 7463);

                                update(
                                    card_sum,
                                    card_mask,
                                    val,
                                    &mut lookup,
                                    &mut lookup_flush,
                                    &hash_func,
                                );
                            }
                            for q in p + 1..NUMBER_OF_RANKS {
                                let cv_7 = (q as u8).try_into().unwrap();
                                let card7 = Card::new(cv_7, Suit::Heart);

                                {
                                    let (card_sum, card_mask) = calc_lookup_key_and_mask(
                                        [card1, card2, card3, card4, card5, card6, card7].iter(),
                                    );

                                    assert_eq!(
                                        7,
                                        get_value_bits_for_flush(card_sum, card_mask)
                                            .unwrap()
                                            .count_ones()
                                    );

                                    let val = eval_7cards(card1.into(), card2.into(), card3.into(), card4.into(), card5.into(), card6.into(), card7.into());

                                    assert!(val > 0);
                                    assert!(val < 7463);

                                    update(
                                        card_sum,
                                        card_mask,
                                        val,
                                        &mut lookup,
                                        &mut lookup_flush,
                                        &hash_func,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Sorting vecs");
    let mut lookup_vec = vec![0; *lookup.keys().max().unwrap() as usize + 1];
    let mut lookup_flush_vec = vec![0; *lookup_flush.keys().max().unwrap() + 1];

    for (key, value) in &lookup {
        lookup_vec[*key as usize] = adjust_hand_rank(*value);
    }

    for (key, value) in &lookup_flush {
        lookup_flush_vec[*key] = adjust_hand_rank(*value);
    }

    let lookup_path = get_lookup_path();
    let mut file = File::create(&lookup_path).unwrap();
    writeln!(
        file,
        "pub const LOOKUP: [u16; {}] = {:?};",
        lookup_vec.len(),
        lookup_vec
    )
    .unwrap();
    writeln!(file).unwrap();
    writeln!(
        file,
        "pub const LOOKUP_FLUSH: [u16; {}] = {:?};",
        lookup_flush_vec.len(),
        lookup_flush_vec
    )
    .unwrap();

    println!("wrote result to 'assets/src/lookup.rs'");
}

#[cfg(test)]
mod tests {
    use crate::init_test_logger;

    use crate::eval::pre_calc::perfect_hash::create_perfect_hash;

    use super::*;

    #[test]
    fn test_adjust_hand_rank() {
        assert_eq!(adjust_hand_rank(0), 7462);
        assert_eq!(adjust_hand_rank(1), 7461);
        assert_eq!(adjust_hand_rank(1277), 6185);
        assert_eq!(adjust_hand_rank(1278), 6184);
        assert_eq!(adjust_hand_rank(4137), 3326);
        assert_eq!(adjust_hand_rank(4138), 3325);
        assert_eq!(adjust_hand_rank(4995), 2468);
        assert_eq!(adjust_hand_rank(4996), 2467);
        assert_eq!(adjust_hand_rank(5853), 1600);
        assert_eq!(adjust_hand_rank(5854), 1599);
        assert_eq!(adjust_hand_rank(5863), 1590);
        assert_eq!(adjust_hand_rank(5864), 1589);
        assert_eq!(adjust_hand_rank(7140), 312);
        assert_eq!(adjust_hand_rank(7141), 311);
        assert_eq!(adjust_hand_rank(7296), 156);
        assert_eq!(adjust_hand_rank(7297), 155);
        assert_eq!(adjust_hand_rank(7452), 9);
        assert_eq!(adjust_hand_rank(7453), 8);
        assert_eq!(adjust_hand_rank(7462), 0);
    }

    #[test]
    #[allow(dead_code)]
    fn test_generate_lookup_tables() {
        init_test_logger();

        //create_perfect_hash();

        //cargo test --lib --release test_generate_lookup_tables -- --nocapture
        //generate_lookup_tables();

        generate_lookup_tables_fast();
    }
}
