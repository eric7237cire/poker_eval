use std::borrow::Borrow;

use crate::{core::Card, CardValue, CardValueRange, Suit, ALL_CARDS};
use bitvec::prelude::*;
use itertools::Itertools;
use log::trace;
use serde::{Deserialize, Serialize};

/// All the different possible hand ranks.
/// For each hand rank the u32 corresponds to
/// the strength of the hand in comparison to others
/// of the same rank.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy, Serialize, Deserialize)]
pub enum OldRank {
    /// The lowest rank.
    /// No matches
    HighCard(u32),
    /// One Card matches another.
    OnePair(u32),
    /// Two different pair of matching cards.
    TwoPair(u32),
    /// Three of the same value.
    ThreeOfAKind(u32),
    /// Five cards in a sequence
    Straight(u32),
    /// Five cards of the same suit
    Flush(u32),
    /// Three of one value and two of another value
    FullHouse(u32),
    /// Four of the same value.
    FourOfAKind(u32),
    /// Five cards in a sequence all for the same suit.
    StraightFlush(u32),
}

pub const NUM_RANK_FAMILIES: usize = 9;

impl OldRank {
    pub fn get_family_index(&self) -> usize {
        match self {
            OldRank::HighCard(_) => 0,
            OldRank::OnePair(_) => 1,
            OldRank::TwoPair(_) => 2,
            OldRank::ThreeOfAKind(_) => 3,
            OldRank::Straight(_) => 4,
            OldRank::Flush(_) => 5,
            OldRank::FullHouse(_) => 6,
            OldRank::FourOfAKind(_) => 7,
            OldRank::StraightFlush(_) => 8,
        }
    }

    pub fn print_winning(&self, cards: &[Card]) -> String {
        
        let winning_cards = self.get_winning(cards);
        let winning_cards_str = winning_cards.iter().map(|c| c.to_string()).join(" ");

        match self {
            OldRank::HighCard(_) => {
                format!("High Card - {}", winning_cards_str)
            }
            OldRank::OnePair(_) => {
                format!("One Pair - {}", winning_cards_str)
            }
            OldRank::TwoPair(_) => {                
                format!("Two Pair - {}", winning_cards_str)
            }
            OldRank::ThreeOfAKind(_) => {
                format!("Trips - {}", winning_cards_str)
            }
            OldRank::Straight(_) => {
                format!("Straight - {}", winning_cards_str)
            }
            OldRank::Flush(_) => {
                format!("Flush - {}", winning_cards_str)
            }
            OldRank::FullHouse(_) => {
                format!("Full House - {}", winning_cards_str)
            }
            OldRank::FourOfAKind(_) => {
                format!("Quads - {}", winning_cards_str)
            }
            OldRank::StraightFlush(_) => {
                format!("Straight Flush - {}", winning_cards_str)
            }
        }
    }

    /*
    Gets winning cards in order

    High Card: HC 1st Kicker 2nd Kicker 3rd Kicker 4th Kicker
    Pair: P P 1st Kicker 2nd Kicker 3rd Kicker
    Trips: T T T 1st Kicker 2nd Kicker
    FH: T T T P P
    Stright: Highest => Lowest
    Flush: F F F F F
    Quads: Q Q Q Q 1st Kicker

    */
    pub fn get_winning(&self, cards: &[Card]) -> [Card; 5] {
        let low_set_mask: u32 = 0b1_1111_1111_1111;
        let mut ret = [ALL_CARDS[0]; 5];
        let mut cur_card_index = 0;

        match self {
            OldRank::HighCard(k) => {
                let bvs: ValueSetType = ValueSetType::new([*k]);
                assert_eq!(5, bvs.count_ones());

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();

                    ret[cur_card_index] = *card;
                    cur_card_index += 1;
                }
            }
            OldRank::OnePair(k) => {
                let pair_value_u32 = (k >> 13).trailing_zeros();
                let pair_value: CardValue = (pair_value_u32 as u8).try_into().unwrap();

                let first_card = cards.iter().find(|c| c.value == pair_value).unwrap();
                let second_card = cards.iter().rev().find(|c| c.value == pair_value).unwrap();

                ret[cur_card_index] = *first_card;
                cur_card_index += 1;
                ret[cur_card_index] = *second_card;
                cur_card_index += 1;

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    ret[cur_card_index] = *card;
                    cur_card_index += 1;
                }
            }
            OldRank::TwoPair(k) => {
                let pair_values_u32 = k >> 13;
                let pv_bvs = ValueSetType::new([pair_values_u32]);

                for set_bit in pv_bvs.iter_ones().rev() {
                    let pair_value: CardValue = set_bit.try_into().unwrap();
                    let first_card = cards.iter().find(|c| c.value == pair_value).unwrap();
                    let second_card = cards.iter().rev().find(|c| c.value == pair_value).unwrap();

                    ret[cur_card_index] = *first_card;
                    cur_card_index += 1;
                    ret[cur_card_index] = *second_card;
                    cur_card_index += 1;
                }

                let last_kicker = (*k & low_set_mask).trailing_zeros();
                let last_kicker_value: CardValue = (last_kicker as u8).try_into().unwrap();
                let card = cards.iter().find(|c| c.value == last_kicker_value).unwrap();
                ret[cur_card_index] = *card;
            }
            OldRank::ThreeOfAKind(k) => {
                let trips_value_u32 = (k >> 13).trailing_zeros();
                let trips_value: CardValue = (trips_value_u32 as u8).try_into().unwrap();

                for c in cards.iter() {
                    if c.value == trips_value {
                        ret[cur_card_index] = *c;
                        cur_card_index += 1;
                    }
                }

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    ret[cur_card_index] = *card;
                    cur_card_index += 1;
                }
            }
            OldRank::Straight(k) => {
                let straight_value = if *k == 0 {
                    CardValue::Five
                } else {
                    //let straight_value_u32 = k.trailing_zeros();
                    let straight_value: CardValue = ((*k + 3) as u8).try_into().unwrap();
                    straight_value
                };

                if straight_value == CardValue::Five {
                    //Find the ace
                    let ace = cards.iter().find(|c| c.value == CardValue::Ace).unwrap();
                    ret[cur_card_index] = *ace;
                    cur_card_index += 1;

                    for card_value in CardValueRange::new(CardValue::Two, CardValue::Five) {
                        let card = cards.iter().find(|c| c.value == card_value).unwrap();
                        ret[cur_card_index] = *card;
                        cur_card_index += 1;
                    }
                } else {
                    let start = CardValue::try_from(straight_value as u8 - 4).unwrap();

                    for card_value in CardValueRange::new(start, straight_value) {
                        let card = cards.iter().find(|c| c.value == card_value).unwrap();
                        ret[cur_card_index] = *card;
                        cur_card_index += 1;
                    }
                }
            }
            OldRank::Flush(k) => {
                let bvs: ValueSetType = ValueSetType::new([*k]);
                assert_eq!(5, bvs.count_ones());

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    ret[cur_card_index] = *card;
                    cur_card_index += 1;
                }
            }
            OldRank::FullHouse(k) => {
                let trips_values_u32 = k >> 13;
                let pair_value_u32 = k & low_set_mask;
                let trips_value: CardValue = (trips_values_u32.trailing_zeros() as u8)
                    .try_into()
                    .unwrap();
                let pair_value: CardValue =
                    (pair_value_u32.trailing_zeros() as u8).try_into().unwrap();

                for c in cards {
                    if c.value == trips_value {
                        ret[cur_card_index] = *c;
                        cur_card_index += 1;
                    }
                }
                //it's possible to have TTT PPP
                for c in cards {
                    if c.value == pair_value && cur_card_index < 5 {
                        ret[cur_card_index] = *c;
                        cur_card_index += 1;
                    }
                }
            }
            OldRank::FourOfAKind(k) => {
                let quads_value_u32 = (k >> 13).trailing_zeros();
                let quads_value: CardValue = (quads_value_u32 as u8).try_into().unwrap();

                for c in cards {
                    if c.value == quads_value {
                        ret[cur_card_index] = *c;
                        cur_card_index += 1;
                    }
                }

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    ret[cur_card_index] = *card;
                    cur_card_index += 1;
                }
            }
            OldRank::StraightFlush(k) => {
                let straight_value = if *k == 0 {
                    CardValue::Five
                } else {
                    //let straight_value_u32 = k.trailing_zeros();
                    let straight_value: CardValue = ((*k + 3) as u8).try_into().unwrap();
                    straight_value
                };

                //Find most common suit
                let mut suit_counts = [0; 4];
                for card in cards.iter() {
                    suit_counts[card.suit as usize] += 1;
                }
                let (max_suit, _max_suit_count) = suit_counts
                    .iter()
                    .enumerate()
                    .max_by_key(|&(_, count)| count)
                    .unwrap();
                let max_suit_class: Suit = (max_suit as u8).try_into().unwrap();

                let suited_cards = cards
                    .iter()
                    .filter(|c| c.suit == max_suit_class)
                    .collect::<Vec<_>>();

                trace!(
                    "Straight value {}=={}, is wheel {}",
                    straight_value,
                    straight_value as u8,
                    straight_value == CardValue::Five
                );

                if straight_value == CardValue::Five {
                    //Find the ace
                    let ace = suited_cards
                        .iter()
                        .find(|c| c.value == CardValue::Ace)
                        .unwrap();
                    ret[cur_card_index] = **ace;
                    cur_card_index += 1;

                    for cv in CardValueRange::new(CardValue::Two, CardValue::Five) {
                        let card = suited_cards.iter().find(|c| c.value == cv).unwrap();
                        ret[cur_card_index] = **card;
                        cur_card_index += 1;
                    }
                } else {
                    let start = CardValue::try_from(straight_value as u8 - 4).unwrap();

                    for cv in CardValueRange::new(start, straight_value) {
                        let card = suited_cards.iter().find(|c| c.value == cv).unwrap();
                        ret[cur_card_index] = **card;
                        cur_card_index += 1;
                    }
                }
            }
        }

        ret
    }
}

/// Bit mask for the wheel (Ace, two, three, four, five)
const WHEEL: u32 = 0b1_0000_0000_1111;
/// Given a bitset of hand ranks. This method
/// will determine if there's a straight, and will give the
/// rank. Wheel is the lowest, broadway is the highest value.
///
/// Returns None if the hand ranks represented don't correspond
/// to a straight.
pub fn rank_straight(value_set: u32) -> Option<u32> {
    // Example of something with a straight:
    //       0000111111100
    //       0001111111000
    //       0011111110000
    //       0111111100000
    //       1111111000000
    //       -------------
    //       0000111000000
    //
    // So there were seven ones in a row
    // we removed the bottom 4.
    //
    // Now an example of an almost straight:
    //
    //       0001110111100
    //       0011101111000
    //       0111011110000
    //       1110111100000
    //       1101111000000
    //       -------------
    //       0000000000000
    let left =
        value_set & (value_set << 1) & (value_set << 2) & (value_set << 3) & (value_set << 4);
    // Now count the leading 0's
    let idx = left.leading_zeros();
    // If this isn't all zeros then we found a straight
    if idx < 32 {
        Some(32 - 4 - idx)
    } else if value_set & WHEEL == WHEEL {
        // Check to see if this is the wheel. It's pretty unlikely.
        Some(0)
    } else {
        // We found nothing.
        None
    }
}
/// Keep only the most significant bit.
fn keep_highest(rank: u32) -> u32 {
    1 << (32 - rank.leading_zeros() - 1)
}
/// Keep the N most significant bits.
///
/// This works by removing the least significant bits.
fn keep_n(rank: u32, to_keep: u32) -> u32 {
    let mut result = rank;
    while result.count_ones() > to_keep {
        result &= result - 1;
    }
    result
}
/// From a slice of values sets find if there's one that has a
/// flush
fn find_flush(suit_value_sets: &[u32]) -> Option<usize> {
    suit_value_sets.iter().position(|sv| sv.count_ones() >= 5)
}

pub type ValueSetType = BitArr!(for 13, in u32, Lsb0);

#[inline]
pub fn count_higher(value_set: ValueSetType, value: usize) -> u8 {
    if value == 12 {
        return 0;
    }
    value_set[1 + value..].count_ones() as u8
}
#[inline]
pub fn count_lower(value_set: ValueSetType, value: usize) -> u8 {
    if value == 0 {
        return 0;
    }
    value_set[0..value].count_ones() as u8
}

pub struct BitSetCardsMetrics {
    //value_to_count[0] is how many 2s, value_to_count[1] is how many 3s, etc.
    pub value_to_count: [u8; 13],
    //count_to_value[0] is a bitset of all the card values that have 0 cards
    //count_to_value[1] is a bitset of all the card values that have 1 card, etc.
    pub count_to_value: [ValueSetType; 5],

    //suit_value_sets[0] is a bitset of all the values that have a club
    pub suit_value_sets: [ValueSetType; 4],

    //value_set is a bitset of all the values that are in the hand
    pub value_set: ValueSetType,
}

impl Default for BitSetCardsMetrics {
    fn default() -> Self {
        BitSetCardsMetrics {
            value_to_count: [0; 13],
            //so count_to_value[2] is a bitset of all the values that have 2 cards (paired)
            count_to_value: [ValueSetType::default(); 5],
            suit_value_sets: [ValueSetType::default(); 4],
            value_set: ValueSetType::default(),
        }
    }
}

pub fn calc_bitset_cards_metrics(cards: &[Card]) -> BitSetCardsMetrics {
    let mut card_metrics = BitSetCardsMetrics::default();

    for c in cards.iter() {
        let v = c.value as u8;
        let s = c.suit as u8;
        card_metrics.value_set.set(v as usize, true);
        card_metrics.value_to_count[v as usize] += 1;
        card_metrics.suit_value_sets[s as usize].set(v as usize, true);
    }

    // Now rotate the value to count map.
    for (value, &count) in card_metrics.value_to_count.iter().enumerate() {
        card_metrics.count_to_value[count as usize].set(value, true)
    }

    card_metrics
}

pub struct CardsMetrics {
    pub value_to_count: [u8; 13],
    pub count_to_value: [u32; 5],
    pub suit_value_sets: [u32; 4],
    pub value_set: u32,
}

impl Default for CardsMetrics {
    fn default() -> Self {
        CardsMetrics {
            value_to_count: [0; 13],
            count_to_value: [0; 5],
            suit_value_sets: [0; 4],
            value_set: 0,
        }
    }
}

pub fn calc_cards_metrics<I, B>(cards: I) -> CardsMetrics
where
    I: Iterator<Item = B>,
    //to accept both Card and &Card
    B: Borrow<Card>,
{
    let mut card_metrics = CardsMetrics::default();

    for c in cards {
        let v = c.borrow().value as u8;
        let s = c.borrow().suit as u8;
        card_metrics.value_set |= 1 << v;
        card_metrics.value_to_count[v as usize] += 1;
        card_metrics.suit_value_sets[s as usize] |= 1 << v;
    }

    // Now rotate the value to count map.
    for (value, &count) in card_metrics.value_to_count.iter().enumerate() {
        card_metrics.count_to_value[count as usize] |= 1 << value;
    }

    card_metrics
}

pub fn rank_cards<I, B>(cards: I) -> OldRank
where
    I: Iterator<Item = B>,
    //to accept both Card and &Card
    B: Borrow<Card>,
{
    //assert!(cards.len() <= 7);
    let cards_metrics = calc_cards_metrics(cards);

    // Find out if there's a flush
    let flush: Option<usize> = find_flush(&cards_metrics.suit_value_sets);

    // If this is a flush then it could be a straight flush
    // or a flush. So check only once.
    if let Some(flush_idx) = flush {
        // If we can find a straight in the flush then it's a straight flush
        if let Some(rank) = rank_straight(cards_metrics.suit_value_sets[flush_idx]) {
            OldRank::StraightFlush(rank)
        } else {
            // Else it's just a normal flush
            let rank = keep_n(cards_metrics.suit_value_sets[flush_idx], 5);
            OldRank::Flush(rank)
        }
    } else if cards_metrics.count_to_value[4] != 0 {
        // Four of a kind.
        let high = keep_highest(cards_metrics.value_set ^ cards_metrics.count_to_value[4]);
        OldRank::FourOfAKind(cards_metrics.count_to_value[4] << 13 | high)
    } else if cards_metrics.count_to_value[3] != 0
        && cards_metrics.count_to_value[3].count_ones() == 2
    {
        // There are two sets. So the best we can make is a full house.
        let set = keep_highest(cards_metrics.count_to_value[3]);
        let pair = cards_metrics.count_to_value[3] ^ set;
        OldRank::FullHouse(set << 13 | pair)
    } else if cards_metrics.count_to_value[3] != 0 && cards_metrics.count_to_value[2] != 0 {
        // there is a pair and a set.
        let set = cards_metrics.count_to_value[3];
        let pair = keep_highest(cards_metrics.count_to_value[2]);
        OldRank::FullHouse(set << 13 | pair)
    } else if let Some(s_rank) = rank_straight(cards_metrics.value_set) {
        // If there's a straight return it now.
        OldRank::Straight(s_rank)
    } else if cards_metrics.count_to_value[3] != 0 {
        // if there is a set then we need to keep 2 cards that
        // aren't in the set.
        let low = keep_n(cards_metrics.value_set ^ cards_metrics.count_to_value[3], 2);
        OldRank::ThreeOfAKind(cards_metrics.count_to_value[3] << 13 | low)
    } else if cards_metrics.count_to_value[2].count_ones() >= 2 {
        // Two pair
        //
        // That can be because we have 3 pairs and a high card.
        // Or we could have two pair and two high cards.
        let pairs = keep_n(cards_metrics.count_to_value[2], 2);
        let low = keep_highest(cards_metrics.value_set ^ pairs);
        OldRank::TwoPair(pairs << 13 | low)
    } else if cards_metrics.count_to_value[2] == 0 {
        // This means that there's no pair
        // no sets, no straights, no flushes, so only a
        // high card.
        OldRank::HighCard(keep_n(cards_metrics.value_set, 5))
    } else {
        // Otherwise there's only one pair.
        let pair = cards_metrics.count_to_value[2];
        // Keep the highest three cards not in the pair.
        let low = keep_n(cards_metrics.value_set ^ cards_metrics.count_to_value[2], 3);
        OldRank::OnePair(pair << 13 | low)
    }
}

#[cfg(test)]
mod tests {
    use crate::{init_test_logger, Board, BoolRange};

    use crate::{get_possible_hole_cards, rank_cards, CardUsedType, OldRank};

    #[test]
    fn test_flop_rank() {
        let range_str = "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o";
        let range: BoolRange = range_str.parse().unwrap();
        let range_set = &range.data;

        let mut used_cards = CardUsedType::default();
        let flop = Board::try_from("Qs Ts 7c")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let other_cards = Board::try_from("8d 7s Qd 5c")
            .unwrap()
            .as_slice_card()
            .to_vec();

        for card in flop.iter() {
            used_cards.set((*card).into(), true);
        }
        for card in other_cards.iter() {
            used_cards.set((*card).into(), true);
        }

        let possible = get_possible_hole_cards(&range_set, used_cards).unwrap();

        assert_eq!(373, possible.len());

        let mut num_trips = 0;
        let mut num_two_pair = 0;
        let mut num_pair = 0;
        let mut num_high_card = 0;

        /*let mut over_pocket_pair = 0;
        let mut ace_high = 0;
        let mut king_high = 0;*/

        let mut eval_cards = flop.clone().to_vec();

        for hole_cards in possible.iter() {
            eval_cards.push(hole_cards.hi_card());
            eval_cards.push(hole_cards.lo_card());

            let rank = rank_cards(eval_cards.iter());
            match rank {
                OldRank::HighCard(_) => {
                    num_high_card += 1;
                }
                OldRank::OnePair(_) => {
                    num_pair += 1;
                }
                OldRank::TwoPair(_) => {
                    num_two_pair += 1;
                }
                OldRank::ThreeOfAKind(_) => {
                    num_trips += 1;
                }
                _ => {}
            }

            eval_cards.pop();
            eval_cards.pop();
        }

        assert_eq!(num_trips, 5);
        assert_eq!(num_two_pair, 3);
        let check_ace_high = 136;
        let check_king_high = 63;

        let check_top_pair = 37;
        let check_middle_pair = 36;
        let check_low_pair = 24;

        let check_low_pocket_pair = 36;
        let check_second_pocket_pair = 6;
        let check_over_pocket_pair = 12;

        let check_num_pairs = check_top_pair
            + check_middle_pair
            + check_low_pocket_pair
            + check_second_pocket_pair
            + check_over_pocket_pair
            + check_low_pair;

        assert_eq!(num_pair, check_num_pairs);

        let check_one_overcard = 183;
        let check_two_overcards = 16;
        let check_nothing = 15;

        let check_highcard = check_one_overcard + check_two_overcards + check_nothing;

        assert_eq!(num_high_card, check_highcard);
        assert_eq!(
            check_ace_high + check_king_high,
            check_one_overcard + check_two_overcards
        );

        assert_eq!(
            0,
            373 - num_trips - num_two_pair - num_high_card - check_num_pairs
        );
    }

    // fn assert_equity(equity: f64, target: f64, tolerance: f64) {
    //     let passed = (equity - target).abs() < tolerance;
    //     if !passed {
    //         println!("assert_equity failed: {} != {}", equity, target);
    //     }
    //     assert!(passed);
    // }

    #[test]
    fn test_print_winning() {
        init_test_logger();

        let cards: Board = "8h Js 3d Qd 2h Tc 7h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "High Card - Qd Js Tc 8h 7h",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "8h 2s 3d Qd 2h Tc 7h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "One Pair - 2s 2h Qd Tc 8h",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "8h 2s Td Qd 2h Tc 7h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Two Pair - Td Tc 2s 2h Qd",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "8h 2s Td Qd 2h Ac 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Trips - 2s 2h 2c Ac Qd",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "5h 3s 5d Ad 4h Ac 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight - Ad 2c 3s 4h 5h",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "5h 3s 5d Ad 4h 6c 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight - 2c 3s 4h 5h 6c",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "Kh Js 5d Ad Th Qc 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight - Th Js Qc Kh Ad",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "2h Js 8h Ah Th 3h 7h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Flush - Ah Th 8h 7h 3h",
            rank.print_winning(&cards.as_slice_card())
        );

        let cards: Board = "2h Js As Ah Th 2s 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Full House - 2h 2s 2c As Ah",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "2h Js 2d 9h Th 2s 2c".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Quads - 2h 2d 2s 2c Js",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "2h Ah 2d 3h Th 4h 5h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight Flush - Ah 2h 3h 4h 5h",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "2h As 2d 3h Ah 4h 5h".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight Flush - Ah 2h 3h 4h 5h",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "9s 7s As Js Qh Ts 8s".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight Flush - 7s 8s 9s Ts Js",
            rank.print_winning(cards.as_slice_card())
        );

        let cards: Board = "Qc 2d 5c 8d 3h As 4d".parse().unwrap();
        let rank = rank_cards(cards.as_slice_card().iter());
        assert_eq!(
            "Straight - As 2d 3h 4d 5c",
            rank.print_winning(cards.as_slice_card())
        );
    }
}
