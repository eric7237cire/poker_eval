use crate::{CardValue, calc_cards_metrics, Card, calc_bitset_cards_metrics, count_higher};

//for pairs, 2 pair, sets, quads, full houses
#[derive(PartialEq, Eq, Debug)]
pub struct PairFamilyRank {
    pub number_above: u8,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
pub enum FlushDrawType {
     BackdoorFlushDraw,
        FlushDraw,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
pub enum StraightDrawType {
    GutShotDraw,
    OpenEndedDraw,
}

//We'll parse a list of these
// pub enum PartialRank {
//     FlushDraw(FlushDraw),
//     StraightDraw(StraightDraw),
//     PockerPair(PocketPair),
//     Pair(Pair),
//     TwoOverCards(TwoOverCards),
// }

#[derive(PartialEq, Eq, Debug)]
pub struct FlushDraw {
    pub hole_card_value: CardValue,
    pub flush_draw_type: FlushDrawType,
}

#[derive(PartialEq, Eq, Debug)]
pub struct StraightDraw {
    pub straight_draw_type: StraightDrawType,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PocketPair {
    pub number_above: u8,
    pub made_set: bool,
    pub made_quads: bool,
}

//Tracking basically what our hole cards are doing
//Meant to be combined with rank to make decisions
pub struct PartialRankContainer {
    flush_draw: Option<FlushDraw>,
    straight_draw: Option<StraightDraw>,
    
    pocket_pair: Option<PocketPair>,
    pair: Option<PairFamilyRank>,
    two_pair: Option<PairFamilyRank>,
    //set: Option<PairFamilyRank>,
    //Don't track full house because it's really a set with a pair on the board
    //full_house: Option<PairFamilyRank>,
    //quads: Option<PairFamilyRank>,

    unpaired_higher_card: Option<PairFamilyRank>,
    unpaired_lower_card: Option<PairFamilyRank>,
}

impl Default for PartialRankContainer {
    fn default() -> Self {
        PartialRankContainer {
            flush_draw: None,
            straight_draw: None,
            pocket_pair: None,
            pair: None,
            two_pair: None,

            unpaired_higher_card: None,
            unpaired_lower_card: None,

        }
    }
} 
  
pub fn partial_rank_cards(hole_cards: &[Card], board: &[Card]) -> PartialRankContainer {
    let mut partial_ranks: PartialRankContainer = Default::default();

    let board_metrics = calc_bitset_cards_metrics(board);

    assert_eq!(2, hole_cards.len());

    if hole_cards[0].value == hole_cards[1].value {
        //let number_above = board_metrics.value_set.iter_ones().filter(|&v| v > hole_cards[0].value as usize).count() as u8;
        let made_set = board_metrics.value_to_count[hole_cards[0].value as usize] == 1;
        let made_quads = board_metrics.value_to_count[hole_cards[0].value as usize] == 2;

        //for quads we need to know how many pairs or trips on the board have a higher value
        let number_above = if made_quads  {
            0
        }
         else {
            count_higher(board_metrics.count_to_value[0], hole_cards[0].value.into())
         } + count_higher(board_metrics.count_to_value[2], hole_cards[0].value.into()) + 
         count_higher(board_metrics.count_to_value[3], hole_cards[0].value.into());
            
        partial_ranks.pocket_pair = Some(PocketPair{number_above, made_set, made_quads});

        
    }


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
        assert_eq!(prc.pocket_pair, Some(PocketPair{number_above: 0, made_quads: false, made_set: true}));
        //we don't count pairs on the board
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);        
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("2c 2h");
        let board_cards = cards_from_string("3c 2s As 3d Ac");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(prc.straight_draw, None);
        assert_eq!(prc.pocket_pair, Some(PocketPair{number_above: 2, made_quads: false, made_set: true}));
        //we don't count pairs on the board
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("7c 7h");
        let board_cards = cards_from_string("3c 7s Ks 7d Ac");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(prc.straight_draw, None);
        assert_eq!(prc.pocket_pair, Some(PocketPair{number_above: 0, made_quads: true, made_set: false}));
        //we don't count pairs on the board
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

    }
}