use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{Card, CardValue, HoleCards, InRangeType, PokerError, Suit};

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct BoolRange {
    pub data: InRangeType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

    #[inline]
    pub fn update_with_singleton(&mut self, combo: &str, enabled: bool) -> Result<(), PokerError> {
        let (rank1, rank2, suitedness) = parse_singleton(combo)?;
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
    for suit1 in Suit::suits() {
        let card1 = Card::new(rank, suit1);
        for suit2 in Suit::suits() {
            if suit1 == suit2 {
                continue;
            }
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
    result
}

#[inline]
fn suited_indices(rank1: CardValue, rank2: CardValue) -> Vec<usize> {
    let mut result = Vec::with_capacity(4);
    for suit1 in Suit::suits() {
        let card1 = Card::new(rank1, suit1);
        let card2 = Card::new(rank2, suit1);
        let hc = HoleCards::new(card1, card2).unwrap();
        result.push(hc.to_range_index());
    }
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
            let hc = HoleCards::new(card1, card2).unwrap();
            result.push(hc.to_range_index());
        }
    }
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
                let card2 = Card::new(rank1, suit2);
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
                let card2 = Card::new(rank1, suit2);
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

#[cfg(test)]
mod tests {
    use super::*;

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
