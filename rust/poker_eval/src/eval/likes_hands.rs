use std::{fmt::{Display, Formatter}, cmp::max};

use crate::{
    Board, BoardTexture, CardValue, FlushDrawType, HoleCards, PartialRankContainer, PokerError,
    Round, StraightDrawType,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum LikesHandLevel {
    None, //might even fold instead of checking
    CallSmallBet,
    SmallBet,
    LargeBet,
    AllIn,
}

impl Display for LikesHandLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LikesHandLevel::None => write!(f, "None"),
            LikesHandLevel::CallSmallBet => write!(f, "CallSmallBet"),
            LikesHandLevel::SmallBet => write!(f, "SmallBet"),
            LikesHandLevel::LargeBet => write!(f, "LargeBet"),
            LikesHandLevel::AllIn => write!(f, "AllIn"),
        }
    }
}

pub struct LikesHandResponse {
    pub likes_hand: LikesHandLevel,
    pub likes_hand_comments: Vec<String>,
    pub not_like_hand_comments: Vec<String>,
}

pub fn likes_hand(
    prc: &PartialRankContainer,
    ft: &BoardTexture,

    board: &Board,
    hc: &HoleCards,
) -> Result<LikesHandResponse, PokerError> {
    let round = board.get_round()?;

    let mut likes_hand_comments: Vec<String> = Vec::new();
    let mut not_like_hand_comments: Vec<String> = Vec::new();
    let mut likes_hand = LikesHandLevel::None;

    if let Some(p) = prc.lo_pair {

        likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
        //if p.number_above == 0 {
        likes_hand_comments.push(format!("lo pair {}", hc.get_hi_card().value));
        //}
        if p.made_quads {
            likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
        }
        if p.made_set {
            likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
        }
    }
    if let Some(p) = prc.hi_pair {
        //if p.number_above == 0 {
        likes_hand_comments.push(format!("pair {}", hc.get_hi_card().value));
        //}

        if p.number_above == 0 {
            likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
        }

        if p.made_quads {
            likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
        }
        if p.made_set {
            likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
        }
    }
    if let Some(p) = prc.pocket_pair {

        if p.number_above == 0 {
            likes_hand_comments.push(format!("pocket overpair {}", hc.get_hi_card().value));
            likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
        } else {
            likes_hand_comments.push(format!("pocket pair {}", hc.get_hi_card().value));
            likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
        }
        if p.made_set {
            likes_hand = max(likes_hand, LikesHandLevel::AllIn);
            likes_hand_comments.push(format!("Pocket Pair Set {}", hc.get_hi_card().value));
        }
        if p.made_quads {
            likes_hand_comments.push(format!("Pocket Pair Quads {}", hc.get_hi_card().value));
        }
    }
    if let Some(p) = prc.hi_card {
        //if the board is paired, then only stay in with an ace or king
        if p.number_above == 0 {
            if ft.has_pair || ft.has_trips || ft.has_two_pair {
                if hc.get_hi_card().value >= CardValue::King {
                    likes_hand_comments.push(format!(
                        "hi card overcard is ace or king with paired board {}",
                        hc.get_hi_card().value
                    ));
                    likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
                } else {
                    not_like_hand_comments.push(format!(
                        "hi card overcard is not ace or king with paired board {}",
                        hc.get_hi_card().value
                    ));
                }
            } else {
                likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
                likes_hand_comments.push(format!("hi card is overpair {}", hc.get_hi_card().value));
            }
        }
    }
    if round != Round::River {
        if let Some(p) = prc.flush_draw {
            if p.flush_draw_type == FlushDrawType::FlushDraw {
                likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
            }
        }
        if let Some(p) = prc.straight_draw {
            if p.straight_draw_type == StraightDrawType::OpenEnded
                || p.straight_draw_type == StraightDrawType::DoubleGutShot
            {
                likes_hand_comments.push(format!("Straight draw"));
            }
            //likes_hand_comments.push( format!("Gutshot straight draw {}", p.) );
        }
    }

    return Ok(LikesHandResponse {
        likes_hand,
        likes_hand_comments,
        not_like_hand_comments,
    });
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::{
        board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
        exact_equity::calc_equity,
        init_test_logger,
        pre_calc::NUMBER_OF_RANKS,
        Board, BoolRange, Card, CardValue, Suit, Deck,
    };

    use super::*;

    fn get_expected_equity_ranges(likes_hand: LikesHandLevel) -> [f64;2] {
        match likes_hand {
            LikesHandLevel::None => [0.0, 0.25],
            LikesHandLevel::CallSmallBet => [0.10, 0.30],
            LikesHandLevel::SmallBet => [0.20, 0.55],
            LikesHandLevel::LargeBet => [0.30, 0.75],
            LikesHandLevel::AllIn => [0.50, 1.0],
        }
    }

    #[test]
    fn test_likes_hand() {
        /*
        cargo test test_likes_hand --release -- --nocapture
         */
        init_test_logger();

        let mut partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
            EvalCacheWithHcReDb::new().unwrap();

        let mut flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

        let mut ranges: Vec<BoolRange> = vec![
            "Ks6s".parse().unwrap(),
            BoolRange::all_enabled(),
            BoolRange::all_enabled(),
            BoolRange::all_enabled(),
        ];

        

        let mut deck = Deck::new();
        //Test rainbow boards
        for v1 in 0..NUMBER_OF_RANKS {
            let card_value1: CardValue = v1.into();
            for v2 in v1..NUMBER_OF_RANKS {
                let card_value2: CardValue = v2.into();
                for v3 in v2..NUMBER_OF_RANKS {
                    let card_value3: CardValue = v3.into();

                    deck.reset();
                    let card1 = Card::new(card_value1, Suit::Spade);
                    let card2 = Card::new(card_value2, Suit::Heart);
                    let card3 = Card::new(card_value3, Suit::Diamond);

                    deck.set_used_card(card1);
                    deck.set_used_card(card2);
                    deck.set_used_card(card3);

                    let mut board: Board = Board::new_from_cards(&[card1, card2, card3]);
                    board.get_index();
                    let ft = flop_texture_db.get_put(&board).unwrap();

                    for hc_v1 in 0..NUMBER_OF_RANKS {
                        let hole_card_value1 = hc_v1.into();
                        let hole_card1 = Card::new(hole_card_value1, Suit::Spade);

                        if deck.is_used(hole_card1) {
                            continue;
                        }
                        for hc_v2 in hc_v1..NUMBER_OF_RANKS {
                            let hole_card_value2 = hc_v2.into();
                            let hole_card2 = Card::new(hole_card_value2, Suit::Club);

                            if deck.is_used(hole_card2) {
                                continue;
                            }

                            let hc : HoleCards = HoleCards::new(hole_card1, hole_card2).unwrap();

                            let prc = partial_rank_db.get_put(&board, &hc).unwrap();
                            let likes_hand_res = likes_hand(&prc, &ft, &board, &hc).unwrap();

                            //Get equity
                            ranges[0].data.fill(false);
                            ranges[0].data.set(hc.to_range_index(), true);

                            //info!("Trying board {} and hole cards {}", &board, &hc);

                            let results = calc_equity(&board, &ranges, 10_000).unwrap();

                            //info!("Equiny hose board {} and hole cards {}", &board, &hc);
                            let allowed_range = get_expected_equity_ranges(likes_hand_res.likes_hand);

                            if results[0] < allowed_range[0] || results[0] > allowed_range[1] {
                                info!("With board {} and hole cards {}; equity is {:.2} but likes hand is {} and expected range is {:.2} to {:.2}",
                                board, hc, results[0]*100.0, likes_hand_res.likes_hand, allowed_range[0]*100.0, allowed_range[1]*100.0);
                            }

                            // if results[0] > 0.75 {
                            //     //assert_eq!(likes_hand_res.likes_hand, LikesHandLevel::AllIn);
                            //     if likes_hand_res.likes_hand != LikesHandLevel::AllIn {
                            //         info!("With board {} and hole cards {}; equity is {:.2} expected all in but likes hand is {}",
                            //         board, hc, results[0]*100.0, likes_hand_res.likes_hand);
                            //     }
                            // } else if results[0] > 0.50 {
                            //     if likes_hand_res.likes_hand != LikesHandLevel::LargeBet {
                            //         info!("With board {} and hole cards {}; equity is {:.2}; expected large bet but likes hand is {}",
                            //         board, hc, results[0]*100.0, likes_hand_res.likes_hand);
                            //     }
                            // } else if results[0] > 0.30 {
                            //     if likes_hand_res.likes_hand != LikesHandLevel::SmallBet {
                            //         info!("With board {} and hole cards {}; equity is {:.2} expected small bet but likes hand is {}",
                            //         board, hc, results[0]*100.0, likes_hand_res.likes_hand);
                            //     }
                            // } else if results[0] > 0.20 {
                            //     if likes_hand_res.likes_hand > LikesHandLevel::CallSmallBet {
                            //         info!("With board {} and hole cards {}; equity is {:.2} expected call bet / none but likes hand is {}",
                            //         board, hc, results[0]*100.0, likes_hand_res.likes_hand);
                            //     }
                            // } else {
                            //     if likes_hand_res.likes_hand > LikesHandLevel::CallSmallBet {
                            //         info!("With board {} and hole cards {}; equity is {:.2} but likes hand is {}",
                            //         board, hc, results[0]*100.0, likes_hand_res.likes_hand);
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }
}
