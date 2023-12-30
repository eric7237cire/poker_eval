use std::str::FromStr;

use itertools::Itertools;
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use bitvec::prelude::*;
use crate::{Card, CardValue, CardValueRange, HoleCards, PokerError, Suit, pre_calc::{NUMBER_OF_SUITS, NUMBER_OF_HOLE_CARDS}};

//52 * 51 / 2
pub type InRangeType = BitArr!(for NUMBER_OF_HOLE_CARDS, in usize, Lsb0);

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct BoolRange {
    pub data: InRangeType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Suitedness {
    Suited,
    Offsuit,
    All,
    Specific(Suit, Suit),
}

impl BoolRange {
    pub fn new() -> Self {
        BoolRange {
            data: InRangeType::default(),
        }
    }

    pub fn new_with_data(data: InRangeType) -> Self {
        BoolRange { data }
    }

    #[inline]
    pub fn set_enabled(&mut self, indices: &[usize], enabled: bool) {
        for &i in indices {
            self.data.set(i, enabled);
        }
    }

    fn is_enabled_for_indices(&self, indices: &[usize]) -> bool {
        indices.iter().all(|&i| self.data[i])
    }

    fn is_enabled_for_holecards(&self, hc: &HoleCards) -> bool {
        self.data[hc.to_range_index()]
    }

    #[inline]
    pub fn update_with_singleton(&mut self, combo: &str, enabled: bool) -> Result<(), PokerError> {
        let (rank1, rank2, suitedness) = parse_singleton(combo)?;
        trace!("update_with_singleton rank1: {}, rank2: {}, suitedness: {:?}", rank1, rank2, suitedness);
        self.set_enabled(&indices_with_suitedness(rank1, rank2, suitedness), enabled);
        Ok(())
    }

    #[inline]
    fn update_with_plus_range(&mut self, range: &str, enabled: bool) -> Result<(), PokerError> {
        let lowest_combo = &range[..range.len() - 1];
        let (rank1, rank2, suitedness) = parse_singleton(lowest_combo)?;
        assert!(rank1 >= rank2);

        let gap = (rank1 as u8) - (rank2 as u8);
        if gap <= 1 {
            let rank1_u8 = rank1 as u8;
            // pair and connector (e.g.,  88+, T9s+)
            for i in rank1_u8..13 {
                let r1: CardValue = i.try_into().unwrap();
                let r2: CardValue = (i - gap).try_into().unwrap();
                self.set_enabled(&indices_with_suitedness(r1, r2, suitedness), enabled);
            }
        } else {
            // otherwise (e.g., ATo+)
            for i in (rank2 as u8)..(rank1 as u8) {
                let r2: CardValue = i.try_into().unwrap();
                self.set_enabled(&indices_with_suitedness(rank1, r2, suitedness), enabled);
            }
        }
        Ok(())
    }

    #[inline]
    fn update_with_dash_range(&mut self, range: &str, enabled: bool) -> Result<(), PokerError> {
        let combo_pair = range.split('-').collect::<Vec<_>>();
        let (rank11, rank12, suitedness) = parse_singleton(combo_pair[0])?;
        let (rank21, rank22, suitedness2) = parse_singleton(combo_pair[1])?;
        let gap = (rank11 as u8) - (rank12 as u8);
        let gap2 = (rank21 as u8) - (rank22 as u8);
        if suitedness != suitedness2 {
            Err(format!("Suitedness does not match: {range}").into())
        } else if gap == gap2 {
            // same gap (e.g., 88-55, KQo-JTo)
            if rank11 > rank21 {
                for i in (rank21 as u8)..=(rank11 as u8) {
                    let r1: CardValue = i.try_into().unwrap();
                    let r2: CardValue = (i - gap).try_into().unwrap();
                    self.set_enabled(&indices_with_suitedness(r1, r2, suitedness), enabled);
                }
                Ok(())
            } else {
                Err(format!("Range must be in descending order: {range}").into())
            }
        } else if rank11 == rank21 {
            // same first rank (e.g., A5s-A2s)
            if rank12 > rank22 {
                for i in (rank22 as u8)..=(rank12 as u8) {
                    let r2 = i.try_into().unwrap();
                    self.set_enabled(&indices_with_suitedness(rank11, r2, suitedness), enabled);
                }
                Ok(())
            } else {
                Err(format!("Range must be in descending order: {range}").into())
            }
        } else {
            Err(format!("Invalid range: {range}").into())
        }
    }

    fn pairs_strings(&self, result: &mut Vec<String>) {
        
        let is_enabled_vec = CardValueRange::new(CardValue::Two, CardValue::Ace).rev().map(
            |rank| {
                (self.is_enabled_for_indices(&pair_indices(rank)), rank)
            },
        ).collect_vec();

        let mut start_index = 0;
        while start_index < is_enabled_vec.len() {
        //for (start_index, (is_enabled, start_rank2)) in is_enabled_vec.iter().enumerate() {
            
            assert!(result.len() < 2000);

            let (is_enabled, start_rank) = is_enabled_vec[start_index];
            if !is_enabled {
                start_index += 1;
                continue;
            }

            //Find the next disabled index
            let stop_index = is_enabled_vec.iter().skip(start_index + 1).position(|(is_enabled, _)| !is_enabled).map_or(                
                is_enabled_vec.len(), |idx| idx + start_index + 1,);

            assert!(stop_index > start_index);

            if start_index == 0 && stop_index > start_index + 1 {
                let stop_rank = is_enabled_vec[stop_index-1].1;
                assert!(stop_rank < start_rank);
                result.push(format!("{stop_rank}{stop_rank}+"));
                start_index = stop_index + 1;
                continue;
                //result.push(format!("{start_rank}{start_rank}+"));
                
            }
            if stop_index == start_index + 1 {
                result.push(format!("{start_rank}{start_rank}"));
                start_index += 1;
                continue;
            }
            let stop_rank = is_enabled_vec[stop_index-1].1;
            start_index = stop_index + 1;
            
            //convention is 55-33; note order is decreasing, so start_rank > stop_rank
            assert!(start_rank > stop_rank);
            result.push(format!("{start_rank}{start_rank}-{stop_rank}{stop_rank}"));

        }
    }

    
    fn nonpairs_strings(&self, result: &mut Vec<String>) {
        for rank1 in CardValueRange::new(CardValue::Three, CardValue::Ace).rev() {
            if self.can_unsuit(rank1) {
                self.high_cards_strings(result, rank1, Suitedness::All);
            } else {
                self.high_cards_strings(result, rank1, Suitedness::Suited);
                self.high_cards_strings(result, rank1, Suitedness::Offsuit);
            }
        }
    }

    fn suit_specified_strings(&self, result: &mut Vec<String>) {
        // pairs
        for rank in CardValueRange::new(CardValue::Two, CardValue::Ace).rev() {
            if !self.is_enabled_for_indices(&pair_indices(rank)) {
                for suit1_int in (0..4).rev() {
                    let suit1: Suit = suit1_int.try_into().unwrap();
                    for suit2_int in (0..suit1_int).rev() {
                        let suit2: Suit = suit2_int.try_into().unwrap();
                        let hc = HoleCards::new(Card::new(rank, suit1), Card::new(rank, suit2))
                            .unwrap();
                        if !self.is_enabled_for_holecards(&hc) {
                            continue;
                        }
                        let tmp = format!(
                            "{rank}{suit1}{rank}{suit2}",
                        );
                        result.push(tmp);
                        
                    }
                }
            }
        }

        // non-pairs
        for rank1 in CardValueRange::new(CardValue::Three, CardValue::Ace).rev() {
        
            for rank2 in CardValueRange::new(CardValue::Two, rank1.prev_card()).rev() {

                assert!(rank1 > rank2);
                if !self.is_enabled_for_indices(&suited_indices(rank1, rank2)) {
                    for suit_int in (0..4).rev() {
                        let suit : Suit = suit_int.try_into().unwrap();
                        let hc = HoleCards::new(Card::new(rank1, suit), Card::new(rank2, suit))
                            .unwrap();
                        if !self.is_enabled_for_holecards(&hc) {
                            continue;
                        }
                        result.push( format!(
                            "{rank1}{suit}{rank2}{suit}",
                        ));
                    }
                }

                // offsuit
                if !self.is_enabled_for_indices(&offsuit_indices(rank1, rank2)) {
                    for suit1_int in (0..4).rev() {
                        let suit1 : Suit = suit1_int.try_into().unwrap();
                        for suit2_int in (0..4).rev() {
                            let suit2 : Suit = suit2_int.try_into().unwrap();
                            if suit1 == suit2 {
                                continue;
                            }
                            
                            let hc = HoleCards::new(Card::new(rank1, suit1), Card::new(rank2, suit2)).unwrap();
                            if !self.is_enabled_for_holecards(&hc) {
                                continue;
                            }
                            result.push(format!(
                                        "{rank1}{suit1}{rank2}{suit2}",
                                    ));
                                
                            
                        }
                    }
                }
            }
        }
    }

    fn high_cards_strings(&self, result: &mut Vec<String>, rank1: CardValue, suitedness: Suitedness) {

        /*
        Generates like A2o+  A3-A5
        */

        type FnPairToIndices = fn(CardValue, CardValue) -> Vec<usize>;
        let (getter, suit_char): (FnPairToIndices, &str) = match suitedness {
            Suitedness::Suited => (suited_indices, "s"),
            Suitedness::Offsuit => (offsuit_indices, "o"),
            Suitedness::All => (nonpair_indices, ""),
            _ => panic!("high_cards_strings: invalid suitedness"),
        };

        // rank1 is the higher rank
        let is_enabled_vec = CardValueRange::new(CardValue::Two, rank1.prev_card()).rev().map(|rank2| {
            (self.is_enabled_for_indices(&getter(rank1, rank2)), rank2)
        }).collect_vec();


        let mut start_index = 0;
        while start_index < is_enabled_vec.len() {
        //for (start_index, (is_enabled, start_rank2)) in is_enabled_vec.iter().enumerate() {
            
            assert!(result.len() < 2000);

            let (is_enabled, start_rank2) = is_enabled_vec[start_index];
            if !is_enabled {
                start_index += 1;
                continue;
            }

            //Find the next disabled index
            let stop_index = is_enabled_vec.iter().skip(start_index + 1).position(|(is_enabled, _)| !is_enabled).map_or(                
                is_enabled_vec.len(), |idx| idx + start_index + 1,);

            assert!(stop_index > start_index);

            //is_enabled is decreasing
            if start_index == 0 && stop_index > start_index + 1 {
                let stop_rank2 = is_enabled_vec[stop_index-1].1;
                assert!(stop_rank2 < start_rank2);
                result.push(format!("{rank1}{stop_rank2}{suit_char}+"));
                start_index = stop_index + 1;
                continue;
            }
            if stop_index == start_index + 1 {
                result.push(format!("{rank1}{start_rank2}{suit_char}"));
                start_index += 1;
                continue;
            }
            let stop_rank2 = is_enabled_vec[stop_index-1].1;
            start_index = stop_index + 1;
            
            //convention is larger one first; so K8s-K5s
            result.push(format!("{rank1}{start_rank2}{suit_char}-{rank1}{stop_rank2}{suit_char}"));
            
        }
    }

    fn can_unsuit(&self, rank1: CardValue) -> bool {
        //Basically if we have something before it that has to be seperated, we can't unsuit
        //if this were true, we would create a string like A2o+  A4-A3 
        for rank2 in CardValueRange::new(CardValue::Two, rank1.prev_card()) {
            let has_suited = self.is_enabled_for_indices(&suited_indices(rank1, rank2));
            let has_offsuit = self.is_enabled_for_indices(&offsuit_indices(rank1, rank2));

            if (has_suited || has_offsuit) && (has_offsuit != has_suited) {
                return false;
            }
        }
        true
    }
}
const COMBO_PAT: &str = r"(?:(?:[AaKkQqJjTt2-9]{2}[os]?)|(?:(?:[AaKkQqJjTt2-9][cdhs]){2}))";
static TRIM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*([-:,])\s*").unwrap());
static RANGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"^(?P<range>{COMBO_PAT}(?:\+|(?:-{COMBO_PAT}))?)$"
    ))
    .unwrap()
});

// #[inline]
// pub fn card_pair_to_index(mut card1: usize, mut card2: usize) -> usize {
//     assert!(card1 < 52);
//     assert!(card2 < 52);
//     assert!(card1 != card2);
//     if card1 > card2 {
//         mem::swap(&mut card1, &mut card2);
//     }
//     //card2 > card1
//     card1 as usize * (101 - card1 as usize) / 2 + card2 as usize - 1
// }

//Use holecard -- to_range_index
#[inline]
fn pair_indices(rank: CardValue) -> Vec<usize> {
    //let rank: usize = rank_obj.try_into().unwrap();
    let mut result = Vec::with_capacity(6);
    for suit1_index in 0..NUMBER_OF_SUITS {
        let suit1: Suit = Suit::suits()[suit1_index];
        let card1 = Card::new(rank, suit1);
        for suit2_index in suit1_index+1..NUMBER_OF_SUITS {
            let suit2: Suit = Suit::suits()[suit2_index];
            let card2 = Card::new(rank, suit2);
            let hc = HoleCards::new(card1, card2).unwrap();
            result.push(hc.to_range_index());
        }
    }
    result
}

#[inline]
fn nonpair_indices(rank1: CardValue, rank2: CardValue) -> Vec<usize> {
    let mut result = Vec::with_capacity(16);
    for suit1 in Suit::suits() {
        let card1 = Card::new(rank1, suit1);
        for suit2 in Suit::suits() {
            let card2 = Card::new(rank2, suit2);
            let hc = HoleCards::new(card1, card2).unwrap();
            result.push(hc.to_range_index());
        }
    }
    assert_eq!(16, result.len());
    result
}

#[inline]
fn suited_indices(rank1: CardValue, rank2: CardValue) -> Vec<usize> {
    let mut result = Vec::with_capacity(4);
    for suit1 in Suit::suits() {
        let card1 = Card::new(rank1, suit1);
        let card2 = Card::new(rank2, suit1);
        if card1 == card2 {
            continue;
        }
        let hc = HoleCards::new(card1, card2).unwrap();
        result.push(hc.to_range_index());
    }
    assert_eq!(4, result.len());
    result
}

#[inline]
fn offsuit_indices(rank1: CardValue, rank2: CardValue) -> Vec<usize> {
    let mut result = Vec::with_capacity(12);
    for suit1 in Suit::suits() {
        let card1 = Card::new(rank1, suit1);
        for suit2 in Suit::suits() {
            if suit1 == suit2 {
                continue;
            }
            let card2 = Card::new(rank2, suit2);

            if card1 == card2 {
                continue;
            }

            let hc = HoleCards::new(card1, card2).unwrap();
            result.push(hc.to_range_index());
        }
    }

    assert_eq!(12, result.len());
    result
}

#[inline]
fn indices_with_suitedness(
    rank1: CardValue,
    rank2: CardValue,
    suitedness: Suitedness,
) -> Vec<usize> {
    if rank1 == rank2 {
        match suitedness {
            Suitedness::All => pair_indices(rank1),
            Suitedness::Specific(suit1, suit2) => {
                let card1 = Card::new(rank1, suit1);
                let card2 = Card::new(rank2, suit2);
                let hc = HoleCards::new(card1, card2).unwrap();
                vec![hc.to_range_index()]
            }
            _ => panic!("invalid suitedness with a pair"),
        }
    } else {
        match suitedness {
            Suitedness::Suited => suited_indices(rank1, rank2),
            Suitedness::Offsuit => offsuit_indices(rank1, rank2),
            Suitedness::All => nonpair_indices(rank1, rank2),
            Suitedness::Specific(suit1, suit2) => {
                let card1 = Card::new(rank1, suit1);
                let card2 = Card::new(rank2, suit2);
                // if card1 == card2 {
                //     return vec![];
                // }
                let hc = HoleCards::new(card1, card2).unwrap();
                vec![hc.to_range_index()]
            }
        }
    }
}

#[inline]
fn parse_singleton(combo: &str) -> Result<(CardValue, CardValue, Suitedness), PokerError> {
    if combo.len() == 4 {
        parse_simple_singleton(combo)
    } else {
        parse_compound_singleton(combo)
    }
}

#[inline]
fn parse_simple_singleton(combo: &str) -> Result<(CardValue, CardValue, Suitedness), PokerError> {
    let mut chars = combo.chars();
    let rank1 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    let suit1 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    let rank2 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    let suit2 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    if rank1 < rank2 {
        return Err(format!(
            "The first rank must be equal or higher than the second rank: {combo}"
        )
        .into());
    }
    if rank1 == rank2 && suit1 == suit2 {
        return Err(format!("Duplicate cards are not allowed: {combo}").into());
    }
    Ok((rank1, rank2, Suitedness::Specific(suit1, suit2)))
}

#[inline]
fn parse_compound_singleton(combo: &str) -> Result<(CardValue, CardValue, Suitedness), PokerError> {
    let mut chars = combo.chars();
    let rank1 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    let rank2 = chars
        .next()
        .ok_or_else(|| "Unexpected end".to_string())?
        .try_into()?;
    let suitedness = chars.next().map_or(Ok(Suitedness::All), |c| match c {
        's' => Ok(Suitedness::Suited),
        'o' => Ok(Suitedness::Offsuit),
        _ => Err(format!("Invalid suitedness: {combo}")),
    })?;
    if rank1 < rank2 {
        return Err(format!(
            "The first rank must be equal or higher than the second rank: {combo}"
        )
        .into());
    }
    if rank1 == rank2 && suitedness != Suitedness::All {
        return Err(format!("A pair with suitedness is not allowed: {combo}").into());
    }
    Ok((rank1, rank2, suitedness))
}

impl FromStr for BoolRange {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = TRIM_REGEX.replace_all(s, "$1").trim().to_string();
        let mut ranges = s.split(',').collect::<Vec<_>>();

        // remove last empty element if any
        if ranges.last().unwrap().is_empty() {
            ranges.pop();
        }

        let mut result = Self::new();

        for range in ranges.into_iter().rev() {
            let caps = RANGE_REGEX
                .captures(range)
                .ok_or_else(|| format!("Failed to parse range: {range}"))?;

            let range = caps.name("range").unwrap().as_str();

            trace!("range: {}", range);

            if range.contains('-') {
                result.update_with_dash_range(range, true)?;
            } else if range.contains('+') {
                result.update_with_plus_range(range, true)?;
            } else {
                result.update_with_singleton(range, true)?;
            }
        }

        Ok(result)
    }
}

impl ToString for BoolRange {
    #[inline]
    fn to_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();
        self.pairs_strings(&mut result);
        self.nonpairs_strings(&mut result);
        self.suit_specified_strings(&mut result);
        
        result.join(",")
    }
}

#[cfg(test)]
mod tests {
    use log::{debug};
    
    use rand::{rngs::StdRng, SeedableRng, seq::SliceRandom};

    use crate::{pre_calc::NUMBER_OF_HOLE_CARDS, init_test_logger};

    use super::*;

    //Pretty slow
    //#[test]
    #[allow(dead_code)]
    fn test_range_to_string() {

        //cargo test --lib test_range_to_string --release -- --nocapture

        init_test_logger();
        let mut rng = StdRng::seed_from_u64(42);

        //initialize an array usize from 0 to NUMBER_OF_HOLE_CARDS
        let mut indices = (0..NUMBER_OF_HOLE_CARDS).collect::<Vec<usize>>();

        for num_set_in_range in 0..=NUMBER_OF_HOLE_CARDS {
            debug!("Doing 10 iterations with {} cards in the range", num_set_in_range);
            //do 10 random subsets of that size
            for _ in 0..10 {
                //shuffle the indices
                indices.shuffle(&mut rng);

                //take the first num_set_in_range indices
                let indices = &indices[..num_set_in_range];

                //create a range from those indices
                let mut range = BoolRange::new();
                range.set_enabled(indices, true);

                assert_eq!(range.data.count_ones(), num_set_in_range );

                //convert the range to a string
                let range_string = range.to_string();

                // let mut check_rng: Range = Range::new();
                // for idx in 0..NUMBER_OF_HOLE_CARDS {
                //     if range.data[idx]  {
                //         check_rng.data[idx] = 1.0;
                //     }
                // }
                // assert_eq!(check_rng.to_string(), range_string);
                
                //convert the string back to a range
                let range2 = range_string.parse::<BoolRange>().unwrap();

                // if range.data.count_ones() != range2.data.count_ones() {
                //     info!("range: {}", range.to_string());     
                //     let mut check_rng: Range = Range::new();
                //     for card1_int in 0..52u8 {
                //         let card1 = card1_int.try_into().unwrap();
                //         for card2_int in card1_int+1..52 {
                //             let card2 = card2_int.try_into().unwrap();
                //             let hc = HoleCards::new(card1, card2).unwrap();
                //             if range.is_enabled_for_holecards(&hc) {
                //                 check_rng.data[hc.to_range_index()]=1.0;
                //             }
                //             if range.is_enabled_for_holecards(&hc) != range2.is_enabled_for_holecards(&hc) {
                //                 info!("card1: {}, card2: {}  enabled originally? {}", card1, card2, range.is_enabled_for_holecards(&hc));
                //             }
                //         }
                //     }      
                    
                //     info!("chk range: {}", check_rng.to_string());
                // }

                //check that the two ranges are equal
                assert_eq!(range.data.count_ones(), range2.data.count_ones());
                assert_eq!(range, range2);
            }
        }
    }

    #[test]
    fn test_range_single_pair_to_string() {

        //cargo test --lib test_range_to_string --release -- --nocapture

        init_test_logger();
        
        let hc1: HoleCards = "Ac Ad".parse().unwrap();
        let hc2: HoleCards = "Ac Ah".parse().unwrap();
        let hc3: HoleCards = "Ac As".parse().unwrap();
        let hc4: HoleCards = "Ad Ah".parse().unwrap();
        let hc5: HoleCards = "Ad As".parse().unwrap();
        let hc6: HoleCards = "Ah As".parse().unwrap();

        let mut range = BoolRange::new();
        let indices = [hc1.to_range_index(), hc2.to_range_index(), hc3.to_range_index(), hc4.to_range_index(), hc5.to_range_index(), hc6.to_range_index()];
        range.set_enabled(&indices, true);

        assert_eq!(6, range.data.count_ones() );

        assert_eq!(6, pair_indices(CardValue::Ace).len());

        //convert the range to a string
        let range_string = range.to_string();
        
        assert_eq!(range_string, "AA");
    }

    #[test]
    fn range_from_str() {
        let pair_plus = "88+".parse::<BoolRange>();
        let pair_plus_equiv = "AA,KK,QQ,JJ,TT,99,88".parse::<BoolRange>();
        assert!(pair_plus.is_ok());
        assert_eq!(pair_plus, pair_plus_equiv);

        let pair_plus_suit = "8s8h+".parse::<BoolRange>();
        let pair_plus_suit_equiv = "AhAs,KhKs,QhQs,JhJs,ThTs,9h9s,8h8s".parse::<BoolRange>();
        assert!(pair_plus_suit.is_ok());
        assert_eq!(pair_plus_suit, pair_plus_suit_equiv);

        let connector_plus = "98s+".parse::<BoolRange>();
        let connector_plus_equiv = "AKs,KQs,QJs,JTs,T9s,98s".parse::<BoolRange>();
        assert!(connector_plus.is_ok());
        assert_eq!(connector_plus, connector_plus_equiv);

        let other_plus = "A8o+".parse::<BoolRange>();
        let other_plus_equiv = "AKo,AQo,AJo,ATo,A9o,A8o".parse::<BoolRange>();
        assert!(other_plus.is_ok());
        assert_eq!(other_plus, other_plus_equiv);

        let pair_dash = "88-55".parse::<BoolRange>();
        let pair_dash_equiv = "88,77,66,55".parse::<BoolRange>();
        assert!(pair_dash.is_ok());
        assert_eq!(pair_dash, pair_dash_equiv);

        let connector_dash = "98s-65s".parse::<BoolRange>();
        let connector_dash_equiv = "98s,87s,76s,65s".parse::<BoolRange>();
        assert!(connector_dash.is_ok());
        assert_eq!(connector_dash, connector_dash_equiv);

        let gapper_dash = "AQo-86o".parse::<BoolRange>();
        let gapper_dash_equiv = "AQo,KJo,QTo,J9o,T8o,97o,86o".parse::<BoolRange>();
        assert!(gapper_dash.is_ok());
        assert_eq!(gapper_dash, gapper_dash_equiv);

        let other_dash = "K5-K2".parse::<BoolRange>();
        let other_dash_equiv = "K5,K4,K3,K2".parse::<BoolRange>();
        assert!(other_dash.is_ok());
        assert_eq!(other_dash, other_dash_equiv);

        let suit_compound = "AhAs-QhQs,JJ".parse::<BoolRange>();
        let suit_compound_equiv = "JJ,AhAs,KhKs,QhQs".parse::<BoolRange>();
        assert!(suit_compound.is_ok());
        assert_eq!(suit_compound, suit_compound_equiv);

        let allow_empty = "".parse::<BoolRange>();
        assert!(allow_empty.is_ok());

        let allow_trailing_comma = "AK,".parse::<BoolRange>();
        assert!(allow_trailing_comma.is_ok());

        let comma_error = "AK,,".parse::<BoolRange>();
        assert!(comma_error.is_err());

        let rank_error = "89".parse::<BoolRange>();
        assert!(rank_error.is_err());

        let pair_error = "AAo".parse::<BoolRange>();
        assert!(pair_error.is_err());

        let weight_error = "AQo:1.1".parse::<BoolRange>();
        assert!(weight_error.is_err());

        let dash_error_1 = "AQo-AQo".parse::<BoolRange>();
        assert!(dash_error_1.is_err());

        let dash_error_2 = "AQo-86s".parse::<BoolRange>();
        assert!(dash_error_2.is_err());

        let dash_error_3 = "AQo-KQo".parse::<BoolRange>();
        assert!(dash_error_3.is_err());

        let dash_error_4 = "K2-K5".parse::<BoolRange>();
        assert!(dash_error_4.is_err());

        let dash_error_5 = "AhAs-QsQh".parse::<BoolRange>();
        assert!(dash_error_5.is_err());
    }
}
