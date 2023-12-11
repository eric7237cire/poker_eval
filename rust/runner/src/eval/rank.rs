use crate::core::Card;


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

pub fn rank_cards(cards: &[Card]) -> Rank {
    let mut value_to_count: [u8; 13] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut count_to_value: [u32; 5] = [0, 0, 0, 0, 0];
    let mut suit_value_sets: [u32; 4] = [0, 0, 0, 0];
    let mut value_set: u32 = 0;

    for c in cards.iter() {
        let v = c.value as u8;
        let s = c.suit as u8;
        value_set |= 1 << v;
        value_to_count[v as usize] += 1;
        suit_value_sets[s as usize] |= 1 << v;
    }

    // Now rotate the value to count map.
    for (value, &count) in value_to_count.iter().enumerate() {
        count_to_value[count as usize] |= 1 << value;
    }

    // Find out if there's a flush
    let flush: Option<usize> = find_flush(&suit_value_sets);

    // If this is a flush then it could be a straight flush
    // or a flush. So check only once.
    if let Some(flush_idx) = flush {
        // If we can find a straight in the flush then it's a straight flush
        if let Some(rank) = rank_straight(suit_value_sets[flush_idx]) {
            Rank::StraightFlush(rank)
        } else {
            // Else it's just a normal flush
            let rank = keep_n(suit_value_sets[flush_idx], 5);
            Rank::Flush(rank)
        }
    } else if count_to_value[4] != 0 {
        // Four of a kind.
        let high = keep_highest(value_set ^ count_to_value[4]);
        Rank::FourOfAKind(count_to_value[4] << 13 | high)
    } else if count_to_value[3] != 0 && count_to_value[3].count_ones() == 2 {
        // There are two sets. So the best we can make is a full house.
        let set = keep_highest(count_to_value[3]);
        let pair = count_to_value[3] ^ set;
        Rank::FullHouse(set << 13 | pair)
    } else if count_to_value[3] != 0 && count_to_value[2] != 0 {
        // there is a pair and a set.
        let set = count_to_value[3];
        let pair = keep_highest(count_to_value[2]);
        Rank::FullHouse(set << 13 | pair)
    } else if let Some(s_rank) = rank_straight(value_set) {
        // If there's a straight return it now.
        Rank::Straight(s_rank)
    } else if count_to_value[3] != 0 {
        // if there is a set then we need to keep 2 cards that
        // aren't in the set.
        let low = keep_n(value_set ^ count_to_value[3], 2);
        Rank::ThreeOfAKind(count_to_value[3] << 13 | low)
    } else if count_to_value[2].count_ones() >= 2 {
        // Two pair
        //
        // That can be because we have 3 pairs and a high card.
        // Or we could have two pair and two high cards.
        let pairs = keep_n(count_to_value[2], 2);
        let low = keep_highest(value_set ^ pairs);
        Rank::TwoPair(pairs << 13 | low)
    } else if count_to_value[2] == 0 {
        // This means that there's no pair
        // no sets, no straights, no flushes, so only a
        // high card.
        Rank::HighCard(keep_n(value_set, 5))
    } else {
        // Otherwise there's only one pair.
        let pair = count_to_value[2];
        // Keep the highest three cards not in the pair.
        let low = keep_n(value_set ^ count_to_value[2], 3);
        Rank::OnePair(pair << 13 | low)
    }
}
