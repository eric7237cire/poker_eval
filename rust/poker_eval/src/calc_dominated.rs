use std::{cmp::{max, min}, rc::Rc, cell::RefCell, fs::File, io::Write, time::Instant};

use itertools::Itertools;
use log::info;
use poker_eval::{CardValueRange, init_logger, CardValue, Suit, Card, HoleCards, board_hc_eval_cache_redb::{ProduceMonteCarloEval, EvalCacheWithHcReDb}, Deck, Board, PokerError, pre_calc::{get_data_file_path, fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash}};

/*
cargo run --release --bin gen_hole_card_data

*/

fn main() {
    main_impl().unwrap();
}

fn main_impl() -> Result<(), PokerError> {
    init_logger();
    
    let mut check_index = 0;
    
    let mut monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
        EvalCacheWithHcReDb::new().unwrap();

    //let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

    let mut deck = Deck::new();

    let num_hands_to_simulate = 10_000;

    let mut last_output = Instant::now();

    let num_players = 4;

    let hash_func = load_boomperfect_hash();

    //Calculate minimum hands, use top pair with hi card
    let mut min_rank = Vec::new();
    for card_value in CardValueRange::new(CardValue::Two, CardValue::Ace) {
        min_rank.push(if card_value <= CardValue::Five {
            let mut board: Board = "2c 3d 4h 5s".parse()?;
            board.add_card(Card::new(card_value, Suit::Club));
            fast_hand_eval(board.get_iter(), &hash_func)
        } else  {
            //2 3 4 cv cv
            let mut board: Board = "2c 3d 4h".parse()?;
            board.add_card(Card::new(card_value, Suit::Club));
            board.add_card(Card::new(card_value, Suit::Spade));
            fast_hand_eval(board.get_iter(), &hash_func)
        }); 
    }

    for row in 0..13u8 {
        for col in 0..13 {
            
            let mut num_below_threshold = 0;
            let mut too_many_above_threshold = 0;
            
            let mut beat_second_best = 0;
            let mut was_second_best = 0;
            let mut two_way_tie = 0;

            let hi_card = CardValue::try_from(12 - min(row, col))?;
            let lo_card = CardValue::try_from(12 - max(row, col))?;
            let is_suited = col > row;
            
            let card1 = Card::new(hi_card, Suit::Club);
            let card2 = Card::new(lo_card, if is_suited {Suit::Club} else {Suit::Diamond});
            let hole_cards = HoleCards::new(card1, card2)?;
            
            for i in 0..num_hands_to_simulate {
                deck.reset();
                deck.set_used_card(card1);
                deck.set_used_card(card2);

                let mut hole_card_vec = Vec::with_capacity(num_players);
                hole_card_vec.push(hole_cards);

                for _ in 1..num_players {
                    let card1 = deck.get_unused_card()?;
                    let card2 = deck.get_unused_card()?;
                    hole_card_vec.push(HoleCards::new(card1, card2)?);
                }

                let board = deck.choose_new_board();

                //maybe eval flop/turn/river ?  for now just river

                let rankings = hole_card_vec.iter().map(|hc| {
                    fast_hand_eval(board.iter().cloned().chain(hc.get_iter()), &hash_func)
                }).collect_vec();

                //if hero doesn't have the min, skip
                if rankings[0] < min_rank[hi_card as usize] {
                    num_below_threshold += 1;
                    continue;
                }

                

                //Only 1 other 
            }
        }
    }

    Ok(())
}