use std::borrow::Borrow;

use crate::{core::Card, CardValue, Suit};
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
pub enum Rank {
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

impl Rank {
    pub fn get_family_index(&self) -> usize {
        match self {
            Rank::HighCard(_) => 0,
            Rank::OnePair(_) => 1,
            Rank::TwoPair(_) => 2,
            Rank::ThreeOfAKind(_) => 3,
            Rank::Straight(_) => 4,
            Rank::Flush(_) => 5,
            Rank::FullHouse(_) => 6,
            Rank::FourOfAKind(_) => 7,
            Rank::StraightFlush(_) => 8,
        }
    }

    pub fn print_winning(&self, cards: &[Card]) -> String {
        let low_set_mask: u32 = 0b1_1111_1111_1111;
        match self {
            Rank::HighCard(k) => {
                let mut r = "High Card - ".to_string();
                let bvs: ValueSetType = ValueSetType::new([*k]);
                assert_eq!(5, bvs.count_ones());

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    r.push_str(&format!("{} ", card));
                }

                r.pop().unwrap();
                r
            }
            Rank::OnePair(k) => {
                let pair_value_u32 = (k >> 13).trailing_zeros();
                let pair_value: CardValue = (pair_value_u32 as u8).try_into().unwrap();
                let mut r = "One Pair - ".to_string();

                let first_card = cards.iter().find(|c| c.value == pair_value).unwrap();
                let second_card = cards.iter().rev().find(|c| c.value == pair_value).unwrap();

                r.push_str(&format!("{} {}", first_card, second_card));

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    r.push_str(&format!(" {}", card));
                }

                r
            }
            Rank::TwoPair(k) => {
                let pair_values_u32 = k >> 13;
                let pv_bvs = ValueSetType::new([pair_values_u32]);

                let mut r = "Two Pair - ".to_string();

                for set_bit in pv_bvs.iter_ones().rev() {
                    let pair_value: CardValue = set_bit.try_into().unwrap();
                    let first_card = cards.iter().find(|c| c.value == pair_value).unwrap();
                    let second_card = cards.iter().rev().find(|c| c.value == pair_value).unwrap();

                    r.push_str(&format!("{} {} ", first_card, second_card));
                }

                let last_kicker = (*k & low_set_mask).trailing_zeros();
                let last_kicker_value: CardValue = (last_kicker as u8).try_into().unwrap();
                let card = cards.iter().find(|c| c.value == last_kicker_value).unwrap();
                r.push_str(&format!("{}", card));

                r
            }
            Rank::ThreeOfAKind(k) => {
                let trips_value_u32 = (k >> 13).trailing_zeros();
                let trips_value: CardValue = (trips_value_u32 as u8).try_into().unwrap();
                let mut r = "Trips - ".to_string();

                let cards_str = cards
                    .iter()
                    .filter(|c| c.value == trips_value)
                    .map(|c| c.to_string())
                    .join(" ");

                r.push_str(&cards_str);

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    r.push_str(&format!(" {}", card));
                }

                r
            }
            Rank::Straight(k) => {
                let straight_value = if *k == 0 {
                    CardValue::Five
                } else {
                    //let straight_value_u32 = k.trailing_zeros();
                    let straight_value: CardValue = ((*k + 3) as u8).try_into().unwrap();
                    straight_value
                };

                let mut r = "Straight - ".to_string();

                trace!(
                    "Straight value {}=={}, is wheel {}",
                    straight_value,
                    straight_value as u8,
                    straight_value == CardValue::Five
                );

                if straight_value == CardValue::Five {
                    //Find the ace
                    let ace = cards.iter().find(|c| c.value == CardValue::Ace).unwrap();
                    r.push_str(&format!("{} ", ace));

                    let mut cv = CardValue::Two;
                    for _ in 0..4 {
                        let card = cards.iter().find(|c| c.value == cv).unwrap();
                        r.push_str(&format!("{} ", card));
                        cv = cv.next_card();
                    }
                } else {
                    let mut cv = CardValue::try_from(straight_value as u8 - 4).unwrap();

                    for _ in 0..5 {
                        let card = cards.iter().find(|c| c.value == cv).unwrap();
                        r.push_str(&format!("{} ", card));
                        cv = cv.next_card();
                    }
                }

                r.pop().unwrap();

                r
            }
            Rank::Flush(k) => {
                let mut r = "Flush - ".to_string();
                let bvs: ValueSetType = ValueSetType::new([*k]);
                assert_eq!(5, bvs.count_ones());

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    r.push_str(&format!("{} ", card));
                }

                r.pop().unwrap();
                r
            }
            Rank::FullHouse(k) => {
                let trips_values_u32 = k >> 13;
                let pair_value_u32 = k & low_set_mask;
                let trips_value: CardValue = (trips_values_u32.trailing_zeros() as u8)
                    .try_into()
                    .unwrap();
                let pair_value: CardValue =
                    (pair_value_u32.trailing_zeros() as u8).try_into().unwrap();

                let mut r = "Full House - ".to_string();

                let trips_cards_str = cards.iter().filter(|c| c.value == trips_value).join(" ");
                let pair_cards_str = cards.iter().filter(|c| c.value == pair_value).join(" ");

                r.push_str(&format!("{} {}", trips_cards_str, pair_cards_str));

                r
            }
            Rank::FourOfAKind(k) => {
                let quads_value_u32 = (k >> 13).trailing_zeros();
                let quads_value: CardValue = (quads_value_u32 as u8).try_into().unwrap();
                let mut r = "Quads - ".to_string();

                let cards_str = cards
                    .iter()
                    .filter(|c| c.value == quads_value)
                    .map(|c| c.to_string())
                    .join(" ");

                r.push_str(&cards_str);

                let bvs = ValueSetType::new([*k & low_set_mask]);

                for set_bit in bvs.iter_ones().rev() {
                    let value: CardValue = set_bit.try_into().unwrap();
                    let card = cards.iter().find(|c| c.value == value).unwrap();
                    r.push_str(&format!(" {}", card));
                }

                r
            }
            Rank::StraightFlush(k) => {
                let straight_value = if *k == 0 {
                    CardValue::Five
                } else {
                    //let straight_value_u32 = k.trailing_zeros();
                    let straight_value: CardValue = ((*k + 3) as u8).try_into().unwrap();
                    straight_value
                };

                let mut r = "Straight Flush - ".to_string();

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
                    r.push_str(&format!("{} ", ace));

                    let mut cv = CardValue::Two;
                    for _ in 0..4 {
                        let card = suited_cards.iter().find(|c| c.value == cv).unwrap();
                        r.push_str(&format!("{} ", card));
                        cv = cv.next_card();
                    }
                } else {
                    let mut cv = CardValue::try_from(straight_value as u8 - 4).unwrap();

                    for _ in 0..5 {
                        let card = suited_cards.iter().find(|c| c.value == cv).unwrap();
                        r.push_str(&format!("{} ", card));
                        cv = cv.next_card();
                    }
                }

                r.pop().unwrap();

                r
            }
        }
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
    pub value_to_count: [u8; 13],
    pub count_to_value: [ValueSetType; 5],
    pub suit_value_sets: [ValueSetType; 4],
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

pub fn rank_cards<I, B>(cards: I) -> Rank
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
            Rank::StraightFlush(rank)
        } else {
            // Else it's just a normal flush
            let rank = keep_n(cards_metrics.suit_value_sets[flush_idx], 5);
            Rank::Flush(rank)
        }
    } else if cards_metrics.count_to_value[4] != 0 {
        // Four of a kind.
        let high = keep_highest(cards_metrics.value_set ^ cards_metrics.count_to_value[4]);
        Rank::FourOfAKind(cards_metrics.count_to_value[4] << 13 | high)
    } else if cards_metrics.count_to_value[3] != 0
        && cards_metrics.count_to_value[3].count_ones() == 2
    {
        // There are two sets. So the best we can make is a full house.
        let set = keep_highest(cards_metrics.count_to_value[3]);
        let pair = cards_metrics.count_to_value[3] ^ set;
        Rank::FullHouse(set << 13 | pair)
    } else if cards_metrics.count_to_value[3] != 0 && cards_metrics.count_to_value[2] != 0 {
        // there is a pair and a set.
        let set = cards_metrics.count_to_value[3];
        let pair = keep_highest(cards_metrics.count_to_value[2]);
        Rank::FullHouse(set << 13 | pair)
    } else if let Some(s_rank) = rank_straight(cards_metrics.value_set) {
        // If there's a straight return it now.
        Rank::Straight(s_rank)
    } else if cards_metrics.count_to_value[3] != 0 {
        // if there is a set then we need to keep 2 cards that
        // aren't in the set.
        let low = keep_n(cards_metrics.value_set ^ cards_metrics.count_to_value[3], 2);
        Rank::ThreeOfAKind(cards_metrics.count_to_value[3] << 13 | low)
    } else if cards_metrics.count_to_value[2].count_ones() >= 2 {
        // Two pair
        //
        // That can be because we have 3 pairs and a high card.
        // Or we could have two pair and two high cards.
        let pairs = keep_n(cards_metrics.count_to_value[2], 2);
        let low = keep_highest(cards_metrics.value_set ^ pairs);
        Rank::TwoPair(pairs << 13 | low)
    } else if cards_metrics.count_to_value[2] == 0 {
        // This means that there's no pair
        // no sets, no straights, no flushes, so only a
        // high card.
        Rank::HighCard(keep_n(cards_metrics.value_set, 5))
    } else {
        // Otherwise there's only one pair.
        let pair = cards_metrics.count_to_value[2];
        // Keep the highest three cards not in the pair.
        let low = keep_n(cards_metrics.value_set ^ cards_metrics.count_to_value[2], 3);
        Rank::OnePair(pair << 13 | low)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Write};

    use crate::{add_eval_card, get_unused_card, init_test_logger, Board};
    use itertools::Itertools;
    use postflop_solver::Hand;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        get_possible_hole_cards, range_string_to_set, rank_cards, CardUsedType, HoleCards, Rank,
    };

    #[test]
    fn test_flop_rank() {
        let range_str = "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o";
        let range_set = range_string_to_set(range_str).unwrap();

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
            eval_cards.push(hole_cards.get_hi_card());
            eval_cards.push(hole_cards.get_lo_card());

            let rank = rank_cards(eval_cards.iter());
            match rank {
                Rank::HighCard(_) => {
                    num_high_card += 1;
                }
                Rank::OnePair(_) => {
                    num_pair += 1;
                }
                Rank::TwoPair(_) => {
                    num_two_pair += 1;
                }
                Rank::ThreeOfAKind(_) => {
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

    fn assert_equity(equity: f64, target: f64, tolerance: f64) {
        let passed = (equity - target).abs() < tolerance;
        if !passed {
            println!("assert_equity failed: {} != {}", equity, target);
        }
        assert!(passed);
    }

    #[test]
    fn test_enumerate_all_equity() {
        let range_str = "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o";
        let range_set = range_string_to_set(range_str).unwrap();

        let mut used_cards = CardUsedType::default();
        let flop = Board::try_from("Qs Ts 7c")
            .unwrap()
            .as_slice_card()
            .to_vec();

        let flop_hand = Hand::new();
        let flop_hand = flop_hand.add_card(flop[0].into());
        let flop_hand = flop_hand.add_card(flop[1].into());
        let flop_hand = flop_hand.add_card(flop[2].into());

        let p1_hole_cards = Board::try_from("8d 7s").unwrap().as_slice_card().to_vec();
        let p2_hole_cards = Board::try_from("Qd 5c").unwrap().as_slice_card().to_vec();

        for card in flop.iter() {
            used_cards.set((*card).into(), true);
        }
        for card in p1_hole_cards.iter() {
            used_cards.set((*card).into(), true);
        }
        for card in p2_hole_cards.iter() {
            used_cards.set((*card).into(), true);
        }
        assert_eq!(used_cards.count_ones(), 7);

        let possible = get_possible_hole_cards(&range_set, used_cards).unwrap();

        assert_eq!(373, possible.len());

        let mut win_equity = vec![0.0; 3];
        let mut tie_equity = vec![0.0; 3];

        let mut total_showdowns = 0;

        //enumerate everything
        for p in possible.iter() {
            p.set_used(&mut used_cards).unwrap();
            assert_eq!(used_cards.count_ones(), 9);
            let check_showndown = total_showdowns;
            for turn_card in 0..52 {
                if used_cards[turn_card] {
                    continue;
                }
                used_cards.set(turn_card, true);
                for river_card in turn_card + 1..52 {
                    if used_cards[river_card] {
                        continue;
                    }

                    total_showdowns += 1;

                    let mut p1_cards = flop.clone().to_vec();
                    p1_cards.push(p1_hole_cards[0]);
                    p1_cards.push(p1_hole_cards[1]);
                    p1_cards.push(turn_card.try_into().unwrap());
                    p1_cards.push(river_card.try_into().unwrap());

                    let mut p2_cards = flop.clone().to_vec();
                    p2_cards.push(p2_hole_cards[0]);
                    p2_cards.push(p2_hole_cards[1]);
                    p2_cards.push(turn_card.try_into().unwrap());
                    p2_cards.push(river_card.try_into().unwrap());

                    let mut p3_cards = flop.clone().to_vec();
                    p3_cards.push(p.get_hi_card());
                    p3_cards.push(p.get_lo_card());
                    p3_cards.push(turn_card.try_into().unwrap());
                    p3_cards.push(river_card.try_into().unwrap());

                    let p1_hand = flop_hand.add_card(p1_hole_cards[0].into());
                    let p1_hand = p1_hand.add_card(p1_hole_cards[1].into());
                    let p1_hand = p1_hand.add_card(turn_card.into());
                    let p1_hand = p1_hand.add_card(river_card.into());

                    let p2_hand = flop_hand.add_card(p2_hole_cards[0].into());
                    let p2_hand = p2_hand.add_card(p2_hole_cards[1].into());
                    let p2_hand = p2_hand.add_card(turn_card.into());
                    let p2_hand = p2_hand.add_card(river_card.into());

                    let p3_hand = flop_hand.add_card(p.get_hi_card().into());
                    let p3_hand = p3_hand.add_card(p.get_lo_card().into());
                    let p3_hand = p3_hand.add_card(turn_card.into());
                    let p3_hand = p3_hand.add_card(river_card.into());

                    let p1_rank = rank_cards(p1_cards.iter());
                    let p2_rank = rank_cards(p2_cards.iter());
                    let p3_rank = rank_cards(p3_cards.iter());

                    let p1_eval = p1_hand.evaluate_internal();
                    let p2_eval = p2_hand.evaluate_internal();
                    let p3_eval = p3_hand.evaluate_internal();

                    if p1_rank == p2_rank && p2_rank == p3_rank {
                        assert_eq!(p1_eval, p2_eval);
                        assert_eq!(p1_eval, p3_eval);
                        tie_equity[0] += 1.0 / 3.0;
                        tie_equity[1] += 1.0 / 3.0;
                        tie_equity[2] += 1.0 / 3.0;
                    } else if p1_rank == p2_rank && p1_rank > p3_rank {
                        assert_eq!(p1_eval, p2_eval);
                        tie_equity[0] += 1.0 / 2.0;
                        tie_equity[1] += 1.0 / 2.0;
                    } else if p1_rank == p3_rank && p1_rank > p2_rank {
                        assert_eq!(p1_eval, p3_eval);
                        tie_equity[0] += 1.0 / 2.0;
                        tie_equity[2] += 1.0 / 2.0;
                    } else if p2_rank == p3_rank && p2_rank > p1_rank {
                        assert_eq!(p2_eval, p3_eval);
                        tie_equity[1] += 1.0 / 2.0;
                        tie_equity[2] += 1.0 / 2.0;
                    } else {
                        let mut ranks = vec![p1_rank, p2_rank, p3_rank];
                        ranks.sort();
                        if ranks[2] == p1_rank {
                            assert!(p1_eval > p2_eval);
                            assert!(p1_eval > p3_eval);
                            win_equity[0] += 1.0;
                        } else if ranks[2] == p2_rank {
                            assert!(p2_eval > p1_eval);
                            assert!(p2_eval > p3_eval);
                            win_equity[1] += 1.0;
                        } else {
                            assert!(p3_eval > p1_eval);
                            assert!(p3_eval > p2_eval);
                            win_equity[2] += 1.0;
                        }
                    }

                    //used_cards.set(river_card, false);
                }

                used_cards.set(turn_card, false);
            }

            p.unset_used(&mut used_cards).unwrap();

            assert_eq!(used_cards.count_ones(), 7);
            //we used 9 cards, so there should be 43*42/2 showdowns total
            assert_eq!(total_showdowns - check_showndown, 43 * 42 / 2);
        }

        //Values from Equilab
        assert_equity(100.0 * tie_equity[0] / total_showdowns as f64, 0.12, 0.005);
        assert_equity(100.0 * win_equity[0] / total_showdowns as f64, 21.03, 0.005);

        assert_equity(100.0 * tie_equity[1] / total_showdowns as f64, 0.82, 0.005);
        assert_equity(100.0 * win_equity[1] / total_showdowns as f64, 50.93, 0.005);

        assert_equity(100.0 * tie_equity[2] / total_showdowns as f64, 0.95, 0.005);
        assert_equity(100.0 * win_equity[2] / total_showdowns as f64, 26.14, 0.005);
    }

    //Slow
    //#[test]
    #[allow(dead_code)]
    fn test_heads_up_ranking() {
        //let mut range_idx_to_string: Vec<String> = Vec::new();
        //let mut range_idx_to_equity: Vec<f64> = Vec::new();

        let mut range_string_to_idx: HashMap<String, usize> = HashMap::new();

        let mut hole_card_list = Vec::new();
        let mut hole_index_list = Vec::new();
        let mut hole_range_strings = Vec::new();

        for card1 in 0..52usize {
            for card2 in card1 + 1..52 {
                let hole_cards =
                    HoleCards::new(card1.try_into().unwrap(), card2.try_into().unwrap()).unwrap();

                let hole_string = hole_cards.to_range_string();

                // if !range_string_to_idx.contains_key(&hole_string) {
                //     println!("{} {}", hole_string, range_idx_to_string.len());
                // }

                hole_card_list.push(hole_cards);
                //range_idx_to_equity.push(0.0);

                let l = range_string_to_idx.len();
                let range_str_index = range_string_to_idx.entry(hole_string.clone()).or_insert(l);

                if hole_range_strings.len() <= *range_str_index {
                    hole_range_strings.push(hole_string.clone());
                }

                hole_index_list.push(*range_str_index);
            }
        }

        assert_eq!(13 * 13, range_string_to_idx.len());
        assert_eq!(range_string_to_idx.len(), hole_range_strings.len());

        assert_eq!(hole_card_list.len(), hole_index_list.len());
        assert_eq!(52 * 51 / 2, hole_card_list.len());

        //let mut hole_idx_to_eq = vec![0.0; hole_card_list.len()];

        let mut range_hole_idx_to_eq = vec![0.0; 13 * 13];
        let mut range_hole_idx_showdown_count = vec![0; 13 * 13];

        //change this to be higher for more accuracy, kept to 1 for speed
        let num_flops_per_matchup = 1;

        let mut rng = StdRng::seed_from_u64(42);

        for (h1_idx, h1) in hole_card_list.iter().enumerate() {
            for h2_idx in h1_idx + 1..hole_card_list.len() {
                let r1_idx = hole_index_list[h1_idx];
                let r2_idx = hole_index_list[h2_idx];

                //no need to do this matchup
                if r1_idx == r2_idx {
                    continue;
                }

                let h2 = &hole_card_list[h2_idx];

                let mut eval_cards = Vec::with_capacity(15);
                let mut cards_used = CardUsedType::default();
                h1.set_used(&mut cards_used).unwrap();

                let is_ok = h2.set_used(&mut cards_used);

                if !is_ok.is_ok() {
                    //some possibilites won't be valid
                    continue;
                }

                for _ in 0..num_flops_per_matchup {
                    assert_eq!(4, cards_used.count_ones());
                    for _ in 0..5 {
                        add_eval_card(
                            get_unused_card(&mut rng, &cards_used).unwrap(),
                            &mut eval_cards,
                            &mut cards_used,
                        )
                        .unwrap();
                    }

                    assert_eq!(4 + 5, cards_used.count_ones());

                    h1.add_to_eval(&mut eval_cards);
                    assert_eq!(7, eval_cards.len());
                    let rank1 = rank_cards(eval_cards.iter());
                    h1.remove_from_eval(&mut eval_cards).unwrap();

                    h2.add_to_eval(&mut eval_cards);
                    assert_eq!(7, eval_cards.len());
                    let rank2 = rank_cards(eval_cards.iter());
                    h2.remove_from_eval(&mut eval_cards).unwrap();

                    range_hole_idx_showdown_count[r1_idx] += 1;
                    range_hole_idx_showdown_count[r2_idx] += 1;
                    if rank1 == rank2 {
                        range_hole_idx_to_eq[r1_idx] += 1.0 / 2.0;
                        range_hole_idx_to_eq[r2_idx] += 1.0 / 2.0;
                    } else if rank1 > rank2 {
                        range_hole_idx_to_eq[r1_idx] += 1.0;
                    } else {
                        range_hole_idx_to_eq[r2_idx] += 1.0;
                    }

                    //Pop off the 5
                    for _ in 0..5 {
                        let c = eval_cards.pop().unwrap();
                        cards_used.set(c.into(), false);
                    }
                }
            }
        }

        for r_idx in 0..range_hole_idx_to_eq.len() {
            range_hole_idx_to_eq[r_idx] /= range_hole_idx_showdown_count[r_idx] as f64;
        }

        let mut with_idx = range_hole_idx_to_eq.iter().enumerate().collect_vec();
        with_idx.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        for (rank, (r_idx, eq)) in with_idx.iter().enumerate().take(10) {
            println!(
                "#{} {} {} ",
                1 + rank,
                hole_range_strings[*r_idx],
                100.0 * **eq
            );
        }

        //open a text file and write the results

        let mut file = std::fs::File::create("/tmp/heads_up_equity.txt").unwrap();
        for (r_idx, _) in with_idx.iter() {
            let line = format!("{}\n", hole_range_strings[*r_idx]);
            file.write_all(line.as_bytes()).unwrap();
        }
    }

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
    }
}
