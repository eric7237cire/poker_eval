// generate lookup tables.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use log::info;
use ph::fmph;

use crate::eval::{pre_calc::{constants::{NUMBER_OF_CARDS, GLOBAL_SUIT_SHIFT, INITIAL_SUIT_COUNT}, get_lookup_path, perfect_hash::load_perfect_hash}, kev::{eval_5cards, eval_7cards, eval_6cards}};

use super::{constants::{CARDS, FLUSH_MASK, RANK_FAMILY_OFFEST}, perfect_hash::get_value_bits_for_flush};

fn adjust_hand_rank(rank: u16) -> u16 {
    let reversed_rank = 7463 - rank; // now best hand = 7462
    match reversed_rank {
        1..=1277 => reversed_rank - 1,                   // 1277 high card
        1278..=4137 => (1 << RANK_FAMILY_OFFEST) + reversed_rank - 1278, // 2860 one pair
        4138..=4995 => (2 << RANK_FAMILY_OFFEST) + reversed_rank - 4138, //  858 two pair
        4996..=5853 => (3 << RANK_FAMILY_OFFEST) + reversed_rank - 4996, //  858 three-kind
        5854..=5863 => (4 << RANK_FAMILY_OFFEST) + reversed_rank - 5854, //   10 straights
        5864..=7140 => (5 << RANK_FAMILY_OFFEST) + reversed_rank - 5864, // 1277 flushes
        7141..=7296 => (6 << RANK_FAMILY_OFFEST) + reversed_rank - 7141, //  156 full house
        7297..=7452 => (7 << RANK_FAMILY_OFFEST) + reversed_rank - 7297, //  156 four-kind
        7453..=7462 => (8 << RANK_FAMILY_OFFEST) + reversed_rank - 7453, //   10 straight flushes
        _ => panic!(),
    }
}

#[inline]
fn add_card(key: u64, mask: u64, card: usize) -> (u64, u64) {
    let (k, m) = CARDS[card];
    (key.wrapping_add(k), mask.wrapping_add(m))
}

#[inline]
fn update(
    key: u64,
    mask: u64,
    val: u16,
    lookup: &mut HashMap<u64, u16>,
    lookup_flush: &mut HashMap<usize, u16>,
    mixed_key_perfect_hash_func: &fmph::Function
) {
    let flush_key = get_value_bits_for_flush(key, mask);
    if let Some(flush_key) = flush_key {
        //let flush_key = (mask >> (4 * is_flush.leading_zeros())) as u16;
        match lookup_flush.insert(flush_key as usize, val) {
            Some(v) => assert_eq!(val, v),
            None => (),
        };
    } else {
        let mixed_key = key as u32 as usize;
        let hash_key = mixed_key_perfect_hash_func.get(&mixed_key).unwrap();
        match lookup.insert(hash_key, val) {
            Some(v) => assert_eq!(val, v),
            None => (),
        }
    }
}

pub fn generate_lookup_tables() {
    let mut lookup = HashMap::new();
    let mut lookup_flush = HashMap::new();

    info!("Loading perfect hash func");
    let hash_func = load_perfect_hash();

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
                            &hash_func
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
                                &hash_func
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
                                    &hash_func
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
        assert_eq!(adjust_hand_rank(7140),  312);
        assert_eq!(adjust_hand_rank(7141),  311);
        assert_eq!(adjust_hand_rank(7296),  156);
        assert_eq!(adjust_hand_rank(7297),  155);
        assert_eq!(adjust_hand_rank(7452),    9);
        assert_eq!(adjust_hand_rank(7453),    8);
        assert_eq!(adjust_hand_rank(7462),    0);
    }

    //#[test]
    #[allow(dead_code)]
    fn test_generate_lookup_tables() {

        init_test_logger();

        //create_perfect_hash();
        
        //cargo test --lib --release test_generate_lookup_tables -- --nocapture
        generate_lookup_tables();
    }
}