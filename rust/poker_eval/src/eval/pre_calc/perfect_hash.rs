use boomphf::Mphf;
use log::info;
#[cfg(not(target_arch = "wasm32"))]
use ph::fmph;
use std::cmp::max;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{remove_file, File},
    io::Write,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::eval::pre_calc::{get_perfect_hash_path, get_boom_path};

use super::{constants::{
    CARD_VALUE_MASK, FLUSH_MASK, GLOBAL_SUIT_SHIFT, NUMBER_OF_RANKS, RANK_BASES,
}, boom::BOOM_PHF_BYTES};

pub fn get_value_bits_for_flush(raw_lookup_key: u64, card_bit_set: u64) -> Option<u16> {
    /*
    We take the raw lookup key (unhashed);
    recall this is the first 32 bits is for the values, and then the next 16 bits
    are for the suit count

    If we have a flush, this will return the up to 16 bits lookup key for the flush which is
    Bits 1_0000_0000_0000 representing
         A_KQJT_9876_5432
    */

    let set_suit_bit = raw_lookup_key & FLUSH_MASK;

    if set_suit_bit == 0 {
        return None;
    }
    //3 because a flush (starting at 3) will have bit 8 set
    let flush_suit = set_suit_bit.trailing_zeros() as u64 - GLOBAL_SUIT_SHIFT - 3;

    let suit = flush_suit / 4;

    return Some(((card_bit_set >> (suit * NUMBER_OF_RANKS as u64)) & CARD_VALUE_MASK) as u16);
}

pub fn enumerate_all_unique_sets() -> Vec<u32> {
    //rank_bases are what we add for each card value
    //so each 2 adds rank_bases[0]
    //each 3 adds rank_bases[1]
    //etc.

    let mut keys: Vec<u32> = Vec::new();

    let rank_bases = RANK_BASES;

    //enumerate all possible 5 to 7 card hands, their value sum

    //i is the lowest card, when cannot be an Ace

    //j k m are the next cards, in >= order

    //n, the 5th card, has the additional constraint that it can't be == i

    //This is an elegant way of enumerating all sums of 5-7 cards
    //with a max of 4 of any card value

    for i in 0..(NUMBER_OF_RANKS - 1) {
        for j in i..NUMBER_OF_RANKS {
            for k in j..NUMBER_OF_RANKS {
                for m in k..NUMBER_OF_RANKS {
                    for n in max(m, i + 1)..NUMBER_OF_RANKS {
                        let x = rank_bases[i] + rank_bases[j] + rank_bases[k];
                        let x = x + rank_bases[m] + rank_bases[n];
                        //5 card hand sum
                        keys.push(x as u32);
                        for p in max(n, j + 1)..NUMBER_OF_RANKS {
                            let x = x + rank_bases[p];
                            keys.push(x as u32);
                            for q in max(p, k + 1)..NUMBER_OF_RANKS {
                                let x = x + rank_bases[q];
                                keys.push(x as u32);
                            }
                        }
                    }
                }
            }
        }
    }

    keys
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_perfect_hash() {
    let unique_sets = enumerate_all_unique_sets();

    info!("Unique sets: {}", unique_sets.len());

    let f = fmph::Function::from(unique_sets.as_ref());

    //open a file hash1.dat in data dir
    let path = get_perfect_hash_path();
    remove_file(&path).unwrap_or_default();
    let mut file = File::create(&path).unwrap();

    f.write(&mut file).unwrap();
    file.flush().unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_perfect_hash_boom_phf() {
    let unique_sets = enumerate_all_unique_sets();

    info!("Unique sets: {}", unique_sets.len());

    let phf = Mphf::new(1.7, &unique_sets);

    let serialized_data = bincode::serialize(&phf).expect("Serialization failed");
    
    //println!("const MY_DATA_BYTES: &[u8] = &{:?};", serialized_data);

    // println!("const MY_DATA_BYTES: &[u8] = b\"{}\";", 
    //     serialized_data.iter()
    //                    .map(|b| format!("\\x{:02x}", b))
    //                    .collect::<String>());

    let boom_path = get_boom_path();
    let mut file = File::create(&boom_path).unwrap();
    
    writeln!(file, "pub const BOOM_PHF_BYTES: &[u8] = b\"{}\";", 
        serialized_data.iter()
                       .map(|b| format!("\\x{:02x}", b))
                       .collect::<String>()).unwrap();
    writeln!(file).unwrap();
    

    println!("wrote result to '{:?}'", &boom_path);

}

pub fn load_boomperfect_hash() -> Mphf<u32> {
    let deserialized_data:  Mphf<u32> = bincode::deserialize(BOOM_PHF_BYTES)
        .expect("Deserialization failed");

    deserialized_data
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_perfect_hash() -> fmph::Function {
    let path = get_perfect_hash_path();
    let mut file = File::open(&path).unwrap();
    let f2 = fmph::Function::read(&mut file).unwrap();

    f2
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;
    type SeenBitSet = BitArr!(for 73775, in usize, Lsb0);

    #[cfg(not(target_arch = "wasm32"))]
    //#[test]
    #[allow(dead_code)]
    fn is_perfect_hash_stable() {
        let unique_sets = enumerate_all_unique_sets();

        create_perfect_hash();

        let f = load_perfect_hash();

        create_perfect_hash();

        let f2 = load_perfect_hash();

        for s in unique_sets {
            assert_eq!(f.get(&s), f2.get(&s));
        }
    }

    #[test]
    fn is_hash_minimal() {
        let unique_sets = enumerate_all_unique_sets();

        assert_eq!(unique_sets.len(), 73_775);

        let f = load_boomperfect_hash();

        let mut seen = SeenBitSet::default();

        for s in unique_sets {
            let hash = f.hash(&s);
            assert!(hash < 73_775);
            assert!(!seen.get(hash as usize).unwrap());
            seen.set(hash as usize, true);
        }

        assert_eq!(seen.count_ones(), 73_775);
    }

    #[cfg(not(target_arch = "wasm32"))]
    //#[test]
    #[allow(dead_code)]
    fn test_create_boom() {
        //cargo test boom --lib -- --nocapture
        create_perfect_hash_boom_phf();
    }
}
