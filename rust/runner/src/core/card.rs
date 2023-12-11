use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::mem;
use std::str::Chars;

// Adapted from https://crates.io/crates/rs-poker

/// Card rank or value.
/// This is basically the face value - 2
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy, Hash)]
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
    /// # Examples
    ///
    /// ```
    /// use rs_poker::core::Value;
    /// assert_eq!(Value::Four, Value::from_u8(Value::Four as u8));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use rs_poker::core::Value;
    ///
    /// assert_eq!(Value::Ace, Value::from_char('A').unwrap());
    /// ```
    pub fn from_char(c: char) -> Option<Self> {
        Self::try_from(c).ok()
    }

    /// Convert this Value to a char.
    pub fn to_char(self) -> char {
        char::from(self)
    }

    /// How card ranks seperate the two values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rs_poker::core::Value;
    /// assert_eq!(1, Value::Ace.gap(Value::King));
    /// ```
    pub fn gap(self, other: Self) -> u8 {
        let min = cmp::min(self as u8, other as u8);
        let max = cmp::max(self as u8, other as u8);
        max - min
    }
}

impl From<u8> for CardValue {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute(cmp::min(value, Self::Ace as u8)) }
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
            _ => panic!("Unsupported char")
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
    /// # Examples
    ///
    /// ```
    /// use rs_poker::core::Suit;
    /// let suits = Suit::suits();
    /// assert_eq!(4, suits.len());
    /// ```
    pub const fn suits() -> [Self; 4] {
        SUITS
    }

    /// Translate a Suit from a u8. If the u8 is above the expected value
    /// then Diamond will be the result.
    ///
    /// #Examples
    /// ```
    /// use rs_poker::core::Suit;
    /// let idx = Suit::Club as u8;
    /// assert_eq!(Suit::Club, Suit::from_u8(idx));
    /// ```
    pub fn from_u8(s: u8) -> Self {
        Self::from(s)
    }

    /// Given a character that represents a suit try and parse that char.
    /// If the char can represent a suit return it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rs_poker::core::Suit;
    ///
    /// let s = Suit::from_char('s');
    /// assert_eq!(Some(Suit::Spade), s);
    /// ```
    ///
    /// ```
    /// use rs_poker::core::Suit;
    ///
    /// let s = Suit::from_char('X');
    /// assert_eq!(None, s);
    /// ```
    pub fn from_char(s: char) -> Option<Self> {
        TryFrom::try_from(s).ok()
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
            _ => panic!("Unsupported char")
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
            suit: suit_char.into()
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


#[cfg(test)]
mod tests {
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
}
