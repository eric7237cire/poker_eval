use std::{
    cmp::{max, min},
    fmt::{Display, Formatter},
    mem,
};

use crate::{
    pre_calc::rank::{Rank, RankEnum},
    Board, BoardTexture, CardValue, FlushDrawType, HoleCards, MadeWith, PartialRankContainer,
    PokerError, Round, StraightDrawType,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum LikesHandLevel {
    None = 0, //might even fold instead of checking
    CallSmallBet = 1,
    SmallBet = 2, // corresponds to calling a 1/3 pot bet, so roughly 20% equity
    LargeBet = 3, // up to a pot size bet
    AllIn = 4,    // calling overbets, going all in etc.
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

#[derive(Debug)]
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
    /*
    establish maxes based on what we like

    at the end we enforce mins (e.g. 4 to a flush on board)
    */

    assert!(num_in_pot >= 2 && num_in_pot <= 10);

    let mut likes_hand_comments: Vec<String> = Vec::new();
    let mut not_like_hand_comments: Vec<String> = Vec::new();
    let mut likes_hand = LikesHandLevel::None;

    handle_set_and_two_pair(
        prc,
        ft,
        rank,
        board,
        hc,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
    );

    handle_hi_and_lo_pair(
        prc,
        ft,
        hc,
        board,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    handle_pocket_pair(
        prc,
        ft,
        rank,
        board,
        hc,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    handle_hi_card(
        prc,
        ft,
        hc,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    likes_draws(
        prc,
        ft,
        board,
        hc,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    likes_made_flushes_and_straights(
        rank,
        ft,
        prc,
        &mut likes_hand,
        &mut likes_hand_comments,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    worried_about_straights(
        ft,
        rank,
        &mut likes_hand,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    worried_about_flushes(
        ft,
        prc,
        rank,
        &mut likes_hand,
        &mut not_like_hand_comments,
        num_in_pot,
    );

    return Ok(LikesHandResponse {
        likes_hand,
        likes_hand_comments,
        not_like_hand_comments,
    });
}

fn handle_set_and_two_pair(
    prc: &PartialRankContainer,
    ft: &BoardTexture,
    rank: &Rank,
    board: &Board,
    hc: &HoleCards,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    _not_like_hand_comments: &mut Vec<String>,
) {
    if let Some(s) = prc.made_a_set() {
        match s {
            MadeWith::HiCard => {
                likes_hand_comments.push(format!(
                    "Made a set with hi card {}",
                    hc.get_hi_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
            }
            MadeWith::LoCard => {
                likes_hand_comments.push(format!(
                    "Made a set with lo card {}",
                    hc.get_lo_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
            }
            MadeWith::BothCards => {
                likes_hand_comments.push(format!(
                    "Made a set with pocket pair {}",
                    hc.get_hi_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
            }
        }
    }
    if !ft.has_quads && rank.get_rank_enum() >= RankEnum::FourOfAKind {
        likes_hand_comments.push(format!("Made Quads or better"));
        *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
    }

    if let Some(num_above) = prc.made_two_pair() {
        if board.get_num_cards() == 3 {
            likes_hand_comments.push(format!(
                "Made two pair on flop with {} above hi card ",
                num_above
            ));
            *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
        } else {
            likes_hand_comments.push(format!("Made two pair with {} above hi card ", num_above));
            *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
        }
    }
}

fn handle_hi_card(
    prc: &PartialRankContainer,
    ft: &BoardTexture,
    hc: &HoleCards,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    if prc.hi_card.is_none() {
        return;
    }
    let p = prc.hi_card.unwrap();
    if prc.get_num_overcards() >= 2 && hc.get_lo_card().value >= CardValue::Ten {
        likes_hand_comments.push(format!(
            "2 good overcards {} and {}",
            hc.get_hi_card().value,
            hc.get_lo_card().value
        ));
        if num_in_pot == 2 {
            *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
        } else {
            *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
        }
    } else if hc.get_hi_card().value >= CardValue::Ten {
        //if the board is paired, then only stay in with an ace or king
        if p.number_above == 0 {
            if ft.has_trips && hc.get_hi_card().value > CardValue::King {
                likes_hand_comments.push(format!(
                    "Trips on board with an Ace {}",
                    hc.get_hi_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
            } else if ft.has_pair || ft.has_trips || ft.has_two_pair {
                if hc.get_hi_card().value >= CardValue::King {
                    likes_hand_comments.push(format!(
                        "hi card overcard is ace or king with paired board {}",
                        hc.get_hi_card().value
                    ));
                    *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
                } else {
                    not_like_hand_comments.push(format!(
                        "hi card overcard is not ace or king with paired board {}",
                        hc.get_hi_card().value
                    ));
                }
            } else {
                if hc.get_hi_card().value >= CardValue::King {
                    *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
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

fn handle_hi_and_lo_pair(
    prc: &PartialRankContainer,
    _ft: &BoardTexture,
    hc: &HoleCards,
    board: &Board,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    let round = board.get_round().unwrap();

    if num_in_pot == 2 && (prc.hi_pair.is_some() || prc.lo_pair.is_some()) {
        likes_hand_comments.push(format!("Pair on board heads up"));
        *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
        return;
    }

    if let Some(p) = prc.hi_pair {
        if p.number_above == 0 {
            if let Some(_p) = prc.lo_pair {
                likes_hand_comments
                    .push(format!("two pair with hi card {}", hc.get_hi_card().value));
                *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
            } else {
                if hc.get_hi_card().value >= CardValue::Eight {
                    likes_hand_comments.push(format!("top pair {}", hc.get_hi_card().value));
                    *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
                } else {
                    likes_hand_comments.push(format!("top pair, <= 8 {}", hc.get_hi_card().value));
                    *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
                }
            }
        } else if p.number_above == 1 {
            if hc.get_lo_card().value >= CardValue::Ten {
                likes_hand_comments.push(format!(
                    "mid pair {} with decent kicker {}",
                    hc.get_hi_card().value,
                    hc.get_lo_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
            } else {
                likes_hand_comments.push(format!(
                    "mid pair {}; bad kicker {}",
                    hc.get_hi_card().value,
                    hc.get_lo_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
            }
        } else {
            likes_hand_comments.push(format!("3rd or worse pair {}", hc.get_hi_card().value));
            *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
        }
    } else if let Some(p) = prc.lo_pair {
        //likes_hand = max(likes_hand, LikesHandLevel::SmallBet);
        if p.number_above == 0 {
            if hc.get_lo_card().value <= CardValue::Eight {
                likes_hand_comments.push(format!(
                    "lo card is top pair {} but is small",
                    hc.get_lo_card().value
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
            } else {
                likes_hand_comments.push(format!("lo card is top pair {}", hc.get_lo_card().value));
                *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
            }
        } else if p.number_above == 1 && prc.get_num_overcards() > 0 && round == Round::Flop {
            likes_hand_comments.push(format!(
                "lo card is mid pair {} with an overcard {}",
                hc.get_lo_card().value,
                hc.get_hi_card().value
            ));
            *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
        } else if p.number_above == 1 {
            likes_hand_comments.push(format!("lo card is mid pair {}", hc.get_lo_card().value));
            *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
        } else {
            not_like_hand_comments.push(format!(
                "lo card is not top pair {}",
                hc.get_lo_card().value
            ));
        }
    }
}

fn handle_pocket_pair(
    prc: &PartialRankContainer,
    ft: &BoardTexture,
    rank: &Rank,
    _board: &Board,
    hc: &HoleCards,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    _not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    if prc.pocket_pair.is_none() {
        return;
    }
    let p = prc.pocket_pair.unwrap();
    if p.number_above == 0 {
        likes_hand_comments.push(format!("pocket overpair {}", hc.get_hi_card().value));
        *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
    } else {
        if num_in_pot == 2 {
            likes_hand_comments.push(format!("pocket underpair {}", hc.get_hi_card().value));
            *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
        } else if p.number_below == 0 {
            likes_hand_comments.push(format!("pocket underpair {}", hc.get_hi_card().value));
            *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
        } else if p.number_above >= 2 {
            likes_hand_comments.push(format!(
                "pocket pair; but with 2 above {}",
                hc.get_hi_card().value
            ));
            *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
        } else {
            likes_hand_comments.push(format!("pocket pair {}", hc.get_hi_card().value));
            *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
        }
    }

    if !ft.has_quads && !ft.has_fh && rank.get_rank_enum() >= RankEnum::FullHouse {
        likes_hand_comments.push(format!(
            "Pocket Pair FH or better {}",
            hc.get_hi_card().value
        ));
        *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
    }
}

fn likes_draws(
    prc: &PartialRankContainer,
    ft: &BoardTexture,
    board: &Board,
    hc: &HoleCards,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    let round = board.get_round().unwrap();
    if round == Round::River {
        return;
    }

    if let Some(p) = prc.flush_draw {
        if p.flush_draw_type == FlushDrawType::FlushDraw {
            if prc.has_straight_draw() {
                likes_hand_comments.push(format!("Flush & str draw {}", p.hole_card_value));
                *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
            } else {
                if prc.has_top_pair() {
                    likes_hand_comments
                        .push(format!("Flush draw {} with top pair", p.hole_card_value));
                    *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
                } else if p.hole_card_value >= CardValue::King {
                    likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
                    *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
                } else {
                    likes_hand_comments.push(format!("Flush draw {}", p.hole_card_value));
                    *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
                }
            }
        }
    }

    if let Some(p) = prc.straight_draw {
        if ft.same_suited_max_count >= 4 && num_in_pot >= 4 {
            not_like_hand_comments.push(format!(
                "4 of same suit on board: {}, not considering straight draws",
                ft.suits_with_max_count
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
            return;
        }

        if p.straight_draw_type == StraightDrawType::OpenEnded
            || p.straight_draw_type == StraightDrawType::DoubleGutShot
        {
            if num_in_pot >= 3 {
                likes_hand_comments.push(format!(
                    "Straight draw {} in multiway pot",
                    p.straight_draw_type
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
            } else if board.get_num_cards() == 3 && prc.get_num_overcards() >= 1 {
                //on flop with 1 overcard we up this
                likes_hand_comments.push(format!(
                    "Straight draw {} with 1 or more overcards",
                    p.straight_draw_type
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::LargeBet);
            } else {
                likes_hand_comments.push(format!("Straight draw {}", p.straight_draw_type));
                *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
            }
        } else {
            //gutshots
            if prc.get_num_overcards() >= 1 && hc.get_hi_card().value >= CardValue::Jack {
                likes_hand_comments.push(format!(
                    "Gutshot straight draw {} with 1 or more overcards J or better",
                    p.straight_draw_type
                ));
                *likes_hand = max(*likes_hand, LikesHandLevel::SmallBet);
            } else {
                likes_hand_comments.push(format!("Gutshot straight draw {}", p.straight_draw_type));
                *likes_hand = max(*likes_hand, LikesHandLevel::CallSmallBet);
            }
        }
        //
    }
}

fn worried_about_flushes(
    ft: &BoardTexture,
    prc: &PartialRankContainer,
    rank: &Rank,
    likes_hand: &mut LikesHandLevel,
    not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    if rank.get_rank_enum() >= RankEnum::Flush {
        return;
    }

    if ft.same_suited_max_count == 3 && !prc.flush_draw.is_some() {
        not_like_hand_comments.push(format!(
            "Worried about flushes with 3 on the board: {}",
            ft.suits_with_max_count
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ));
        *likes_hand = min(*likes_hand, LikesHandLevel::LargeBet);
    }

    if ft.same_suited_max_count == 4 && !prc.flush_draw.is_some() {
        not_like_hand_comments.push(format!(
            "Worried about flushes with 4 on the board: {}",
            ft.suits_with_max_count
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ));
        if num_in_pot >= 4 {
            *likes_hand = min(*likes_hand, LikesHandLevel::CallSmallBet);
        } else if num_in_pot == 3 {
            *likes_hand = min(*likes_hand, LikesHandLevel::SmallBet);
        } else {
            *likes_hand = min(*likes_hand, LikesHandLevel::LargeBet);
        }
    }
}

fn worried_about_straights(
    ft: &BoardTexture,
    rank: &Rank,
    likes_hand: &mut LikesHandLevel,
    not_like_hand_comments: &mut Vec<String>,
    num_in_pot: u8,
) {
    //Maybe needed to worry about better straights?
    if rank.get_rank_enum() >= RankEnum::Straight {
        return;
    }

    if ft.others_with_str8.len() > 3 {
        not_like_hand_comments.push(format!(
            "Worried about straights: {} hole cards make a straight",
            ft.others_with_str8.len()
        ));
        if num_in_pot >= 4 {
            *likes_hand = min(*likes_hand, LikesHandLevel::CallSmallBet);
        } else if num_in_pot == 3 {
            *likes_hand = min(*likes_hand, LikesHandLevel::SmallBet);
        } else {
            *likes_hand = min(*likes_hand, LikesHandLevel::LargeBet);
        }
    }
}

fn likes_made_flushes_and_straights(
    rank: &Rank,
    ft: &BoardTexture,
    prc: &PartialRankContainer,
    likes_hand: &mut LikesHandLevel,
    likes_hand_comments: &mut Vec<String>,
    not_like_hand_comments: &mut Vec<String>,
    _num_in_pot: u8,
) {
    if RankEnum::Straight == rank.get_rank_enum() {
        if ft.has_straight {
            likes_hand_comments.push(format!("Straight on board"));
        } else {
            likes_hand_comments.push(format!(
                "Made straight with {:?} other hole cards with a straight",
                ft.others_with_str8
            ));
            *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
        }
    }

    if RankEnum::Flush == rank.get_rank_enum() {
        if let Some(made_flush) = prc.made_flush {
            assert!(ft.same_suited_max_count >= 3);
            if ft.same_suited_max_count >= 4 {
                if made_flush == CardValue::Ace {
                    likes_hand_comments
                        .push(format!("Made nut flush with a good card {}", made_flush));
                    *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
                } else if made_flush <= CardValue::Ten {
                    not_like_hand_comments
                        .push(format!("Flush 4 on board, and only have {}", made_flush));
                    *likes_hand = min(*likes_hand, LikesHandLevel::CallSmallBet);
                } else {
                    likes_hand_comments.push(format!(
                        "Made decent flush with 4 on the board {}",
                        made_flush
                    ));
                }
            } else {
                //if ft.same_suited_max_count == 3 {
                if made_flush == CardValue::Ace {
                    likes_hand_comments.push(format!("Made nut flush {}", made_flush));
                    *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
                } else {
                    likes_hand_comments.push(format!("Made flush with a {}", made_flush));
                    *likes_hand = max(*likes_hand, LikesHandLevel::AllIn);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Borrow;

    use log::{debug, info};

    use crate::{
        board_eval_cache_redb::{EvalCacheReDb, ProduceFlopTexture},
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
        calc_board_texture, init_test_logger,
        monte_carlo_equity::calc_equity,
        partial_rank_cards,
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

    fn get_response<B>(hc: HoleCards, board: B, num_in_pot: u8) -> LikesHandResponse
    where
        B: Borrow<Board>,
    {
        let prc = partial_rank_cards(&hc, board.borrow().as_slice_card());

        let board_texture = calc_board_texture(board.borrow().as_slice_card());

        let hash_func = load_boomperfect_hash();

        let rank = fast_hand_eval(board.borrow().get_iter().chain(hc.get_iter()), &hash_func);

        let likes_hand_response = likes_hand(
            &prc,
            &board_texture,
            &rank,
            &board.borrow(),
            &hc,
            num_in_pot,
        )
        .unwrap();

        debug!(
            "Likes hand response: {:?}\nhttp://127.0.0.1:5173/?board={}&hero={}",
            likes_hand_response,
            board.borrow().to_string_no_spaces(),
            hc.to_string()
        );

        likes_hand_response
    }

    #[test]
    fn test_mid_pair() {
        init_test_logger();

        let hc: HoleCards = "Qs Td".parse().unwrap();

        let board: Board = "3d Ks Qc".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::SmallBet);

        let hc: HoleCards = "Ah Qs".parse().unwrap();

        let board: Board = "3d Ks Qc".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);
    }

    #[test]
    fn test_likes_made_flush() {
        init_test_logger();

        let hc: HoleCards = "Jh 2h".parse().unwrap();

        let board: Board = "Kh 4s Qh 3h Qc".parse().unwrap();

        let prc = partial_rank_cards(&hc, board.borrow().as_slice_card());

        assert_eq!(prc.made_flush, Some(CardValue::Jack));

        let board_texture = calc_board_texture(board.borrow().as_slice_card());

        assert_eq!(board_texture.same_suited_max_count, 3);

        let likes_hand_response = get_response(hc, &board, 5);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::AllIn);
    }

    #[test]
    fn test_str8_vs_flush() {
        init_test_logger();

        let hc: HoleCards = "Qs 9d".parse().unwrap();

        let board: Board = "Jc 8c 7c Th".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 2);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        let likes_hand_response = get_response(hc, &board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        let board: Board = "Jc 8c 7c Th 2c".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 2);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        //let board : Board = "Jc 8c 7c Th 2c".parse().unwrap();
        let likes_hand_response = get_response(hc, &board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::CallSmallBet);
    }

    #[test]
    fn test_likes_heads_up() {
        init_test_logger();

        let hc: HoleCards = "Ac 3d".parse().unwrap();

        let board: Board = "9c 6d 3h".parse().unwrap();

        {
            let likes_hand_response = get_response(hc, &board, 2);

            assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);
        }
        let likes_hand_response = get_response(hc, &board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::CallSmallBet);

        let hc: HoleCards = "5c 5d".parse().unwrap();

        let board: Board = "9c 6d 3h".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 2);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::SmallBet);

        let hc: HoleCards = "Ac Jd".parse().unwrap();

        let board: Board = "9c 6d 3h".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 2);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::SmallBet);
    }

    #[test]
    fn test_likes_over_card_and_str8_draw() {
        let hc: HoleCards = "Ac Td".parse().unwrap();

        let board: Board = "Jd 9s 8h".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);

        debug!("Likes hand response: {:?}", likes_hand_response);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);
    }

    #[test]
    fn test_likes_pair_in_flush() {
        init_test_logger();

        let hc: HoleCards = "8h 8d".parse().unwrap();

        let board: Board = "Ad 5d 6h Kd".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);

        debug!("Likes hand response: {:?}", likes_hand_response);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::SmallBet);

        //not a diamond
        let hc: HoleCards = "8h 8c".parse().unwrap();

        let board: Board = "Ad 5d 6h Kd".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);

        debug!("Likes hand response: {:?}", likes_hand_response);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::CallSmallBet);
    }

    #[test]
    fn test_likes_two_pair_on_paired_board() {
        init_test_logger();

        let hc: HoleCards = "Qc 4s".parse().unwrap();

        let board: Board = "Qh 9d 4c 9h".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);

        debug!("Likes hand response: {:?}", likes_hand_response);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);
    }

    #[test]
    fn test_likes_two_pair() {
        init_test_logger();

        let hc: HoleCards = "3c 5s".parse().unwrap();

        let board: Board = "Qh 3d 5c".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::AllIn);

        //With 3 and 4 of same suit on the board
        let hc: HoleCards = "As Ts".parse().unwrap();

        let board: Board = "Ah Th Jh".parse().unwrap();

        let likes_hand_response = get_response(hc, board, 4);
        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        let board: Board = "Ah Th Jh 4c".parse().unwrap();
        let likes_hand_response = get_response(hc, board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        let board: Board = "Ah Th Jh 4c 6h".parse().unwrap();
        let likes_hand_response = get_response(hc, board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::CallSmallBet);

        //Vs 1 card to a straight
        let hc: HoleCards = "Qc 5h".parse().unwrap();

        let board: Board = "Ad Qs 5d 3s 4c".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 3);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::SmallBet);

        let likes_hand_response = get_response(hc, board, 4);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::CallSmallBet);
    }

    #[test]
    fn test_straight_draw() {
        init_test_logger();

        let hc: HoleCards = "9c 8s".parse().unwrap();

        //4 to a flush
        let board: Board = "4h Th 7h Kh".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 5);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::None);

        //3 to a flush
        let board: Board = "4h Th 7h Ks".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 5);

        //Maybe too risky, but keeping for now
        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        //2 to a flush
        let board: Board = "4h Tc 7h Ks".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 5);

        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);

        //No flush possibilites
        let board: Board = "4d Tc 7h Ks".parse().unwrap();

        let likes_hand_response = get_response(hc, &board, 5);

        //This would be a good semi bluff
        assert_eq!(likes_hand_response.likes_hand, LikesHandLevel::LargeBet);
    }

    //#[test]
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    fn test_likes_hand() {
        //compares likes hand with actual equity

        /*
        cargo test test_likes_hand --release -- --nocapture
         */
        init_test_logger();

        let mut partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
            EvalCacheWithHcReDb::new().unwrap();

        let mut flop_texture_db: EvalCacheReDb<ProduceFlopTexture> = EvalCacheReDb::new().unwrap();

        let mut ranges: Vec<BoolRange> = vec![
            //We'll replace this one with the hole cards
            BoolRange::default(),
            BoolRange::default(),
            BoolRange::default(),
            BoolRange::default(),
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

                            let prc = partial_rank_db.get_put(&board, &hc, 0).unwrap();

                            let rank =
                                fast_hand_eval(board.get_iter().chain(hc.get_iter()), &hash_func);

                            let likes_hand_res =
                                likes_hand(&prc, &ft, &rank, &board, &hc, 4).unwrap();

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
