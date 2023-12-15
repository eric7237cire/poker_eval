use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::mem;
use bitvec::prelude::*;
use postflop_solver::Range;
use postflop_solver::card_pair_to_index;

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
    
    pub fn from_u8(v: u8) -> Self {
        Self::from(v)
    }
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
            CardValue::from(next)
        }
    }

    pub fn prev_card(self) -> Self {
        if self == CardValue::Two {
            CardValue::Ace
        } else {
            CardValue::from(self as u8 - 1)
        }
    }
}

impl From<u8> for CardValue {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(cmp::min(value, Self::Ace as u8)) }
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

impl From<char> for CardValue {
    fn from(value: char) -> Self {
        match value.to_ascii_uppercase() {
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
            _ => panic!("Unsupported char"),
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

/// Enum for the four different suits.
/// While this has support for ordering it's not
/// sensical. The sorting is only there to allow sorting cards.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Suit {
    /// Spades
    Spade = 0,
    /// Clubs
    Club = 1,
    /// Hearts
    Heart = 2,
    /// Diamonds
    Diamond = 3,
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

    /// Translate a Suit from a u8. If the u8 is above the expected value
    /// then Diamond will be the result.
    ///
    
    pub fn from_u8(s: u8) -> Self {
        Self::from(s)
    }

    /// This Suit to a character.
    pub fn to_char(self) -> char {
        char::from(self)
    }
}

impl From<u8> for Suit {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(cmp::min(value, Self::Diamond as u8)) }
    }
}

impl From<char> for Suit {
    fn from(value: char) -> Self {
        match value.to_ascii_lowercase() {
            'd' => Self::Diamond,
            's' => Self::Spade,
            'h' => Self::Heart,
            'c' => Self::Club,
            _ => panic!("Unsupported char"),
        }
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

    pub fn from_range_index_part(index: usize) -> Self {
        let value = index >> 2;
        let suit = index & 0x3;
        Self {
            value: CardValue::from(value as u8),
            suit: Suit::from(suit as u8),
        }
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

impl From<&str> for Card {
    fn from(value: &str) -> Self {
        let mut chars = value.chars();
        let value_char = chars.next().unwrap();
        let suit_char = chars.next().unwrap();
        Self {
            value: value_char.into(),
            suit: suit_char.into(),
        }
    }
}

//Card as in postflop-solver
pub fn card_to_eval_card(card: Card) -> u8 {
    //Use values from poker_evaluate
    let suit = match card.suit {
        Suit::Spade => 3,
        Suit::Heart => 2,
        Suit::Diamond => 1,
        Suit::Club => 0,
    };
    let value = card.value as u8;

    (value << 2) | suit
}

pub fn eval_card_to_card(card: u8) -> Card {

    let suit = card & 0x3;
    let value = card >> 2;

    //Use values from poker_evaluate
    let suit = match suit {
        3 =>Suit::Spade,
        2 => Suit::Heart,
        1 => Suit::Diamond,
        0 => Suit::Club,
        _ => panic!("Invalid suit"),
    };
    Card {
        value: CardValue::from(value),
        suit,
    }
}

pub fn cards_from_string(a_string: &str) -> Vec<Card> {
    let mut cards = Vec::with_capacity(5);
    let mut chars = a_string.chars().filter(|c| !c.is_whitespace());
    while let Some(c) = chars.next() {
        let value = c;
        let suit = chars.next().unwrap();
        cards.push(Card::new(value.into(), suit.into()));
    }
    cards
}

pub fn add_cards_from_string(cards: &mut Vec<Card>, a_string: &str) -> ()  {
    let mut chars = a_string.chars().filter(|c| !c.is_whitespace());
    while let Some(c) = chars.next() {
        let value = c;
        let suit = chars.next().unwrap();
        cards.push(Card::new(value.into(), suit.into()));
    }
}

pub fn core_cards_to_range_index(card1: Card, card2: Card) -> usize {
    let card1_index = card1.to_range_index_part();
    let card2_index = card2.to_range_index_part();

    return card1_index * 52 + card2_index;
}

pub type InRangeType = BitArr!(for 52*52, in u64, Lsb0);

pub type CardUsedType = BitArr!(for 52, in u64, Lsb0);

pub fn range_string_to_set(range_str: &str) -> InRangeType {
    let range: Range = range_str.parse().unwrap();
    let mut set = InRangeType::default();

    for card1 in 0..52 {
        let core_card1 = eval_card_to_card(card1);
        
        for card2 in card1 + 1..52 {
            let core_card2 = eval_card_to_card(card2);

            let range_index = card_pair_to_index(card1, card2);

            let in_range = range.data[range_index] > 0.0;

            
            set.set(core_cards_to_range_index(core_card1, core_card2) , in_range);
            set.set(core_cards_to_range_index(core_card2, core_card1), in_range);

            
        }
    }

    set
}

// pub fn get_random_unused_card(cards_used: &CardUsedType) -> Card {
//     let num = rand::thread_rng().gen_range(0..52);
// }

#[cfg(test)]
mod tests {
    use postflop_solver::{card_from_str};

    

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
                assert_eq!(e, Card::try_from(card_string.as_str()).unwrap());
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
        assert!(c3 > c2);
    }

    #[test]
    fn test_value_cmp() {
        assert!(CardValue::Two < CardValue::Ace);
        assert!(CardValue::King < CardValue::Ace);
        assert_eq!(CardValue::Two, CardValue::Two);
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(CardValue::Two, CardValue::from_u8(0));
        assert_eq!(CardValue::Ace, CardValue::from_u8(12));
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

        let card = eval_card_to_card(ps_solver_card);

        assert_eq!(card, Card::try_from("7c").unwrap());
        assert_eq!(card.suit, Suit::Club);
        assert_eq!(card.value, CardValue::Seven);
        assert_eq!(card_to_eval_card(card), ps_solver_card);


        let ps_solver_card = card_from_str("Ad").unwrap();

        let card = eval_card_to_card(ps_solver_card);

        assert_eq!(card, Card::try_from("Ad").unwrap());
        assert_eq!(card.suit, Suit::Diamond);
        assert_eq!(card.value, CardValue::Ace);
        assert_eq!(card_to_eval_card(card), ps_solver_card);
        
        let ps_solver_card = card_from_str("2h").unwrap();
        let card = eval_card_to_card(ps_solver_card);

        assert_eq!(card, Card::try_from("2h").unwrap());
        assert_eq!(card.suit, Suit::Heart);
        assert_eq!(card.value, CardValue::Two);

        assert_eq!(card_to_eval_card(card), ps_solver_card);
    }
    
    #[test]
    fn test_range_to_set() {
        
        let range_str = "Q4o+";
        //let range: Range = Range::from_sanitized_str(rangeStr).unwrap();

        let set = range_string_to_set(range_str);     

        assert!(!set[core_cards_to_range_index(Card::from("Qs"), Card::from("3h"))]);
        assert!(set[core_cards_to_range_index(Card::from("Qs"), Card::from("4h"))]);
        assert!(set[core_cards_to_range_index(Card::from("5s"), Card::from("Qc"))]);
        assert!(!set[core_cards_to_range_index(Card::from("Qs"), Card::from("Qc"))]);
        assert!(!set[core_cards_to_range_index(Card::from("Qs"), Card::from("Kc"))]);
        assert!(set[core_cards_to_range_index(Card::from("Js"), Card::from("Qc"))]);

        let range_str = "22+";
        //let range: Range = Range::from_sanitized_str(rangeStr).unwrap();

        let set = range_string_to_set(range_str); 

        assert_eq!(set.count_ones(), 13*2*6);

        let range_str = "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32";

        let set = range_string_to_set(range_str); 

        assert_eq!(set.count_ones(), 52*51);
    }
}

