use std::{
    cmp::{max, min},
    fmt::{Display, Formatter}, mem,
};

use crate::{
    pre_calc::rank::{Rank, RankEnum},
    Board, BoardTexture, CardValue, FlushDrawType, HoleCards, MadeWith, PartialRankContainer,
    PokerError, Round, StraightDrawType,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum LikesHandLevel {
    None=0, //might even fold instead of checking
    CallSmallBet=1,
    SmallBet=2, // corresponds to calling a 1/3 pot bet, so roughly 20% equity
    LargeBet=3, // up to a pot size bet 
    AllIn=4, // calling overbets, going all in etc. 
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

impl TryFrom<u8> for LikesHandLevel {
    type Error = PokerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > LikesHandLevel::AllIn as u8 {
            return Err(PokerError::from_string(format!("Invalid value: {}", value)));
        }

        Ok(unsafe { mem::transmute(value) })
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
    rank: &Rank,
    board: &Board,
    hc: &HoleCards,
    //Idea is to tweak responses based on 2 vs more players
    num_in_pot: u8, 
) -> Result<LikesHandResponse, PokerError> {

    assert!(num_in_pot >= 2 && num_in_pot <= 10);

    let round = board.get_round()?;

    let mut likes_hand_comments: Vec<String> = Vec::new();
    let mut not_like_hand_comments: Vec<String> = Vec::new();
    let mut likes_hand = LikesHandLevel::None;

    if let Some(s) = prc.made_a_set() {
        match s {
            MadeWith::HiCard => {
                likes_hand_comments.push(format!(
                    "Made a set with hi card {}",
                    hc.get_hi_card().value
                ));
                likes_hand = max(likes_hand, LikesHandLevel::AllIn);
            }
            MadeWith::LoCard => {
                likes_hand_comments.push(format!(
                    "Made a set with lo card {}",
                    hc.get_lo_card().value
                ));
                likes_hand = max(likes_hand, LikesHandLevel::AllIn);
            }
            MadeWith::BothCards => {
                likes_hand_comments.push(format!(
                    "Made a set with pocket pair {}",
                    hc.get_hi_card().value
                ));
                likes_hand = max(likes_hand, LikesHandLevel::AllIn);
            }
        }
    }
    if !ft.has_quads && rank.get_rank_enum() >= RankEnum::FourOfAKind {
        likes_hand_comments.push(format!("Made Quads or better"));
        likes_hand = max(likes_hand, LikesHandLevel::AllIn);
    }

    if let Some(p) = prc.hi_pair {
        if p.number_above == 0 {
            if let Some(_p) = prc.lo_pair {
                likes_hand_comments
                    .push(format!("two pair with hi card {}", hc.get_hi_card().value));
                likes_hand = max(likes_hand, LikesHandLevel::AllIn);
            } else {
                if hc.get_hi_card().value >= CardValue::Eight {
                    likes_hand_comments.push(format!("top pair {}", hc.get_hi_card().value));
                    likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
                } else {
                    likes_hand_comments.push(format!("top pair, <= 8 {}", hc.get_hi_card().value));
                    likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
                }
            }
        } else if p.number_above == 1 {
            likes_hand_comments.push(format!("mid pair {}", hc.get_hi_card().value));
            likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
        } else {
            likes_hand_comments.push(format!("3rd or worse pair {}", hc.get_hi_card().value));
            likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
        }
    } else if let Some(p) = prc.lo_pair {
        //likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
        if p.number_above == 0 {
            if hc.get_lo_card().value <= CardValue::Eight {
                likes_hand_comments.push(format!(
                    "lo card is top pair {} but is small",
                    hc.get_lo_card().value
                ));
                likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
            } else {
                likes_hand_comments.push(format!("lo card is top pair {}", hc.get_lo_card().value));
                likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
            }
        } else {
            not_like_hand_comments.push(format!(
                "lo card is not top pair {}",
                hc.get_lo_card().value
            ));
        }
    }

    if let Some(p) = prc.pocket_pair {
        if p.number_above == 0 {
            likes_hand_comments.push(format!("pocket overpair {}", hc.get_hi_card().value));
            likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
        } else {
            if p.number_below == 0 {
                likes_hand_comments.push(format!("pocket underpair {}", hc.get_hi_card().value));
                likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
            } else if p.number_above >= 2 {
                likes_hand_comments.push(format!(
                    "pocket pair; but with 2 above {}",
                    hc.get_hi_card().value
                ));
                likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
            } else {
                likes_hand_comments.push(format!("pocket pair {}", hc.get_hi_card().value));
                likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
            }
        }

        if !ft.has_quads && !ft.has_fh && rank.get_rank_enum() >= RankEnum::FullHouse {
            likes_hand_comments.push(format!(
                "Pocket Pair FH or better {}",
                hc.get_hi_card().value
            ));
            likes_hand = max(likes_hand, LikesHandLevel::AllIn);
        }
    }
    if let Some(p) = prc.hi_card {
        if prc.get_num_overcards() >= 2 && hc.get_lo_card().value >= CardValue::Ten {
            likes_hand_comments.push(format!(
                "2 good overcards {} and {}",
                hc.get_hi_card().value,
                hc.get_lo_card().value
            ));
            likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
        } else if hc.get_hi_card().value >= CardValue::Ten {
            //if the board is paired, then only stay in with an ace or king
            if p.number_above == 0 {
                if ft.has_trips && hc.get_hi_card().value > CardValue::King {
                    likes_hand_comments.push(format!(
                        "Trips on board with an Ace {}",
                        hc.get_hi_card().value
                    ));
                    likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
                } else if ft.has_pair || ft.has_trips || ft.has_two_pair {
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
                    if hc.get_hi_card().value >= CardValue::King {
                        likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
                        likes_hand_comments.push(format!(
                            "overcard card A/K is overpair {}",
                            hc.get_hi_card().value
                        ));
                    } else {
                        not_like_hand_comments.push(format!(
                            "overcard card is not ace or king {}",
                            hc.get_hi_card().value
                        ));
                    }
                }
            }
        }
    }
    if round != Round::River {
        if let Some(p) = prc.flush_draw {
            if p.flush_draw_type == FlushDrawType::FlushDraw {
                if prc.has_straight_draw() {
                    likes_hand_comments.push(format!("Flush & str draw {}", p.hole_card_value));
                    likes_hand = max(likes_hand, LikesHandLevel::AllIn);
                } else {
                    if prc.has_top_pair() {
                        likes_hand_comments
                            .push(format!("Flush draw {} with top pair", p.hole_card_value));
                        likes_hand = max(likes_hand, LikesHandLevel::AllIn);
                    } else if p.hole_card_value >= CardValue::King {
                        likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
                        likes_hand = max(likes_hand, LikesHandLevel::LargeBet);
                    } else {
                        likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
                        likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
                    }
                }
            }
        }
        if let Some(p) = prc.straight_draw {
            if p.straight_draw_type == StraightDrawType::OpenEnded
                || p.straight_draw_type == StraightDrawType::DoubleGutShot
            {
                likes_hand_comments.push(format!("Straight draw"));
                likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
            } else {
                if prc.get_num_overcards() >= 1 && hc.get_hi_card().value >= CardValue::Jack {
                    likes_hand_comments.push(format!(
                        "Gutshot straight draw {} with 1 or more overcards J or better",
                        p.straight_draw_type
                    ));
                    likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
                } else {
                    likes_hand_comments
                        .push(format!("Gutshot straight draw {}", p.straight_draw_type));
                    likes_hand = max(likes_hand, LikesHandLevel::CallSmallBet);
                }
            }
            //
        }
    }

    if RankEnum::Straight == rank.get_rank_enum() {
        let ratio_with_str8 = ft.num_with_str8 as f64 / ft.num_hole_cards as f64;
        if ratio_with_str8 > 0.5 {
            likes_hand_comments.push(format!("Straight on board"));
        } else {
            likes_hand_comments.push(format!(
                "Made straight with only {} / {} = {:.2}% other hole cards",
                ft.num_with_str8,
                ft.num_hole_cards,
                ratio_with_str8 * 100.0
            ));
            likes_hand = max(likes_hand, LikesHandLevel::AllIn);
        }
    }

    if RankEnum::Flush == rank.get_rank_enum() {
        if ft.same_suited_max_count >= 4 {
            if let Some(made_flush) = prc.made_flush {
                if made_flush == CardValue::Ace {
                    likes_hand_comments
                        .push(format!("Made nut flush with a good card {}", made_flush));
                    likes_hand = max(likes_hand, LikesHandLevel::AllIn);
                } else if made_flush <= CardValue::Ten {
                    not_like_hand_comments
                        .push(format!("Flush 4 on board, and only have {}", made_flush));
                    likes_hand = min(likes_hand, LikesHandLevel::CallSmallBet);
                } else {
                    likes_hand_comments.push(format!(
                        "Made decent flush with 4 on the board {}",
                        made_flush
                    ));
                }
            }
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
        init_test_logger,
        monte_carlo_equity::calc_equity,
        pre_calc::{
            fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash, NUMBER_OF_RANKS,
        },
        Board, BoolRange, Card, CardValue, Deck, Suit,
    };

    use super::*;

    fn get_expected_equity_ranges(likes_hand: LikesHandLevel) -> [f64; 2] {
        match likes_hand {
            LikesHandLevel::None => [0.0, 0.25],
            LikesHandLevel::CallSmallBet => [0.05, 0.35],
            LikesHandLevel::SmallBet => [0.10, 0.55],
            LikesHandLevel::LargeBet => [0.25, 0.75],
            LikesHandLevel::AllIn => [0.35, 1.0],
        }
    }

    //#[test]
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    fn test_likes_hand() {
        /*
        cargo test test_likes_hand --release -- --nocapture
         */
        init_test_logger();

        let mut partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
            EvalCacheWithHcReDb::new().unwrap();

        let mut flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

        let mut ranges: Vec<BoolRange> = vec![
            //We'll replace this one with the hole cards
            BoolRange::all_enabled(),
            BoolRange::all_enabled(),
            BoolRange::all_enabled(),
            BoolRange::all_enabled(),
        ];

        let mut it_count = 0;
        let mut issue_count = 0;

        let mut deck = Deck::new();
        let hash_func = load_boomperfect_hash();

        //Test rainbow boards
        for v1 in 0..NUMBER_OF_RANKS {
            let card_value1: CardValue = v1.into();
            for v2 in v1..NUMBER_OF_RANKS {
                let card_value2: CardValue = v2.into();
                for v3 in v2..NUMBER_OF_RANKS {
                    let card_value3: CardValue = v3.into();

                    deck.reset();
                    let card1 = Card::new(card_value1, Suit::Spade);
                    //let card2 = Card::new(card_value2, Suit::Heart);
                    let card2 = Card::new(card_value2, Suit::Spade);
                    let card3 = Card::new(card_value3, Suit::Diamond);

                    if card1 == card2 {
                        continue;
                    }

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
                            //let hole_card2 = Card::new(hole_card_value2, Suit::Club);
                            let hole_card2 = Card::new(hole_card_value2, Suit::Spade);

                            if deck.is_used(hole_card2) {
                                continue;
                            }
                            if hole_card1 == hole_card2 {
                                continue;
                            }

                            let hc: HoleCards = HoleCards::new(hole_card1, hole_card2).unwrap();

                            let prc = partial_rank_db.get_put(&board, &hc).unwrap();

                            let rank =
                                fast_hand_eval(board.get_iter().chain(hc.get_iter()), &hash_func);

                            let likes_hand_res = likes_hand(&prc, &ft, &rank, &board, &hc, 4).unwrap();

                            //Get equity
                            ranges[0].data.fill(false);
                            ranges[0].data.set(hc.to_range_index(), true);

                            //info!("Trying board {} and hole cards {}", &board, &hc);
                            it_count += 1;
                            if it_count % 5000 == 0 && it_count > 0 {
                                info!("Iteration {}", it_count);
                            }
                            let results = calc_equity(&board, &ranges, 1_000).unwrap();

                            if it_count > 450_000 {
                                return;
                            }

                            //info!("Equiny hose board {} and hole cards {}", &board, &hc);
                            let allowed_range =
                                get_expected_equity_ranges(likes_hand_res.likes_hand);

                            if results[0] < allowed_range[0] || results[0] > allowed_range[1] {
                                info!("Issue {} of iteration {} With board {} and hole cards {}; equity is {:.2} but likes hand is {} and expected range is {:.2} to {:.2}\npos: {} ; neg: {}",
                                issue_count, it_count,
                                board, hc, results[0]*100.0, likes_hand_res.likes_hand, allowed_range[0]*100.0, allowed_range[1]*100.0,
                                likes_hand_res.likes_hand_comments.join(", "), likes_hand_res.not_like_hand_comments.join(", ")
                            );

                                issue_count += 1;
                                if issue_count > 1050 {
                                    return;
                                }
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
