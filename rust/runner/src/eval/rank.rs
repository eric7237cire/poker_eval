use crate::core::Card;
use bitvec::prelude::*;

/// All the different possible hand ranks.
/// For each hand rank the u32 corresponds to
/// the strength of the hand in comparison to others
/// of the same rank.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
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

/// Bit mask for the wheel (Ace, two, three, four, five)
const WHEEL: u32 = 0b1_0000_0000_1111;
/// Given a bitset of hand ranks. This method
/// will determine if there's a straight, and will give the
/// rank. Wheel is the lowest, broadway is the highest value.
///
/// Returns None if the hand ranks represented don't correspond
/// to a straight.
fn rank_straight(value_set: u32) -> Option<u32> {
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

type ValueSetType = BitArr!(for 13, in u32, Lsb0);

#[inline]
pub fn count_higher(value_set: ValueSetType, value: usize) -> u8 {
    value_set[1 + value..].count_ones() as u8
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

pub fn calc_cards_metrics(cards: &[Card]) -> CardsMetrics {
    let mut card_metrics = CardsMetrics::default();

    for c in cards.iter() {
        let v = c.value as u8;
        let s = c.suit as u8;
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

pub fn rank_cards(cards: &[Card]) -> Rank {
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
