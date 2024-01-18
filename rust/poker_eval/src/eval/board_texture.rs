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

use serde::{Deserialize, Serialize};
// use rmps crate to serialize structs using the MessagePack format
use crate::{
    calc_cards_metrics,
    pre_calc::{
        fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash, rank::Rank, NUMBER_OF_RANKS,
    },
    rank_straight, Card, CardUsedType, CardValue, Suit, ALL_HOLE_CARDS,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardTexture {
    // Highest same suited count, 1 is a raindbow board
    pub same_suited_max_count: u8,

    //This can be 2 with 4 cards HH CC for example
    pub suits_with_max_count: Vec<Suit>,

    pub gaps: Vec<u8>, // a flop will have 2

    //Goes through card value combinations to see what would make a draw or straight
    //These are empty if there is a straight already on the board
    //Ignores pairs because this would be covered by the non paired card values
    pub others_with_str8: Vec<(CardValue, CardValue)>,
    pub others_with_str8_draw: Vec<(CardValue, CardValue)>,
    pub others_with_gut_shot: Vec<(CardValue, CardValue)>,

    pub has_straight: bool,

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

    //AA, AKs, etc
    //rank 1 means the nuts
    //For suited, it's assumed the best choice is the suited version
    //Capped at rank 256, length is 169
    pub simple_rank_index: Vec<u8>,
    pub lowest_rank_index: u8,
}

pub fn calc_board_texture(cards: &[Card]) -> BoardTexture {
    let mut texture = BoardTexture {
        same_suited_max_count: 0,
        suits_with_max_count: Vec::new(),
        gaps: Vec::with_capacity(cards.len() - 1),
        has_trips: false,
        has_pair: false,
        has_two_pair: false,
        has_fh: false,
        has_quads: false,
        has_straight: false,

        high_value_count: 0,
        med_value_count: 0,
        low_value_count: 0,

        others_with_str8: Vec::new(),
        others_with_str8_draw: Vec::new(),
        others_with_gut_shot: Vec::new(),

        simple_rank_index: vec![0; 169],
        lowest_rank_index: 0,
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

    // Same suit calculations
    for svs in cards_metrics.suit_value_sets.iter() {
        texture.same_suited_max_count = max(texture.same_suited_max_count, svs.count_ones() as u8);
    }
    texture.suits_with_max_count = Vec::with_capacity(4);
    for (suit_index, svs) in cards_metrics.suit_value_sets.iter().enumerate() {
        if texture.same_suited_max_count != svs.count_ones() as u8 {
            continue;
        }
        texture
            .suits_with_max_count
            .push((suit_index as u8).try_into().unwrap());
    }

    // pairs
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

    //Calculate straight related fields
    calc_straight_texture(&mut texture, cards_metrics.value_set, cards);

    //Calculate simple rank index
    calc_simple_rank_index(&mut texture, cards);

    texture
}

fn calc_simple_rank_index(texture: &mut BoardTexture, cards: &[Card]) {
    let hash_func = load_boomperfect_hash();
    texture.simple_rank_index = vec![0; 169];

    //along with index
    let mut raw_eval_values: Vec<(Option<Rank>, u8)> = (0..texture.simple_rank_index.len())
        .map(|i| (None, i as u8))
        .collect();

    let used_cards: CardUsedType = {
        let mut uc = CardUsedType::default();
        for c in cards.iter() {
            uc.set(c.index as usize, true);
        }
        uc
    };

    // get flush suit
    let flush_suit = if texture.same_suited_max_count >= 3 {
        assert_eq!(1, texture.suits_with_max_count.len());
        Some(texture.suits_with_max_count[0])
    } else {
        None
    };

    for hole_cards in ALL_HOLE_CARDS.iter() {
        let simple_index = hole_cards.simple_range_index();
        assert_eq!(raw_eval_values[simple_index].1, simple_index as u8);

        if used_cards[hole_cards.hi_card().index as usize]
            || used_cards[hole_cards.lo_card().index as usize]
        {
            //Already used
            continue;
        }

        if hole_cards.is_suited()
            && flush_suit.is_some()
            && hole_cards.hi_card().suit == flush_suit.unwrap()
        {
            //Always recompute
        } else if raw_eval_values[simple_index].0.is_some() {
            //Already computed
            continue;
        }

        let eval = fast_hand_eval(cards.iter().cloned().chain(hole_cards.iter()), &hash_func);

        raw_eval_values[simple_index].0 = Some(eval);
    }

    //Sort by rank, highest first
    raw_eval_values.sort_by(|a, b| b.0.cmp(&a.0));

    let mut current_rank = 1;
    let mut current_eval_value = raw_eval_values[0].0;

    for (rank_value, simple_range_index) in raw_eval_values.into_iter() {
        if rank_value != current_eval_value && rank_value.is_some() {
            current_rank += 1;
            current_eval_value = rank_value;
        }

        if rank_value.is_none() {
            texture.simple_rank_index[simple_range_index as usize] = u8::MAX;
        } else {
            texture.simple_rank_index[simple_range_index as usize] = current_rank;
        }
    }
    texture.lowest_rank_index = current_rank;
}

fn calc_straight_texture(texture: &mut BoardTexture, value_set: u32, cards: &[Card]) {
    texture.has_straight = rank_straight(value_set).is_some();

    if texture.has_straight {
        return;
    }

    for hole_card1_value in 0..NUMBER_OF_RANKS {
        if value_set & (1 << hole_card1_value) != 0 {
            continue;
        }
        let hole_card1_card_value: CardValue = hole_card1_value.try_into().unwrap();

        for hole_card2_value in hole_card1_value + 1..NUMBER_OF_RANKS {
            if hole_card2_value & (1 << hole_card2_value) != 0 {
                continue;
            }
            let hole_card2_card_value: CardValue = hole_card2_value.try_into().unwrap();

            if rank_straight(value_set | (1 << hole_card1_value) | (1 << hole_card2_value))
                .is_some()
            {
                texture
                    .others_with_str8
                    .push((hole_card2_card_value, hole_card1_card_value));
                continue;
            }

            //draws only apply on flop/turn
            if cards.len() >= 5 {
                continue;
            }

            let mut values_that_make_straight = 0;
            for drawing_value in 0..NUMBER_OF_RANKS {
                if value_set & (1 << drawing_value) != 0 {
                    continue;
                }
                if drawing_value == hole_card1_value || drawing_value == hole_card2_value {
                    continue;
                }
                if rank_straight(
                    value_set
                        | (1 << hole_card1_value)
                        | (1 << hole_card2_value)
                        | (1 << drawing_value),
                )
                .is_some()
                {
                    values_that_make_straight += 1;
                }
            }

            assert!(values_that_make_straight <= 2);

            if values_that_make_straight == 1 {
                texture
                    .others_with_gut_shot
                    .push((hole_card2_card_value, hole_card1_card_value));
            } else if values_that_make_straight == 2 {
                texture
                    .others_with_str8_draw
                    .push((hole_card2_card_value, hole_card1_card_value));
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use log::{debug, info, trace};

    use crate::{init_test_logger, Board, HoleCards};

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
        assert_eq!(
            texture.others_with_str8,
            vec![(CardValue::Five, CardValue::Four)]
        );
        assert_eq!(texture.others_with_str8_draw.len(), 0);
        assert_eq!(texture.others_with_gut_shot.len(), 18);

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
        assert_eq!(texture.others_with_str8.len(), 0);
        assert_eq!(texture.others_with_str8_draw.len(), 0);
        assert_eq!(texture.others_with_gut_shot.len(), 0);

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
        assert_eq!(
            texture.others_with_str8,
            vec![(CardValue::Jack, CardValue::Ten)]
        );
        assert_eq!(texture.others_with_str8_draw.len(), 0);
        assert_eq!(texture.others_with_gut_shot.len(), 0);

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

        let cards = Board::try_from("6c 8h Td")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(
            texture.others_with_str8,
            vec![(CardValue::Nine, CardValue::Seven),]
        );
        assert_eq!(
            texture.others_with_str8_draw,
            vec![
                (CardValue::Seven, CardValue::Four),
                (CardValue::Seven, CardValue::Five),
                (CardValue::Jack, CardValue::Nine),
                (CardValue::Queen, CardValue::Nine),
            ]
        );
        assert_eq!(texture.others_with_gut_shot.len(), 17);

        let cards = Board::try_from("6c 8h Td 9d")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(
            texture.others_with_str8,
            vec![
                (CardValue::Seven, CardValue::Two),
                (CardValue::Seven, CardValue::Three),
                (CardValue::Seven, CardValue::Four),
                (CardValue::Seven, CardValue::Five),
                (CardValue::Eight, CardValue::Seven),
                (CardValue::Nine, CardValue::Seven),
                (CardValue::Ten, CardValue::Seven),
                (CardValue::Jack, CardValue::Seven),
                (CardValue::Queen, CardValue::Seven),
                (CardValue::King, CardValue::Seven),
                (CardValue::Ace, CardValue::Seven),
                (CardValue::Queen, CardValue::Jack),
            ]
        );
        assert_eq!(texture.others_with_str8_draw.len(), 12);
        assert_eq!(texture.others_with_gut_shot.len(), 31);

        let cards = Board::try_from("Ac 2h 4d 5d")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(texture.others_with_str8.len(), 11);
        assert_eq!(texture.others_with_str8_draw.len(), 3);
        assert_eq!(texture.others_with_gut_shot.len(), 33);

        let cards = Board::try_from("2h 4d 5d")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        trace!("Texture\n{:#?}", &texture);

        assert_eq!(
            texture.others_with_str8,
            vec![
                (CardValue::Six, CardValue::Three),
                (CardValue::Ace, CardValue::Three),
            ]
        );
        assert_eq!(texture.others_with_str8_draw.len(), 11);
        assert_eq!(texture.others_with_gut_shot.len(), 14);
    }

    #[test]
    fn test_simple_rank() {
        init_test_logger();

        trace!("test_board_texture");
        let cards = Board::try_from("3c 2s As")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        let nuts1: HoleCards = "4c 5h".parse().unwrap();
        let nuts2: HoleCards = "Ac Ad".parse().unwrap();
        let nuts3: HoleCards = "3d 3s".parse().unwrap();
        let nuts4: HoleCards = "2d 2h".parse().unwrap();
        let nuts5: HoleCards = "Ac 3d".parse().unwrap();

        assert_eq!(
            texture.simple_rank_index[nuts1.simple_range_index() as usize],
            1
        );
        assert_eq!(
            texture.simple_rank_index[nuts2.simple_range_index() as usize],
            2
        );
        assert_eq!(
            texture.simple_rank_index[nuts3.simple_range_index() as usize],
            3
        );
        assert_eq!(
            texture.simple_rank_index[nuts4.simple_range_index() as usize],
            4
        );
        assert_eq!(
            texture.simple_rank_index[nuts5.simple_range_index() as usize],
            5
        );

        //suited should not matter
        let nuts1s: HoleCards = "4s 5s".parse().unwrap();
        let nuts5s: HoleCards = "Ad 3d".parse().unwrap();

        assert_eq!(
            texture.simple_rank_index[nuts1.simple_range_index() as usize],
            1
        );
        assert_eq!(
            texture.simple_rank_index[nuts5.simple_range_index() as usize],
            5
        );

        assert_ne!(nuts1.simple_range_index(), nuts1s.simple_range_index());
        assert_ne!(nuts5.simple_range_index(), nuts5s.simple_range_index());

        let cards = Board::try_from("3d 5d Ad Ac Ks")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        let nuts1: HoleCards = "2d 4d".parse().unwrap();
        let nuts2: HoleCards = "As Ah".parse().unwrap();
        let nuts3: HoleCards = "As Kc".parse().unwrap();
        let nuts4: HoleCards = "Kd 2d".parse().unwrap();

        assert_eq!(
            texture.simple_rank_index[nuts1.simple_range_index() as usize],
            1
        );
        assert_eq!(
            texture.simple_rank_index[nuts2.simple_range_index() as usize],
            2
        );
        assert_eq!(
            texture.simple_rank_index[nuts3.simple_range_index() as usize],
            3
        );
        // behind all the full houses
        assert_eq!(
            texture.simple_rank_index[nuts4.simple_range_index() as usize],
            17
        );

        let h1: HoleCards = "Ad 2d".parse().unwrap();
        let h2: HoleCards = "Ah 2d".parse().unwrap();
        //Because the Ad is already being used, this should be the same rank as A2o
        assert_eq!(
            texture.simple_rank_index[h1.simple_range_index() as usize],
            texture.simple_rank_index[h2.simple_range_index() as usize]
        );

        let cards = Board::try_from("2d Ad 2c 2h 2s")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        let h1: HoleCards = "Kd Ks".parse().unwrap();
        assert_eq!(
            texture.simple_rank_index[h1.simple_range_index() as usize],
            1
        );

        let h1: HoleCards = "7d 8s".parse().unwrap();
        assert_eq!(
            texture.simple_rank_index[h1.simple_range_index() as usize],
            1
        );

        let h1: HoleCards = "Kd 2d".parse().unwrap();
        assert_eq!(
            texture.simple_rank_index[h1.simple_range_index() as usize],
            u8::MAX
        );

        //Every hand is the nuts
        assert_eq!(texture.lowest_rank_index, 1);

        let cards = Board::try_from("2d 7d 5h")
            .unwrap()
            .as_slice_card()
            .to_vec();
        let texture = calc_board_texture(&cards);

        let h1: HoleCards = "3s 4h".parse().unwrap();
        assert_eq!(
            texture.simple_rank_index[h1.simple_range_index() as usize],
            texture.lowest_rank_index
        );
        assert_eq!(texture.lowest_rank_index, 91);
    }
}
