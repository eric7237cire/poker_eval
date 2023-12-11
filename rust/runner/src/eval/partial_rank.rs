use crate::{CardValue, calc_cards_metrics, Card};



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
    FlushDraw(FlushDraw),
    StraightDraw(StraightDraw),
    PockerPair(PocketPair),
    Pair(Pair),
    TwoOverCards(TwoOverCards),
}

#[derive(PartialEq, Eq)]
struct FlushDraw {
    flush_draw_type: FlushDrawType,
}

struct StraightDraw {
    straight_draw_type: StraightDrawType,
}

struct PocketPair {
    pocket_pair_type: PocketPairType,
}

struct Pair {
    pair_type: PairType,
}

struct TwoPair {
    pair_type: PairType,
}

struct TwoOverCards {

}


pub struct PartialRankContainer {
    flush_draw: Option<FlushDraw>,
    straight_draw: Option<StraightDraw>,
    pocket_pair: Option<PocketPair>,
    pair: Option<Pair>,
    two_over_cards: Option<TwoOverCards>,
}

impl Default for PartialRankContainer {
    fn default() -> Self {
        PartialRankContainer {
            flush_draw: None,
            straight_draw: None,
            pocket_pair: None,
            pair: None,
            two_over_cards: None,
        }
    }
}

pub fn partial_rank_cards(hole_cards: &[Card], board: &[Card]) -> PartialRankContainer {
    let mut partial_ranks: PartialRankContainer = Default::default();

    let board_metrics = calc_cards_metrics(board);

    partial_ranks
}



#[cfg(test)]
mod tests {

    use crate::cards_from_string;

    use super::*;

    #[test]
    fn test_partial_ranks() {
        let hole_cards = cards_from_string("Ac Ah");
        let board_cards = cards_from_string("3c 2s As");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(prc.straight_draw, None);
        assert_eq!(prc.pocket_pair, PocketPair::Overpair);

    }
}