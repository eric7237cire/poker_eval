use crate::CardValue;



#[repr(u8)]
pub enum PairType {
    Top,
    Second,
    ThirdOrWorse
}


#[repr(u8)]
pub enum PocketPairType {
    Overpair,
    OneBelow, // KK with A on the board
    TwoBelow,
    ThirdOrWorseBelow,
}

#[repr(u8)]
pub enum FlushDrawType {
    HoleCardValue(CardValue),
}

#[repr(u8)]
pub enum StraightDrawType {
    GutShotDraw
}

//We'll parse a list of these
pub enum PartialRank {
    FlushDraw(FlushDrawType),
    Draw(StraightDrawType),
    PockerPair(PocketPairType),
    Pair(PairType),
    TwoOverCards,
}

pub fn partial_rank_cards(hole_cards: &(CardValue, CardValue), board: &[CardValue]) -> Vec<PartialRank> {
    let mut partial_ranks: Vec<PartialRank> = Vec::new();

    partial_ranks
}