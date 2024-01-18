use std::fmt::{Display, Formatter};

use super::RANK_FAMILY_OFFEST;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Rank {
    pub raw_rank: u16,
}

impl Rank {
    pub fn get_rank_enum(&self) -> RankEnum {
        let rank_family = self.raw_rank >> RANK_FAMILY_OFFEST;

        match rank_family {
            0 => RankEnum::HighCard,
            1 => RankEnum::OnePair,
            2 => RankEnum::TwoPair,
            3 => RankEnum::ThreeOfAKind,
            4 => RankEnum::Straight,
            5 => RankEnum::Flush,
            6 => RankEnum::FullHouse,
            7 => RankEnum::FourOfAKind,
            8 => RankEnum::StraightFlush,
            _ => panic!("Unknown rank family {}", rank_family),
        }
    }

    //The rest of the bits
    pub fn get_kicker(&self) -> u16 {
        let kicker_mask = (1 << RANK_FAMILY_OFFEST) - 1;

        self.raw_rank & kicker_mask
    }

    pub fn lowest_rank() -> Self {
        Self { raw_rank: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum RankEnum {
    //0
    HighCard = 0,
    OnePair = 1,
    TwoPair = 2,
    ThreeOfAKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    StraightFlush = 8,
}

impl From<u16> for Rank {
    fn from(raw_rank: u16) -> Self {
        Self { raw_rank }
    }
}

impl Display for RankEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RankEnum::HighCard => "High Card",
            RankEnum::OnePair => "One Pair",
            RankEnum::TwoPair => "Two Pair",
            RankEnum::ThreeOfAKind => "Trips",
            RankEnum::Straight => "Straight",
            RankEnum::Flush => "Flush",
            RankEnum::FullHouse => "Full House",
            RankEnum::FourOfAKind => "Quads",
            RankEnum::StraightFlush => "Straight Flush",
        };

        write!(f, "{}", s)
    }
}
