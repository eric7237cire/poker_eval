use crate::{calc_bitset_cards_metrics, calc_cards_metrics, count_higher, Card, CardValue};

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

    //If we hit our straight, how many better straights exist on the board
    pub number_above: u8,
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

    //Handle pocket pairs
    if hole_cards[0].value == hole_cards[1].value {
        //let number_above = board_metrics.value_set.iter_ones().filter(|&v| v > hole_cards[0].value as usize).count() as u8;
        let made_set = board_metrics.value_to_count[hole_cards[0].value as usize] == 1;
        let made_quads = board_metrics.value_to_count[hole_cards[0].value as usize] == 2;

        //for quads we need to know how many pairs or trips on the board have a higher value
        let number_above =
            if made_quads {
                0
            } else {
                count_higher(board_metrics.count_to_value[1], hole_cards[0].value.into())
            } + count_higher(board_metrics.count_to_value[2], hole_cards[0].value.into())
                + count_higher(board_metrics.count_to_value[3], hole_cards[0].value.into());

        partial_ranks.pocket_pair = Some(PocketPair {
            number_above,
            made_set,
            made_quads,
        });
    }

    //draws

    //To know how good our draw is
    //take the board and calculate all the possible straight draws and gut shot draws

    //an open ended draw is when we have a block of 4 consecutive values
    //with a lower and higher one possible
    let mut open_ended_draws: Vec<CardValue> = Vec::new();

    //So we do a rolling count of 1s in a row on the value set
    //Note though there are some cases
    // [0] 1 1 1 1 [0] is the normal one
    // [0] 1 1 1 1 [1] means there are no open ended draws, since we have a straight
    // [1] 1 1 1 1 [0] means there are no open ended draws, since we have a straight
    // But we do have a gut shot draw to a higher straight in the 3rd case

    // The value of the draw is the highest value in the draw, so for
    // [0] 1 1 1 (1) ([0]) is the 2 values we add are in ()

    //First one as either 2 3 4 5
    let mut rolling_one_count = board_metrics.value_set[CardValue::Two as usize..=CardValue::Five as usize].count_ones();

    for range_end in CardValue::Six as usize..=CardValue::Ace as usize {
        // [0] 1 1 1 1 [range_end] 

        //the wheel
        let range_begin = if range_end == CardValue::Six as usize {
            CardValue::Ace as usize
        } else {
            range_end - 5
        };

        assert!(rolling_one_count <= 4);

        //Basically we need to use at least one max 2 of the hole cards
        //So if the board already has the entire block set, there is no draw here
        if rolling_one_count >= 2 && rolling_one_count < 4 && board_metrics.value_set[range_end] && board_metrics.value_set[range_begin] {
            open_ended_draws.push(range_end.into());
        } 

        rolling_one_count += if board_metrics.value_set[range_end] {1} else {0};
        rolling_one_count -= if board_metrics.value_set[range_begin] {1} else {0};
    }

    let mut gut_shot_draws: Vec<CardValue> = Vec::new();
    
    //A gut shot is to a specific draw, so we look at all blocks of 5
    //and if the board has at least 2, but <= 3 (since we need a hole card to make it a draw)
    //If we had 4 on the board, then the 5th card would make a straight on the board without hole cards

    //start with wheel
    let mut rolling_one_count = board_metrics.value_set[CardValue::Two as usize..=CardValue::Five as usize].count_ones();
    rolling_one_count += if board_metrics.value_set[CardValue::Ace as usize] {1} else {0};

    if rolling_one_count >= 2 && rolling_one_count < 4 {
        gut_shot_draws.push(CardValue::Five);
    } 

    //take off the ace 
    rolling_one_count -= if board_metrics.value_set[CardValue::Ace as usize] {1} else {0};

    //Add the 6
    rolling_one_count += if board_metrics.value_set[CardValue::Six as usize] {1} else {0};

    for range_end in CardValue::Six as usize..=CardValue::Ace as usize {
        //range_end is already counted
        let range_begin = range_end - 5;

        assert!(rolling_one_count <= 5);

        //Basically we need to use at least one max 2 of the hole cards
        //So if the board already has the entire block set, there is no draw here
        if rolling_one_count >= 2 && rolling_one_count < 4  {
            gut_shot_draws.push(range_end.into());
        } 

        rolling_one_count += if board_metrics.value_set[range_end] {1} else {0};
        rolling_one_count -= if board_metrics.value_set[range_begin] {1} else {0};
    }


    //Ok, now we need to figure out, do *we* have a draw



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
        assert_eq!(
            prc.pocket_pair,
            Some(PocketPair {
                number_above: 0,
                made_quads: false,
                made_set: true
            })
        );
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
        assert_eq!(
            prc.pocket_pair,
            Some(PocketPair {
                number_above: 2,
                made_quads: false,
                made_set: true
            })
        );
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
        assert_eq!(
            prc.pocket_pair,
            Some(PocketPair {
                number_above: 0,
                made_quads: true,
                made_set: false
            })
        );
        //we don't count pairs on the board
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("Ac 2h");
        let board_cards = cards_from_string("3c 7s 5s Td Ac");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::GutShotDraw,
                //Draw to 2 3 4 5 6
                //Draw to 3 4 5 6 7
                //Draw to 4 [5] 6 [7] 8
                number_above: 3
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("2c 6h");
        let board_cards = cards_from_string("3c 4s 7d Ac");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::GutShotDraw,
                //include made straights too, 5 6
                //other better gut shots made with 5 8, 6 8, 2 5, 2 6 
                //4 5 6 7 8
                //3 4 5 6 7
                //2 3 4 5 6
                number_above: 3
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        //Not a straight draw ?  hmmm
        let hole_cards = cards_from_string("7c 8h");
        let board_cards = cards_from_string("Ah Ts Kd Qc Jd");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            None
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("7c 8h");
        let board_cards = cards_from_string("2c Ts Kd Qc Jd");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::GutShotDraw,
                //A K Q J T
                //K Q J T 9
                //Q J T 9 8
                //J T 9 8 7
                number_above: 3
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("Kc Jh");
        let board_cards = cards_from_string("Ts Qc 8d");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::OpenEndedDraw,
                //there is one better gut shot, but we only count better open ended draws
                number_above: 0
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("6c 8h");
        let board_cards = cards_from_string("7s 9c 2d");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::OpenEndedDraw,
                //we are drawing to 6 7 8 9 T
                //but have an open ended draw 8 T that is better
                number_above: 1
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0 }));
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);
    }
}
