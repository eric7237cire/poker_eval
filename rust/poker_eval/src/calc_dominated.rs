use std::{cmp::{max, min}};

use itertools::Itertools;
use log::{info};
use poker_eval::{CardValueRange, init_logger, CardValue, Suit, Card, HoleCards, Deck, Board, PokerError, pre_calc::{fast_eval::fast_hand_eval, perfect_hash::load_boomperfect_hash, rank::{RankEnum}}, BoolRange, NUM_RANK_FAMILIES};

/*
cargo run --release --bin calc_dominated

*/

fn main() {
    main_impl().unwrap();
}

fn main_impl() -> Result<(), PokerError> {
    init_logger();
    
    

    //let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

    let mut deck = Deck::new();

    let num_hands_to_simulate = 100_000;

    let num_players = 5;

    let hash_func = load_boomperfect_hash();

    //let range_to_print: BoolRange = "32o, 43o, 54o, 65o, 76o, 87o, 98o, T9o, JTo, QJo, KQo".parse()?;
    //let range_to_print: BoolRange = "KQo".parse()?;
    //let range_to_print: BoolRange = "87s, 98s, T9s, JTs".parse()?;
    let range_to_print: BoolRange = "A2s, K2s, Q2s, J2s, T2s, 92s, 82s, 72s, 62s, 52s, 43s".parse()?;

    //let rank_families_to_print = [RankEnum::TwoPair, RankEnum::ThreeOfAKind, RankEnum::Straight, RankEnum::Flush];
    let rank_families_to_print = [RankEnum::Flush];

    //Calculate minimum hands, use top pair with hi card
    let mut min_rank = Vec::new();
    for card_value in CardValueRange::new(CardValue::Two, CardValue::Ace) {
        min_rank.push(if card_value <= CardValue::Five {
            let mut board: Board = "2c 3d 4h 5s".parse()?;
            board.add_card(Card::new(card_value, Suit::Club))?;
            fast_hand_eval(board.get_iter(), &hash_func)
        } else  {
            //2 3 4 cv cv
            let mut board: Board = "2c 3d 4h".parse()?;
            board.add_card(Card::new(card_value, Suit::Club))?;
            board.add_card(Card::new(card_value, Suit::Spade))?;
            fast_hand_eval(board.get_iter(), &hash_func)
        }); 
    }

    //Set min rank to lowest 2 pair possible
    // for i in 0..NUMBER_OF_RANKS {
    //     let board: Board = "2c 2d 3h 3d 4s".parse()?;            
    //     let rank = fast_hand_eval(board.get_iter(), &hash_func);
    //     min_rank[i] = rank;
    // }

    // //Set min rank to lowest 2 pair possible
    // for i in 0..NUMBER_OF_RANKS {
    //     let board: Board = "Ad 2c 3h 4s 5c".parse()?;            
    //     let rank = fast_hand_eval(board.get_iter(), &hash_func);
    //     min_rank[i] = rank;
    // }

    for row in 0..13u8 {
        for col in 0..13 {
            
            //heros rank was below threshold
            let mut num_below_threshold = 0;
            let mut too_many_above_threshold = 0;
            
            let mut beat_second_best = 0;
            let mut was_second_best = 0;
            let mut two_way_tie = 0;

            //both hero and villian had same rank family (e.g both pairs, both trips)
            let mut was_same_family = 0;
            //won, lost, tie
            let mut same_family_stats = vec![0; NUM_RANK_FAMILIES];
            let mut win_family_stats = vec![0; NUM_RANK_FAMILIES];
            let mut tie_family_stats = vec![0; NUM_RANK_FAMILIES];
            let mut lose_family_stats = vec![0; NUM_RANK_FAMILIES];
            

            let hi_card = CardValue::try_from(12 - min(row, col))?;
            let lo_card = CardValue::try_from(12 - max(row, col))?;
            let is_suited = col > row;
            
            let card1 = Card::new(hi_card, Suit::Club);
            let card2 = Card::new(lo_card, if is_suited {Suit::Club} else {Suit::Diamond});
            let hole_cards = HoleCards::new(card1, card2)?;

            if !range_to_print.data[hole_cards.to_range_index()] {
                continue;
            }
            
            for _ in 0..num_hands_to_simulate {
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

                let is_above_threshold = rankings.iter().map(|r| {
                    r >= &min_rank[hi_card as usize]
                }).collect_vec();

                //if hero doesn't have the min, skip
                if !is_above_threshold[0] {
                    num_below_threshold += 1;
                    continue;
                }

                //allow 3 people above 
                if is_above_threshold.iter().filter(|b| **b).count() > 3 {
                    too_many_above_threshold += 1;
                    continue;
                }

                let other_pos = is_above_threshold.iter().skip(1).position(|b| *b);

                if other_pos.is_none() {
                    continue;
                }

                let hero_rank = &rankings[0];
                let other_guy_index = other_pos.unwrap() + 1;
                let other_guy_rank = &rankings[other_guy_index];
                
                if other_guy_rank > hero_rank {
                    was_second_best += 1;
                } else if other_guy_rank == hero_rank {
                    two_way_tie += 1;
                } else {
                    beat_second_best += 1;
                }

                if other_guy_rank.get_rank_enum() == hero_rank.get_rank_enum() {
                    was_same_family += 1;
                    let family_index: usize = hero_rank.get_rank_enum() as u8 as usize;
                    same_family_stats[family_index] += 1;

                    if other_guy_rank > hero_rank {
                        lose_family_stats[family_index] += 1;

                        // let old_rank1 = rank_cards(board.iter().cloned().chain(hole_cards.get_iter()));
                        // let old_rank2 = rank_cards(board.iter().cloned().chain(hole_card_vec[other_guy_index].get_iter()));
                        // debug!("\nBoard: {}\n{} with {}\nOther guys cards: {} with {}", 
                        //     Board::new_from_cards(&board),
                        //     &hole_cards, &hole_card_vec[other_guy_index],
                        //     old_rank1.print_winning(&board.iter().cloned().chain(hole_cards.get_iter()).collect_vec()),
                        //     old_rank2.print_winning(&board.iter().cloned().chain(hole_card_vec[other_guy_index].get_iter()).collect_vec()),
                        // )
                    } else if other_guy_rank == hero_rank {
                        tie_family_stats[family_index] += 1;
                    } else {
                        win_family_stats[family_index] += 1;
                    }   
                }
                

            }


            if !range_to_print.data[hole_cards.to_range_index()] {
                continue;
            }

            let mut dv = Vec::new();
            // dv.push( ("% below threshold", num_below_threshold as f64 / num_hands_to_simulate as f64) );
            // dv.push( ("% too many above threshold", too_many_above_threshold as f64 / num_hands_to_simulate as f64) );
            
            // dv.push( ("% was second best".to_string(), was_second_best as f64 / num_hands_to_simulate as f64) );
            // dv.push( ("% beat second best".to_string(), beat_second_best as f64 / num_hands_to_simulate as f64) );
            // dv.push( ("% two way tie".to_string(), two_way_tie as f64 / num_hands_to_simulate as f64) );

            let sum_heads_up = was_second_best + beat_second_best + two_way_tie;
            
            dv.push( ("% 2 or 3 with good hands".to_string(), sum_heads_up as f64 / num_hands_to_simulate as f64) );
            dv.push( ("% beat second best HU".to_string(), beat_second_best as f64 / sum_heads_up as f64) );
            dv.push( ("% two way tie HU".to_string(), two_way_tie as f64 / sum_heads_up as f64) );
            dv.push( ("% was second best HU".to_string(), was_second_best as f64 / sum_heads_up as f64) );
            
            dv.push( ("% was same family".to_string(), was_same_family as f64 / num_hands_to_simulate as f64) );

            for fs in rank_families_to_print.iter() {
                let fs_index: usize = (*fs) as u8 as usize;
                dv.push(("".to_string(), 0.0));
                dv.push( (format!("% {}", fs), 
                    same_family_stats[fs_index] as f64 / num_hands_to_simulate as f64) );
                dv.push( (format!("% won {}", fs),
                 win_family_stats[fs_index] as f64 / same_family_stats[fs_index] as f64) );
                dv.push( (
                    format!("% tie {}", fs), tie_family_stats[fs_index] as f64 / same_family_stats[fs_index] as f64) );
                dv.push( (
                    format!("% lost {}", fs), lose_family_stats[fs_index] as f64 / same_family_stats[fs_index] as f64) );
            }

            let largest_width = dv.iter().map(|(s, _)| s.len()).max().unwrap();

            //print 80 *
            info!("{}", "*".repeat(80));
            info!("For hole cards: {}", hole_cards.to_simple_range_string());

            for (s, v) in dv {
                info!("{:width$}: {:.2}%", s, v * 100.0, width = largest_width);
            }
        }
    }

    Ok(())
}