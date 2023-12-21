use crate::HoleCards;
use bitvec::prelude::*;
use postflop_solver::card_pair_to_index;
use postflop_solver::Range;
use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::mem;
use std::str::FromStr;

use crate::PokerError;

//use bitvec::BitArr;
// Adapted from https://crates.io/crates/rs-poker

/// Card rank or value.
/// This is basically the face value - 2
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum CardValue {
    /// 2
    Two = 0,
    /// 3
    Three = 1,
    /// 4
    Four = 2,
    /// 5
    Five = 3,
    /// 6
    Six = 4,
    /// 7
    Seven = 5,
    /// 8
    Eight = 6,
    /// 9
    Nine = 7,
    /// T
    Ten = 8,
    /// J
    Jack = 9,
    /// Q
    Queen = 10,
    /// K
    King = 11,
    /// A
    Ace = 12,
}

/// Constant of all the values.
/// This is what `Value::values()` returns
const VALUES: [CardValue; 13] = [
    CardValue::Two,
    CardValue::Three,
    CardValue::Four,
    CardValue::Five,
    CardValue::Six,
    CardValue::Seven,
    CardValue::Eight,
    CardValue::Nine,
    CardValue::Ten,
    CardValue::Jack,
    CardValue::Queen,
    CardValue::King,
    CardValue::Ace,
];

impl CardValue {
    /// Take a u32 and convert it to a value.
    ///

    // pub fn from_u8(v: u8) -> Self {
    //     Self::from(v)
    // }
    /// Get all of the `Value`'s that are possible.
    /// This is used to iterate through all possible
    /// values when creating a new deck, or
    /// generating all possible starting hands.
    pub const fn values() -> [Self; 13] {
        VALUES
    }

    /// Given a character parse that char into a value.
    /// Case is ignored as long as the char is in the ascii range (It should
    /// be).
    ///
    /// @returns None if there's no valid value there. Otherwise a Value enum
    ///

    pub fn from_char(c: char) -> Option<Self> {
        Self::try_from(c).ok()
    }

    /// Convert this Value to a char.
    pub fn to_char(self) -> char {
        //info!("to_char: {:?}", self);
        char::from(self)
    }

    /// How card ranks seperate the two values.
    ///

    pub fn gap(self, other: Self) -> u8 {
        let min = cmp::min(self as u8, other as u8);
        let max = cmp::max(self as u8, other as u8);
        max - min
    }

    pub fn next_card(self) -> Self {
        let next = self as u8 + 1;
        if next > 12 {
            CardValue::Two
        } else {
            CardValue::try_from(next).unwrap()
        }
    }

    pub fn prev_card(self) -> Self {
        if self == CardValue::Two {
            CardValue::Ace
        } else {
            CardValue::try_from(self as u8 - 1).unwrap()
        }
    }
}

impl TryFrom<u8> for CardValue {
    type Error = PokerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 12 {
            return Err(PokerError::from_string(format!("Invalid value: {}", value)));
        }

        Ok(unsafe { mem::transmute(value) })
    }
}

impl From<usize> for CardValue {
    fn from(value: usize) -> Self {
        VALUES[value]
    }
}

impl Into<usize> for CardValue {
    fn into(self) -> usize {
        self as usize
    }
}

impl TryFrom<char> for CardValue {
    type Error = PokerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value.to_ascii_uppercase() {
            'A' => Self::Ace,
            'K' => Self::King,
            'Q' => Self::Queen,
            'J' => Self::Jack,
            'T' => Self::Ten,
            '9' => Self::Nine,
            '8' => Self::Eight,
            '7' => Self::Seven,
            '6' => Self::Six,
            '5' => Self::Five,
            '4' => Self::Four,
            '3' => Self::Three,
            '2' => Self::Two,
            c => return Err(PokerError::from_string(format!("Unsupported char {}", c))),
        })
    }
}

impl FromStr for CardValue {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let value_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        match value_char.to_ascii_uppercase() {
            'A' => Ok(Self::Ace),
            'K' => Ok(Self::King),
            'Q' => Ok(Self::Queen),
            'J' => Ok(Self::Jack),
            'T' => Ok(Self::Ten),
            '9' => Ok(Self::Nine),
            '8' => Ok(Self::Eight),
            '7' => Ok(Self::Seven),
            '6' => Ok(Self::Six),
            '5' => Ok(Self::Five),
            '4' => Ok(Self::Four),
            '3' => Ok(Self::Three),
            '2' => Ok(Self::Two),
            c => Err(PokerError::from_string(format!("Unsupported char: {}", c))),
        }
    }
}

impl From<CardValue> for char {
    fn from(value: CardValue) -> Self {
        match value {
            CardValue::Ace => 'A',
            CardValue::King => 'K',
            CardValue::Queen => 'Q',
            CardValue::Jack => 'J',
            CardValue::Ten => 'T',
            CardValue::Nine => '9',
            CardValue::Eight => '8',
            CardValue::Seven => '7',
            CardValue::Six => '6',
            CardValue::Five => '5',
            CardValue::Four => '4',
            CardValue::Three => '3',
            CardValue::Two => '2',
        }
    }
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

/// Enum for the four different suits.
/// While this has support for ordering it's not
/// sensical. The sorting is only there to allow sorting cards.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Suit {
    /// Spades
    Spade = 3,
    /// Clubs
    Club = 0,
    /// Hearts
    Heart = 2,
    /// Diamonds
    Diamond = 1,
}

/// All of the `Suit`'s. This is what `Suit::suits()` returns.
const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];

/// Impl of Suit
///
/// This is just here to provide a list of all `Suit`'s.
impl Suit {
    /// Provide all the Suit's that there are.
    ///
    pub const fn suits() -> [Self; 4] {
        SUITS
    }

    /// This Suit to a character.
    pub fn to_char(self) -> char {
        char::from(self)
    }
}

impl TryFrom<u8> for Suit {
    type Error = PokerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 3 {
            return Err(PokerError::from_string(format!("Invalid value: {}", value)));
        }

        Ok(unsafe { mem::transmute(value) })
    }
}

impl TryFrom<char> for Suit {
    type Error = PokerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value.to_ascii_lowercase() {
            'd' => Self::Diamond,
            's' => Self::Spade,
            'h' => Self::Heart,
            'c' => Self::Club,
            c => return Err(PokerError::from_string(format!("Unsupported char {}", c))),
        })
    }
}

impl From<Suit> for char {
    fn from(value: Suit) -> Self {
        match value {
            Suit::Diamond => 'd',
            Suit::Spade => 's',
            Suit::Heart => 'h',
            Suit::Club => 'c',
        }
    }
}

impl FromStr for Suit {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let suit_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        match suit_char.to_ascii_lowercase() {
            'd' => Ok(Self::Diamond),
            's' => Ok(Self::Spade),
            'h' => Ok(Self::Heart),
            'c' => Ok(Self::Club),
            c => Err(PokerError::from_string(format!("Unsupported char: {}", c))),
        }
    }
}

/// The main struct of this library.
/// This is a carrier for Suit and Value combined.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub struct Card {
    /// The face value of this card.
    pub value: CardValue,
    /// The suit of this card.
    pub suit: Suit,
}

impl Card {
    pub fn new(value: CardValue, suit: Suit) -> Self {
        Self { value, suit }
    }

    pub fn to_range_index_part(&self) -> usize {
        let value = self.value as usize;
        assert!(value < 13);
        let suit = self.suit as usize;
        assert!(suit < 4);
        let ret = (value << 2) + suit;
        assert!(ret < 52);
        ret
    }

    pub fn from_range_index_part(index: usize) -> Result<Self, PokerError> {
        let value = index >> 2;
        let suit = index & 0x3;
        Ok(Self {
            value: CardValue::try_from(value as u8)?,
            suit: Suit::try_from(suit as u8)?,
        })
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Card({}{})",
            char::from(self.value),
            char::from(self.suit)
        )
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", char::from(self.value), char::from(self.suit))
    }
}

// impl From<&str> for Card {
//     fn from(value: &str) -> Self {
//         let mut chars = value.chars();
//         let value_char = chars.next().unwrap();
//         let suit_char = chars.next().unwrap();
//         Self {
//             value: value_char.into(),
//             suit: suit_char.into(),
//         }
//     }
// }

impl FromStr for Card {
    type Err = PokerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let value_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        let suit_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        Ok(Self {
            value: value_char.to_string().parse()?,
            suit: suit_char.to_string().parse()?,
        })
    }
}

impl TryFrom<&str> for Card {
    type Error = PokerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut chars = value.chars();
        let value_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        let suit_char = chars.next().ok_or(PokerError::from_str("No character"))?;
        Ok(Self {
            value: value_char.to_string().parse()?,
            suit: suit_char.to_string().parse()?,
        })
    }
}

impl Into<u8> for Card {
    fn into(self) -> u8 {
        (self.value as u8) << 2 | self.suit as u8
    }
}

impl Into<usize> for Card {
    fn into(self) -> usize {
        (self.value as usize) << 2 | self.suit as usize
    }
}

impl TryFrom<u8> for Card {
    type Error = PokerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self {
            value: CardValue::try_from(value >> 2)?,
            suit: Suit::try_from(value & 0x3)?,
        })
    }
}

impl TryFrom<usize> for Card {
    type Error = PokerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self {
            value: CardValue::try_from((value >> 2) as u8)?,
            suit: Suit::try_from((value & 0x3) as u8)?,
        })
    }
}

pub struct CardVec(pub Vec<Card>);

impl TryFrom<&str> for CardVec {
    type Error = PokerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut cards = Vec::with_capacity(7);
        let mut chars = value.chars().filter(|c| c.is_alphanumeric());
        while let Some(c) = chars.next() {
            let value = c;
            let suit = chars.next().ok_or(PokerError::from_string(format!(
                "Unable to parse suit from {}",
                value
            )))?;
            cards.push(Card::new(value.try_into()?, suit.try_into()?));
        }
        Ok(CardVec(cards))
    }
}

impl CardVec {
    pub fn as_vec_u8(&self) -> Vec<u8> {

        self.0.iter().map(|c| {
            let c_u8 : u8 = (*c).into();
            c_u8
    }).collect()
    }
}

// pub fn cards_from_string(a_string: &str) -> Result<Vec<Card>, PokerError> {
//     let mut cards = Vec::with_capacity(7);
//     let mut chars = a_string.chars().filter(|c| c.is_alphanumeric());
//     while let Some(c) = chars.next() {
//         let value = c;
//         let suit = chars.next().ok_or(PokerError::from_string(format!(
//             "Unable to parse suit from {}",
//             a_string
//         )))?;
//         cards.push(Card::new(value.try_into()?, suit.try_into()?));
//     }
//     Ok(cards)
// }

pub fn add_cards_from_string(cards: &mut Vec<Card>, a_string: &str) -> () {
    let mut chars = a_string.chars().filter(|c| !c.is_whitespace());
    while let Some(c) = chars.next() {
        let value = c;
        let suit = chars.next().unwrap();
        cards.push(Card::new(
            value.try_into().unwrap(),
            suit.try_into().unwrap(),
        ));
    }
}

//52 * 51 / 2
pub type InRangeType = BitArr!(for 1326, in usize, Lsb0);

pub type CardUsedType = BitArr!(for 52, in usize, Lsb0);

pub fn range_string_to_set(range_str: &str) -> Result<InRangeType, PokerError> {
    let range: Range = range_str
        .parse()
        .ok()
        .ok_or(PokerError::from_string(format!(
            "Unable to parse range {}",
            range_str
        )))?;
    let mut set = InRangeType::default();

    for card1 in 0..52 {
        //let core_card1 = card1.into();

        for card2 in card1 + 1..52 {
            //let core_card2 = card2.into();

            let range_index = card_pair_to_index(card1, card2);

            let in_range = range.data[range_index] > 0.0;

            set.set(range_index, in_range);
        }
    }

    Ok(set)
}

//returns in range / total
pub fn get_possible_hole_cards_count(
    range_set: &InRangeType,
    used_card_set: CardUsedType,
) -> (u16, u16) {
    //let range: Range = range_str.parse().unwrap();
    //let mut vec = Vec::new();
    let mut in_range_count = 0;
    let mut total = 0;

    for card1 in 0..52 {
        if used_card_set[card1] {
            continue;
        }

        for card2 in card1 + 1..52 {
            //let core_card2 = card2.into();

            if used_card_set[card2] {
                continue;
            }

            total += 1;

            let range_index = card_pair_to_index(card1 as u8, card2 as u8);

            let in_range = range_set[range_index];

            if in_range {
                //vec.push((card1, card2));
                in_range_count += 1;
            }
        }
    }

    (in_range_count, total)
}

pub fn get_possible_hole_cards(
    range_set: &InRangeType,
    used_card_set: CardUsedType,
) -> Result<Vec<HoleCards>, PokerError> {
    //let range: Range = range_str.parse().unwrap();
    let mut vec = Vec::new();

    for card1 in 0..52 {
        if used_card_set[card1] {
            continue;
        }

        for card2 in card1 + 1..52 {
            //let core_card2 = card2.into();

            if used_card_set[card2] {
                continue;
            }

            let range_index = card_pair_to_index(card1 as u8, card2 as u8);

            let in_range = range_set[range_index];

            if in_range {
                vec.push(HoleCards::new(
                    Card::try_from(card1)?,
                    Card::try_from(card2)?,
                )?);
            }
        }
    }

    Ok(vec)
}

pub fn get_filtered_range_set(range_set: &InRangeType, used_card_set: CardUsedType) -> InRangeType {
    //let range: Range = range_str.parse().unwrap();
    let mut filtered_range_set = range_set.clone();

    for card1 in 0..52 {
        for card2 in card1 + 1..52 {
            //let core_card2 = card2.into();

            let range_index = card_pair_to_index(card1 as u8, card2 as u8);

            if !range_set[range_index] {
                continue;
            }

            if !used_card_set[card2] && !used_card_set[card1] {
                continue;
            }

            filtered_range_set.set(range_index, false);
        }
    }

    filtered_range_set
}

// pub fn get_random_unused_card(cards_used: &CardUsedType) -> Card {
//     let num = rand::thread_rng().gen_range(0..52);
// }

#[cfg(test)]
mod tests {
    use postflop_solver::{card_from_str, index_to_card_pair};

    use super::*;
    use std::mem;

    #[test]
    fn test_constructor() {
        let c = Card {
            value: CardValue::Three,
            suit: Suit::Spade,
        };
        assert_eq!(Suit::Spade, c.suit);
        assert_eq!(CardValue::Three, c.value);
    }

    #[test]
    fn test_try_parse_card() {
        let expected = Card {
            value: CardValue::King,
            suit: Suit::Spade,
        };

        assert_eq!(expected, Card::try_from("Ks").unwrap())
    }

    #[test]
    fn test_parse_all_cards() {
        for suit in SUITS {
            for value in VALUES {
                let e = Card { suit, value };
                let card_string = format!("{}{}", char::from(value), char::from(suit));
                let card: Card = card_string.parse().unwrap();
                assert_eq!(e, card);
            }
        }
    }

    #[test]
    fn test_compare() {
        let c1 = Card {
            value: CardValue::Three,
            suit: Suit::Spade,
        };
        let c2 = Card {
            value: CardValue::Four,
            suit: Suit::Spade,
        };
        let c3 = Card {
            value: CardValue::Four,
            suit: Suit::Club,
        };

        // Make sure that the values are ordered
        assert!(c1 < c2);
        assert!(c2 > c1);
        // Make sure that suit is used.
        assert!(c3 < c2);
    }

    #[test]
    fn test_value_cmp() {
        assert!(CardValue::Two < CardValue::Ace);
        assert!(CardValue::King < CardValue::Ace);
        assert_eq!(CardValue::Two, CardValue::Two);
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(CardValue::Two, CardValue::try_from(0u8).unwrap());
        assert_eq!(CardValue::Three, CardValue::try_from(1u8).unwrap());
        assert_eq!(CardValue::King, CardValue::try_from(11u8).unwrap());
        assert_eq!(CardValue::Ace, CardValue::try_from(12u8).unwrap());

        assert!(CardValue::try_from(13u8).is_err());
    }

    #[test]
    fn test_size_card() {
        // Card should be really small. Hopefully just two u8's
        assert!(mem::size_of::<Card>() <= 2);
    }

    #[test]
    fn test_size_suit() {
        // One byte for Suit
        assert!(mem::size_of::<Suit>() <= 1);
    }

    #[test]
    fn test_size_value() {
        // One byte for Value
        assert!(mem::size_of::<CardValue>() <= 1);
    }

    #[test]
    fn test_gap() {
        // test on gap
        assert!(1 == CardValue::Ace.gap(CardValue::King));
        // test no gap at the high end
        assert!(0 == CardValue::Ace.gap(CardValue::Ace));
        // test no gap at the low end
        assert!(0 == CardValue::Two.gap(CardValue::Two));
        // Test one gap at the low end
        assert!(1 == CardValue::Two.gap(CardValue::Three));
        // test that ordering doesn't matter
        assert!(1 == CardValue::Three.gap(CardValue::Two));
        // Test things that are far apart
        assert!(12 == CardValue::Ace.gap(CardValue::Two));
        assert!(12 == CardValue::Two.gap(CardValue::Ace));
    }

    #[test]
    fn test_conversions() {
        let ps_solver_card = card_from_str("7c").unwrap();

        let card = Card::try_from("7c").unwrap();

        assert_eq!(ps_solver_card, card.into());

        assert_eq!(card.suit, Suit::Club);
        assert_eq!(card.value, CardValue::Seven);

        let ps_solver_card = card_from_str("Ad").unwrap();
        let card = Card::try_from("Ad").unwrap();

        assert_eq!(card.suit, Suit::Diamond);
        assert_eq!(card.value, CardValue::Ace);

        assert_eq!(ps_solver_card, card.into());

        let ps_solver_card = card_from_str("2h").unwrap();
        let card = Card::try_from("2h").unwrap();

        assert_eq!(card.suit, Suit::Heart);
        assert_eq!(card.value, CardValue::Two);

        assert_eq!(ps_solver_card, card.into());
    }

    #[test]
    fn test_range_to_set() {
        let range_str = "Q4o+";
        //let range: Range = Range::from_sanitized_str(rangeStr).unwrap();

        let set = range_string_to_set(range_str).unwrap();

        let hc: HoleCards = "Qs 3h".parse().unwrap();
        assert!(!set[hc.to_range_index()]);

        let hc: HoleCards = "Qs 4h".parse().unwrap();
        assert!(set[hc.to_range_index()]);

        let hc: HoleCards = "Qs 5h".parse().unwrap();
        assert!(set[hc.to_range_index()]);

        let hc: HoleCards = "Qs Qc".parse().unwrap();
        assert!(!set[hc.to_range_index()]);

        let hc: HoleCards = "Qs Kc".parse().unwrap();
        assert!(!set[hc.to_range_index()]);

        let hc: HoleCards = "Js Qc".parse().unwrap();
        assert!(set[hc.to_range_index()]);

        let range_str = "22+";
        //let range: Range = Range::from_sanitized_str(rangeStr).unwrap();

        let set = range_string_to_set(range_str).unwrap();

        assert_eq!(set.count_ones(), 13 * 6);

        let range_str = "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32";

        let set = range_string_to_set(range_str).unwrap();

        assert_eq!(set.count_ones(), 52 * 51 / 2);
    }

    #[test]
    fn test_card_pair_to_index() {
        for card1 in 0..52 {
            for card2 in card1 + 1..52 {
                let index = card_pair_to_index(card1, card2);
                let (c1, c2) = index_to_card_pair(index);
                assert_eq!(card1, c1);
                assert_eq!(card2, c2);
            }
        }
    }

    #[test]
    fn test_get_possible_hole_cards() {
        let range_str = "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o";
        let range_set = range_string_to_set(range_str).unwrap();

        let mut used_cards = CardUsedType::default();
        let cards = CardVec::try_from("8d 7s Qd 5c Qs Ts 7c").unwrap().0;

        for card in cards.iter() {
            used_cards.set((*card).into(), true);
        }

        let (in_range, total) = get_possible_hole_cards_count(&range_set, used_cards);

        assert_eq!(990, total);
        assert_eq!(373, in_range);

        let dbg_fs = get_filtered_range_set(&range_set, used_cards);
        assert_eq!(373, dbg_fs.count_ones());

        assert_eq!(11029519011840, used_cards.data[0]);
    }
}
