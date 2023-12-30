/// number of ranks
pub const NUMBER_OF_RANKS: usize = 13;

/// number of ranks
pub const NUMBER_OF_CARDS: usize = 4 * NUMBER_OF_RANKS;

pub const NUMBER_OF_HOLE_CARDS: usize = 1326; //52 choose 2

//We want 32 (not 31) so we can truncate easily via as u32
pub const GLOBAL_SUIT_SHIFT: u64 = 32;

//the ranks are repurposed to have the family after bit 12
pub const RANK_FAMILY_OFFEST: u16 = 12;

// see readme, we initialize the suit count to 3, because 5 more gives us 8, the 4th bit
// we check for flushes
pub const INITIAL_SUIT_COUNT: u64 = 0x3333;

/// bit mask for checking flush; so as we start with 3, this is checking for 8
pub const FLUSH_MASK: u64 = 0x8888 << GLOBAL_SUIT_SHIFT;

pub const CARD_VALUE_MASK: u64 = 0x1fff;

pub const RANK_BASES: [u64; NUMBER_OF_RANKS] = [
    1,
    5,
    25,
    125,
    625,
    3125,
    15625,
    78125,
    390_625,
    1_953_125,
    9_765_625,
    48_828_125,
    244_140_625,
    //I don't know how these are derived, but we don't need to be as good
    //base 5 can store the counts for each value no problem
    // 0x000800, 0x002000, 0x024800, 0x025005, 0x03102e, 0x05f0e4, 0x13dc93, 0x344211, 0x35a068,
    // 0x377813, 0x378001, 0x378800, 0x380000,
];

/*
better bases that I'm not sure how they were derived, they use 25 bits instead of 31 above
// https://github.com/b-inary/holdem-hand-evaluator/blob/main/assets/src/constants.rs
        let rank_bases: [u64; 13] = [
            0x000800, 0x002000, 0x024800, 0x025005, 0x03102e, 0x05f0e4, 0x13dc93, 0x344211,
            0x35a068, 0x377813, 0x378001, 0x378800, 0x380000,
        ];

        //https://github.com/zekyll/OMPEval/blob/master/omp/HandEvaluator.cpp
        let rank_bases2: [u64; 13] = [
            0x2000, 0x8001, 0x11000, 0x3a000, 0x91000, 0x176005, 0x366000, 0x41a013, 0x47802e,
            0x479068, 0x48c0e4, 0x48f211, 0x494493,
        ];
*/

/// suit keys (club, diamond, heart, spade)
/// Follows the same order as core::card::Suit
pub const SUIT_BASES: [u64; 4] = [0x0001, 0x0010, 0x0100, 0x1000];

//Use to create the bitmasks
pub const SUIT_SHIFTS: [u64; 4] = [0, 13, 26, 39];

//Note here the bit mask is different, in core::card::Card, suit are the 1st 2 bits, then value
//so the 0-51 order would be
//0 - 2c, 1 - 2d, 2 - 2h, etc. exactly the order in this array
//But for the flush evaluation, we need the values grouped together, so instead we do
//0 - 2c, 1 - 3c, 2 - 4c,
#[rustfmt::skip]
pub const CARDS: [(u64, u64); NUMBER_OF_CARDS] = [
    /* 2c */ (RANK_BASES[0] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x1 << SUIT_SHIFTS[0]),
    /* 2d */ (RANK_BASES[0] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x1 << SUIT_SHIFTS[1]),
    /* 2h */ (RANK_BASES[0] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x1 << SUIT_SHIFTS[2]),
    /* 2s */ (RANK_BASES[0] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x1 << SUIT_SHIFTS[3]),
    /* 3c */ (RANK_BASES[1] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x2 << SUIT_SHIFTS[0]),
    /* 3d */ (RANK_BASES[1] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x2 << SUIT_SHIFTS[1]),
    /* 3h */ (RANK_BASES[1] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x2 << SUIT_SHIFTS[2]),
    /* 3s */ (RANK_BASES[1] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x2 << SUIT_SHIFTS[3]),
    /* 4c */ (RANK_BASES[2] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x4 << SUIT_SHIFTS[0]),
    /* 4d */ (RANK_BASES[2] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x4 << SUIT_SHIFTS[1]),
    /* 4h */ (RANK_BASES[2] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x4 << SUIT_SHIFTS[2]),
    /* 4s */ (RANK_BASES[2] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x4 << SUIT_SHIFTS[3]),
    /* 5c */ (RANK_BASES[3] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x8 << SUIT_SHIFTS[0]),
    /* 5d */ (RANK_BASES[3] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x8 << SUIT_SHIFTS[1]),
    /* 5h */ (RANK_BASES[3] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x8 << SUIT_SHIFTS[2]),
    /* 5s */ (RANK_BASES[3] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x8 << SUIT_SHIFTS[3]),
    /* 6c */ (RANK_BASES[4] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x10 << SUIT_SHIFTS[0]),
    /* 6d */ (RANK_BASES[4] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x10 << SUIT_SHIFTS[1]),
    /* 6h */ (RANK_BASES[4] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x10 << SUIT_SHIFTS[2]),
    /* 6s */ (RANK_BASES[4] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x10 << SUIT_SHIFTS[3]),
    /* 7c */ (RANK_BASES[5] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x20 << SUIT_SHIFTS[0]),
    /* 7d */ (RANK_BASES[5] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x20 << SUIT_SHIFTS[1]),
    /* 7h */ (RANK_BASES[5] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x20 << SUIT_SHIFTS[2]),
    /* 7s */ (RANK_BASES[5] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x20 << SUIT_SHIFTS[3]),
    /* 8c */ (RANK_BASES[6] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x40 << SUIT_SHIFTS[0]),
    /* 8d */ (RANK_BASES[6] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x40 << SUIT_SHIFTS[1]),
    /* 8h */ (RANK_BASES[6] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x40 << SUIT_SHIFTS[2]),
    /* 8s */ (RANK_BASES[6] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x40 << SUIT_SHIFTS[3]),
    /* 9c */ (RANK_BASES[7] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x80 << SUIT_SHIFTS[0]),
    /* 9d */ (RANK_BASES[7] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x80 << SUIT_SHIFTS[1]),
    /* 9h */ (RANK_BASES[7] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x80 << SUIT_SHIFTS[2]),
    /* 9s */ (RANK_BASES[7] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x80 << SUIT_SHIFTS[3]),
    /* Tc */ (RANK_BASES[8] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x100 << SUIT_SHIFTS[0]),
    /* Td */ (RANK_BASES[8] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x100 << SUIT_SHIFTS[1]),
    /* Th */ (RANK_BASES[8] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x100 << SUIT_SHIFTS[2]),
    /* Ts */ (RANK_BASES[8] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x100 << SUIT_SHIFTS[3]),
    /* Jc */ (RANK_BASES[9] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x200 << SUIT_SHIFTS[0]),
    /* Jd */ (RANK_BASES[9] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x200 << SUIT_SHIFTS[1]),
    /* Jh */ (RANK_BASES[9] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x200 << SUIT_SHIFTS[2]),
    /* Js */ (RANK_BASES[9] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x200 << SUIT_SHIFTS[3]),
    /* Qc */ (RANK_BASES[10] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x400 << SUIT_SHIFTS[0]),
    /* Qd */ (RANK_BASES[10] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x400 << SUIT_SHIFTS[1]),
    /* Qh */ (RANK_BASES[10] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x400 << SUIT_SHIFTS[2]),
    /* Qs */ (RANK_BASES[10] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x400 << SUIT_SHIFTS[3]),
    /* Kc */ (RANK_BASES[11] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x800 << SUIT_SHIFTS[0]),
    /* Kd */ (RANK_BASES[11] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x800 << SUIT_SHIFTS[1]),
    /* Kh */ (RANK_BASES[11] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x800 << SUIT_SHIFTS[2]),
    /* Ks */ (RANK_BASES[11] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x800 << SUIT_SHIFTS[3]),
    /* Ac */ (RANK_BASES[12] + (SUIT_BASES[0] << GLOBAL_SUIT_SHIFT), 0x1000 << SUIT_SHIFTS[0]),
    /* Ad */ (RANK_BASES[12] + (SUIT_BASES[1] << GLOBAL_SUIT_SHIFT), 0x1000 << SUIT_SHIFTS[1]),
    /* Ah */ (RANK_BASES[12] + (SUIT_BASES[2] << GLOBAL_SUIT_SHIFT), 0x1000 << SUIT_SHIFTS[2]),
    /* As */ (RANK_BASES[12] + (SUIT_BASES[3] << GLOBAL_SUIT_SHIFT), 0x1000 << SUIT_SHIFTS[3]),

];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::eval::pre_calc::perfect_hash::get_value_bits_for_flush;
    use crate::init_test_logger;
    use log::{info, trace};

    use super::*;
    use crate::{eval::pre_calc::perfect_hash::enumerate_all_unique_sets, CardValue};
    use crate::{Card, CardValueRange};

    #[test]
    fn test_rank_base_unique_sum() {
        //We want to test that if we have 5-7 cards and add up the rank bases, we get a unique sum
        let unique_sets = enumerate_all_unique_sets();

        assert_eq!(unique_sets.len(), 73_775);

        //Check we fit in 31 bits
        assert!(unique_sets.iter().all(|&x| x < 0x8000_0000));
    }

    #[test]
    fn test_value_suit_masks() {
        init_test_logger();

        for suit in 0..4 {
            let mut key_sum = 0;
            let mut all_set_bits = 0;

            for card_value in CardValueRange::new(CardValue::Two, CardValue::Ace) {
                let card: Card = Card::new(card_value, suit.try_into().unwrap());
                let card_usize: usize = card.into();
                let (value_suit_key, card_bit) = CARDS[card_usize];

                let value_key = value_suit_key as u32;

                //Print binary width 64 of value_suit_key
                // trace!("Card #{:>2} {} {:0>64b} {:>10x}",
                //     //card.into() as u8,
                //     card_usize,
                //     card,
                //     value_suit_key, value_suit_key);
                // trace!("Card #{:>2} {} {:0>64b} {:>10x} Value key",
                //     //card.into() as u8,
                //     card_usize,
                //     card,
                //     value_key, value_key);

                let suit_key = value_suit_key >> GLOBAL_SUIT_SHIFT;
                //trace!("{:0>64b}", suit_key);
                let suit_index: usize = card.suit as usize;
                assert_eq!(suit_key, SUIT_BASES[suit_index]);

                all_set_bits |= card_bit;

                //adds without shifting
                key_sum += value_suit_key;
            }

            assert_eq!(all_set_bits.count_ones(), NUMBER_OF_RANKS as u32);
            assert!(key_sum & FLUSH_MASK > 0);

            info!(
                "All set bits for {} suit = {:0>64b} {:x}",
                suit, all_set_bits, all_set_bits
            );

            let set_suit_bit = key_sum & FLUSH_MASK;

            //3 is because the set bit will be 8
            //this returns
            let flush_suit = set_suit_bit.trailing_zeros() as u64 - GLOBAL_SUIT_SHIFT - 3;
            //let suit_check = if flush_suit == 0 { 0 } else { flush_suit.trailing_zeros() };
            let suit_check = flush_suit / 4;

            info!(
                "Suit bit? {:0>64b} {:x} {} leading zeros {} trailing zeros {} flush suit {}",
                set_suit_bit,
                set_suit_bit,
                suit_check,
                set_suit_bit.leading_zeros(),
                set_suit_bit.trailing_zeros(),
                flush_suit
            );

            assert_eq!(suit_check, suit as u64);

            let value_bits = get_value_bits_for_flush(key_sum, all_set_bits);

            assert_eq!(value_bits.unwrap().count_ones(), NUMBER_OF_RANKS as u32);
            assert_eq!(Some(value_bits.unwrap()), Some(CARD_VALUE_MASK as u16));
        }

        //assert_eq!(value_keys_seen.len(), 73_775);
        //assert!(false);
    }
}
