use crate::{PartialRankContainer, BoardTexture, Rank, HoleCards, Card, CardValue, Round, PokerError, Board, FlushDrawType, StraightDrawType};

#[repr(u8)]
pub enum LikesHandLevel {
    None, //might even fold instead of checking
    SmallBet,
    LargeBet,
    AllIn,
}

pub struct LikesHandResponse {
    pub likes_hand: LikesHandLevel,
    pub likes_hand_comments: Vec<String>,
    pub not_like_hand_comments: Vec<String>,
}

pub fn likes_hand(
    prc: &PartialRankContainer,
    ft: &BoardTexture,
    hand_rank: &Rank,
    board: &Board,
    hc: &HoleCards,
) -> Result<LikesHandResponse, PokerError> {

    let round = board.get_round()?;

    let mut likes_hand_comments: Vec<String> = Vec::new();
    let mut not_like_hand_comments: Vec<String> = Vec::new();

    if let Some(p) = prc.lo_pair {
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
        if p.made_quads {
            likes_hand_comments.push(format!("Quads {}", hc.get_hi_card().value));
        }
        if p.made_set {
            likes_hand_comments.push(format!("Set {}", hc.get_hi_card().value));
        }
    }
    if let Some(p) = prc.pocket_pair {
        //if p.number_above == 0 {
        likes_hand_comments.push(format!("pocket pair {}", hc.get_hi_card().value));
        //}
        if p.made_set {
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
                    likes_hand_comments.push(format!("hi card overcard is ace or king with paired board {}", hc.get_hi_card().value));
                } else {
                    not_like_hand_comments.push(format!("hi card overcard is not ace or king with paired board {}", hc.get_hi_card().value));
                }
            } else {
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
        likes_hand: LikesHandLevel::None,
        likes_hand_comments,
        not_like_hand_comments,
    });
}