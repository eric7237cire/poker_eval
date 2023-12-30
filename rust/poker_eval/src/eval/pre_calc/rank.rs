use super::RANK_FAMILY_OFFEST;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Rank {
    pub raw_rank: u16,
}

impl Rank {
    pub fn get_rank_enum(&self) -> RankEnum {
        let rank_family = self.raw_rank >> RANK_FAMILY_OFFEST;

        let kicker_mask = (1 << RANK_FAMILY_OFFEST) - 1;

        match rank_family {
            0 => RankEnum::HighCard(self.raw_rank & kicker_mask),
            1 => RankEnum::OnePair(self.raw_rank & kicker_mask),
            2 => RankEnum::TwoPair(self.raw_rank & kicker_mask),
            3 => RankEnum::ThreeOfAKind(self.raw_rank & kicker_mask),
            4 => RankEnum::Straight(self.raw_rank & kicker_mask),
            5 => RankEnum::Flush(self.raw_rank & kicker_mask),
            6 => RankEnum::FullHouse(self.raw_rank & kicker_mask),
            7 => RankEnum::FourOfAKind(self.raw_rank & kicker_mask),
            8 => RankEnum::StraightFlush(self.raw_rank & kicker_mask),
            _ => panic!("Unknown rank family {}", rank_family),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RankEnum {
    //0
    HighCard(u16),
    OnePair(u16),
    TwoPair(u16),
    ThreeOfAKind(u16),
    Straight(u16),
    Flush(u16),
    FullHouse(u16),
    FourOfAKind(u16),
    StraightFlush(u16),
}

impl From<u16> for Rank {
    fn from(raw_rank: u16) -> Self {
        Self { raw_rank }
    }
}
