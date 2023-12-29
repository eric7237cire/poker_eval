use std::{
    cmp::max,
    fs::{File, remove_file}, io::Write,
};
use log::info;
use ph::fmph;

use crate::eval::pre_calc::get_perfect_hash_path;

use super::constants::{RANK_BASES, NUMBER_OF_RANKS, FLUSH_MASK, GLOBAL_SUIT_SHIFT, CARD_VALUE_MASK};

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

    return Some( ((card_bit_set >> (suit * NUMBER_OF_RANKS as u64)) & CARD_VALUE_MASK) as u16 );
}

pub fn enumerate_all_unique_sets() -> Vec<u64> {

    //rank_bases are what we add for each card value 
    //so each 2 adds rank_bases[0]
    //each 3 adds rank_bases[1]
    //etc. 

    let mut keys: Vec<u64> = Vec::new();

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
                        keys.push(x);
                        for p in max(n, j + 1)..NUMBER_OF_RANKS {
                            let x = x + rank_bases[p];
                            keys.push(x);
                            for q in max(p, k + 1)..NUMBER_OF_RANKS {
                                let x = x + rank_bases[q];
                                keys.push(x);
                            }
                        }
                    }
                }
            }
        }
    }

    keys
}


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

pub fn load_perfect_hash() -> fmph::Function {
    let path = get_perfect_hash_path();
    let mut file = File::open(&path).unwrap();
    let f2 = fmph::Function::read(&mut file).unwrap();

    f2 
}
