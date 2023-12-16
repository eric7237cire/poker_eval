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
    assert!(cards.len() <= 7);
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
    use log::info;
    use postflop_solver::Hand;

    use crate::{
        cards_from_string, get_possible_hole_cards, range_string_to_set, rank_cards, CardUsedType,
        Rank,
    };

    #[test]
    fn test_flop_rank() {
        let range_str = "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o";
        let range_set = range_string_to_set(range_str);

        let mut used_cards = CardUsedType::default();
        let flop = cards_from_string("Qs Ts 7c");
        let other_cards = cards_from_string("8d 7s Qd 5c");

        for card in flop.iter() {
            used_cards.set((*card).into(), true);
        }
        for card in other_cards.iter() {
            used_cards.set((*card).into(), true);
        }

        let possible = get_possible_hole_cards(&range_set, used_cards);

        assert_eq!(373, possible.len());

        let mut num_trips = 0;
        let mut num_two_pair = 0;
        let mut num_pair = 0;
        let mut num_high_card = 0;

        /*let mut over_pocket_pair = 0;
        let mut ace_high = 0;
        let mut king_high = 0;*/

        let mut eval_cards = flop.clone();

        for hole_cards in possible.iter() {
            eval_cards.push(hole_cards.0);
            eval_cards.push(hole_cards.1);

            let rank = rank_cards(&eval_cards);
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
        let range_set = range_string_to_set(range_str);

        let mut used_cards = CardUsedType::default();
        let flop = cards_from_string("Qs Ts 7c");

        let flop_hand = Hand::new();
        let flop_hand = flop_hand.add_card(flop[0].into());
        let flop_hand = flop_hand.add_card(flop[1].into());
        let flop_hand = flop_hand.add_card(flop[2].into());

        let p1_hole_cards = cards_from_string("8d 7s");
        let p2_hole_cards = cards_from_string("Qd 5c");

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

        let possible = get_possible_hole_cards(&range_set, used_cards);

        assert_eq!(373, possible.len());

        let mut win_equity = vec![0.0; 3];
        let mut tie_equity = vec![0.0; 3];

        let mut total_showdowns = 0;

        //enumerate everything
        for p in possible.iter() {
            used_cards.set(p.0.into(), true);
            used_cards.set(p.1.into(), true);
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

                    let mut p1_cards = flop.clone();
                    p1_cards.push(p1_hole_cards[0]);
                    p1_cards.push(p1_hole_cards[1]);
                    p1_cards.push(turn_card.into());
                    p1_cards.push(river_card.into());

                    let mut p2_cards = flop.clone();
                    p2_cards.push(p2_hole_cards[0]);
                    p2_cards.push(p2_hole_cards[1]);
                    p2_cards.push(turn_card.into());
                    p2_cards.push(river_card.into());

                    let mut p3_cards = flop.clone();
                    p3_cards.push(p.0);
                    p3_cards.push(p.1);
                    p3_cards.push(turn_card.into());
                    p3_cards.push(river_card.into());

                    let p1_hand = flop_hand.add_card(p1_hole_cards[0].into());
                    let p1_hand = p1_hand.add_card(p1_hole_cards[1].into());
                    let p1_hand = p1_hand.add_card(turn_card.into());
                    let p1_hand = p1_hand.add_card(river_card.into());

                    let p2_hand = flop_hand.add_card(p2_hole_cards[0].into());
                    let p2_hand = p2_hand.add_card(p2_hole_cards[1].into());
                    let p2_hand = p2_hand.add_card(turn_card.into());
                    let p2_hand = p2_hand.add_card(river_card.into());

                    let p3_hand = flop_hand.add_card(p.0.into());
                    let p3_hand = p3_hand.add_card(p.1.into());
                    let p3_hand = p3_hand.add_card(turn_card.into());
                    let p3_hand = p3_hand.add_card(river_card.into());

                    let p1_rank = rank_cards(&p1_cards);
                    let p2_rank = rank_cards(&p2_cards);
                    let p3_rank = rank_cards(&p3_cards);

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

            used_cards.set(p.0.into(), false);
            used_cards.set(p.1.into(), false);
            assert_eq!(used_cards.count_ones(), 7);
            //we used 9 cards, so there should be 43*42/2 showdowns total
            assert_eq!(total_showdowns - check_showndown, 43 * 42 / 2);
        }

        //Values for Equilab
        assert_equity(100.0 * tie_equity[0] / total_showdowns as f64, 0.12, 0.005);
        assert_equity(100.0 * win_equity[0] / total_showdowns as f64, 21.03, 0.005);

        assert_equity(100.0 * tie_equity[1] / total_showdowns as f64, 0.82, 0.005);
        assert_equity(100.0 * win_equity[1] / total_showdowns as f64, 50.93, 0.005);

        assert_equity(100.0 * tie_equity[2] / total_showdowns as f64, 0.95, 0.005);
        assert_equity(100.0 * win_equity[2] / total_showdowns as f64, 26.14, 0.005);
    }
}
