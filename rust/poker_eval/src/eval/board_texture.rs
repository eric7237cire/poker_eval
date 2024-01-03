//Flop Texture: has

// 3 same suit
// 2 same suit, 1 other
// 0, 1 trips
// 0, 1 pair
// Ordering ranks, what are the gaps

//'Partial' rank / draws

// Flush draws
// 2 board, 2 hand
// 3 board, 1 hand

// Straight draws

// hole connected draw -- high
// 8 9 on board 6 7

// hole connected draw -- low
// 8 9 on board T J

// 1gap hole interleaved
// 8 T on board 9 J -- med   targetting 789TJ or 89TJQ
// 8 T on board 7 9 -- low

// 2gap Hole
// 8 J on board 9 T

// Top pair, 2nd pair, 3rd pair, 4th pair etc.

// Best 2 pair, 2nd best, etc.

use std::cmp::max;

use log::trace;
use serde::{Deserialize, Serialize};
// use rmps crate to serialize structs using the MessagePack format
use crate::{
    calc_cards_metrics, partial_rank_cards, rank_cards, Board, Card, CardValue, HoleCards,
    StraightDrawType,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardTexture {
    // Highest same suited count, 1 is a raindbow board
    pub same_suited_max_count: u8,

    pub gaps: Vec<u8>, // a flop will have 2

    //56 on board gives straight draw to
    //43 and 78 and 4 7
    //5 7 gives straight draw to 46, 68
    //5 8 gives straight draw to 67
    pub num_with_str8: u16,
    pub num_with_str8_draw: u16,
    pub num_with_gut_shot: u16,
    pub num_hole_cards: u16,

    //these are all mutually exclusive
    pub has_quads: bool,
    pub has_fh: bool,
    pub has_trips: bool,
    pub has_pair: bool,
    pub has_two_pair: bool,

    //group values into 3 chunks
    //[A K Q J] [T 9 8 7] [6 5 4 3 2]
    pub high_value_count: u8,
    pub med_value_count: u8,
    pub low_value_count: u8,
}

pub fn calc_board_texture(cards: &[Card]) -> BoardTexture {
    let mut texture = BoardTexture {
        same_suited_max_count: 0,
        gaps: Vec::with_capacity(cards.len() - 1),
        has_trips: false,
        has_pair: false,
        has_two_pair: false,
        has_fh: false,
        has_quads: false,
        high_value_count: 0,
        med_value_count: 0,
        low_value_count: 0,
        num_with_str8: 0,
        num_with_str8_draw: 0,
        num_with_gut_shot: 0,
        num_hole_cards: 0,
    };

    let cards_metrics = calc_cards_metrics(cards.iter());

    let mut card_values: Vec<CardValue> = cards.iter().map(|c| c.value).collect();
    //highest value 1st
    card_values.sort_by(|a, b| b.cmp(a));

    //Gap is the difference between the values of the cards
    for i in 1..cards.len() {
        texture.gaps.push(card_values[i].gap(card_values[i - 1]));
    }

    //If highest card is an Ace then also add the gap between it and the lowest card value
    if card_values[0] == CardValue::Ace {
        //2 is == 0 so the distance is the lowest value + 1
        texture.gaps.push(card_values[cards.len() - 1] as u8 + 1);
    }

    //filter out 0 gaps, these don't matter for straights, then return lowest order first

    //T 9 -- 1
    //T 8 -- 2
    //T 7 -- 3  // T [9 8] 7
    //T 6 -- 4 // T [9 8 7] 6

    //The lowest gap distance we care about is 4

    texture.gaps.retain(|&x| x > 0 && x <= 4);
    texture.gaps.sort_by(|a, b| a.cmp(b));

    for card_value in card_values.iter() {
        if *card_value as u8 >= CardValue::Jack as u8 {
            texture.high_value_count += 1;
        } else if *card_value as u8 >= CardValue::Seven as u8 {
            texture.med_value_count += 1;
        } else {
            texture.low_value_count += 1;
        }
    }

    // Find out if there's a flush
    for svs in cards_metrics.suit_value_sets.iter() {
        texture.same_suited_max_count = max(texture.same_suited_max_count, svs.count_ones() as u8);
    }

    let pair_count = cards_metrics.count_to_value[2].count_ones();
    if cards_metrics.count_to_value[4] != 0 {
        texture.has_quads = true;
    } else if cards_metrics.count_to_value[3] != 0 {
        if pair_count == 0 {
            texture.has_trips = true;
        } else {
            texture.has_fh = true;
        }
    } else if pair_count >= 2 {
        texture.has_two_pair = true;
    } else if pair_count == 1 {
        texture.has_pair = true;
    }

    //Calculate expensive fields
    let mut eval_cards = cards.to_vec();
    let mut total_eval = 0;
    let mut num_str8 = 0;
    let mut num_str8_draw = 0;
    let mut num_gut_shot = 0;
    for hole_card1_u8 in 0..52u8 {
        let hole_card1: Card = hole_card1_u8.try_into().unwrap();
        if cards.contains(&hole_card1) {
            continue;
        }
        for hole_card2_u8 in hole_card1_u8 + 1..52u8 {
            let hole_card2: Card = hole_card2_u8.try_into().unwrap();
            if cards.contains(&hole_card2) {
                continue;
            }
            let hc = HoleCards::new(hole_card1, hole_card2).unwrap();
            let prc = partial_rank_cards(&hc, cards);
            eval_cards.push(hole_card1);
            eval_cards.push(hole_card2);

            let rank = rank_cards(eval_cards.iter());

            eval_cards.pop();
            eval_cards.pop();

            total_eval += 1;

            if rank.get_family_index() == 4 {
                num_str8 += 1;
                continue;
            }
            if let Some(s) = prc.straight_draw {
                match s.straight_draw_type {
                    StraightDrawType::OpenEnded => {
                        trace!(
                            "open ended str8 with board {} and hole cards {} {}",
                            Board::new_from_cards(cards),
                            hole_card1,
                            hole_card2
                        );

                        num_str8_draw += 1;
                    }
                    StraightDrawType::GutShot(_) => {
                        // trace!("{} gut shot with board {} and hole cards {} {}",
                        // num_gut_shot,
                        // Board(cards.to_vec()), hole_card1, hole_card2);

                        num_gut_shot += 1;
                    }
                    StraightDrawType::DoubleGutShot => {
                        trace!(
                            "Double gut shot with board {} and hole cards {} {}",
                            Board::new_from_cards(cards),
                            hole_card1,
                            hole_card2
                        );

                        num_str8_draw += 1;
                    }
                }
            }
        }
    }

    texture.num_with_str8 = num_str8;
    texture.num_with_str8_draw = num_str8_draw;
    texture.num_with_gut_shot = num_gut_shot;
    texture.num_hole_cards = total_eval;

    texture
}

#[cfg(test)]
mod tests {

    use log::{debug, info};

    use crate::init_test_logger;

    use super::*;

    #[test]
    fn test_board_texture() {
        init_test_logger();
        info!("test_board_texture");
        debug!("test_board_texture");
        trace!("test_board_texture");
        let cards = Board::try_from("3c 2s As")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 2);
        assert_eq!(texture.gaps.len(), 2);
        assert_eq!(texture.gaps[0], 1);
        assert_eq!(texture.gaps[1], 1);
        assert_eq!(texture.has_trips, false);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 1);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 2);
        assert_eq!(texture.num_hole_cards, 1176); // 49*48/2
        assert_eq!(texture.num_with_str8, 16);
        assert_eq!(texture.num_with_str8_draw, 0);
        assert_eq!(texture.num_with_gut_shot, 340);

        let cards = Board::try_from("Ac Ah As")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 1);
        assert_eq!(texture.gaps.len(), 0);
        assert_eq!(texture.has_trips, true);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 3);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 0);
        assert_eq!(texture.num_hole_cards, 1176); // 49*48/2
        assert_eq!(texture.num_with_str8, 0);
        assert_eq!(texture.num_with_str8_draw, 0);
        assert_eq!(texture.num_with_gut_shot, 0);

        let cards = Board::try_from("Qc Kh Qd As Ks")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 2);
        assert_eq!(texture.gaps.len(), 2);
        assert_eq!(texture.gaps[0], 1);
        assert_eq!(texture.gaps[1], 1);
        assert_eq!(texture.has_trips, false);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, true);
        assert_eq!(texture.high_value_count, 5);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 0);
        assert_eq!(texture.num_hole_cards, 1081); // 47*46/2
        assert_eq!(texture.num_with_str8, 16);
        assert_eq!(texture.num_with_str8_draw, 0);
        assert_eq!(texture.num_with_gut_shot, 324);

        let cards = Board::try_from("9c 2h Td As 6s")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 2);
        assert_eq!(texture.gaps.len(), 5);
        debug!("gaps {:?}", texture.gaps);
        trace!("Texture\n{:#?}", &texture);
        assert_eq!(texture.gaps[0], 1);
        assert_eq!(texture.gaps[1], 1);
        assert_eq!(texture.gaps[2], 3);
        assert_eq!(texture.gaps[3], 4);
        assert_eq!(texture.gaps[4], 4);
        assert_eq!(texture.has_trips, false);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 1);
        assert_eq!(texture.med_value_count, 2);
        assert_eq!(texture.low_value_count, 2);
        assert_eq!(texture.num_hole_cards, 1081); // 47*46/2
        assert_eq!(texture.num_with_str8, 16);
        assert_eq!(texture.num_with_str8_draw, 48);
        assert_eq!(texture.num_with_gut_shot, 372);

        let cards = Board::try_from("9c 2h Td Ks 6s")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 1081); // 47*46/2
        assert_eq!(texture.num_with_str8, 32);
        assert_eq!(texture.num_with_str8_draw, 64);
        assert_eq!(texture.num_with_gut_shot, 568);

        let cards = Board::try_from("9c 2h Td Qs 6s")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 1081); // 47*46/2
        assert_eq!(texture.num_with_str8, 48);
        assert_eq!(texture.num_with_str8_draw, 308);
        assert_eq!(texture.num_with_gut_shot, 308);

        let cards = Board::try_from("9c 2h Td Js 6s")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 1081); // 47*46/2
        assert_eq!(texture.num_with_str8, 48);
        assert_eq!(texture.num_with_str8_draw, 308);
        assert_eq!(texture.num_with_gut_shot, 308);

        let cards = Board::try_from("6c 8h Td")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 49 * 48 / 2);
        assert_eq!(texture.num_with_str8, 16);
        assert_eq!(texture.num_with_str8_draw, 64);
        assert_eq!(texture.num_with_gut_shot, 308);

        let cards = Board::try_from("6c 8h Td 9d")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 48 * 47 / 2);
        assert_eq!(texture.num_with_str8, 198);
        assert_eq!(texture.num_with_str8_draw, 166);
        assert_eq!(texture.num_with_gut_shot, 268);

        let cards = Board::try_from("Ac 2h 4d 5d")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.num_hole_cards, 48 * 47 / 2);
        assert_eq!(texture.num_with_str8, 182);
        assert_eq!(texture.num_with_str8_draw, 32);
        assert_eq!(texture.num_with_gut_shot, 150);
    }
}
