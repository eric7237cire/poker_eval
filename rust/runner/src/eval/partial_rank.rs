use std::ops::BitOr;

use log::{trace, debug};

use crate::{calc_bitset_cards_metrics, calc_cards_metrics, count_higher, Card, CardValue, value_set_iterator, ValueSetType, BitSetCardsMetrics, rank_straight};

//for pairs, 2 pair, sets, quads, full houses
#[derive(PartialEq, Eq, Debug)]
pub struct PairFamilyRank {
    pub number_above: u8,
    pub number_below: u8,
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

    //The card that makes the straight, it's not the highest card in the straight
    GutShot(CardValue),

    OpenEnded,

    DoubleGutShot,
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

    //If we hit our straight, how many better straight draws (of any type) exist on the board
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

fn debug_print_value_set(desc: &str, value_set: ValueSetType) {
    let mut s = String::new();
    for i in 0..13 {
        if value_set[i] {
            s.push_str(CardValue::from(i).to_char().to_string().as_str());
        } else {
            //s.push_str("0");
        }
    }
    debug!("{} Value set: {}", desc, s);
}

impl PartialRankContainer {
    fn handle_pocket_pairs(&mut self, 
        hole_cards: &[Card],
        board_metrics: &BitSetCardsMetrics) {

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

            self.pocket_pair = Some(PocketPair {
                number_above,
                made_set,
                made_quads,
            });
        }
        
    }

    fn handle_draws(&mut self, 
        hole_cards: &[Card],
        hole_metrics: &BitSetCardsMetrics,
        board_metrics: &BitSetCardsMetrics) 
    {
        
        if board_metrics.value_set.count_ones() >= 5 {
            //No draws if a st8 on the board
            if rank_straight(board_metrics.value_set.data[0]).is_some() {
                return;
            }
        }
        debug!("Handle draws");

            //To know how good our draw is
        //take the board and calculate all the possible straight draws 
        //The value stored is the highest value in the straight if the draw hits

        //an open ended draw is when we have a block of 4 consecutive values
        //with a lower and higher one possible
        let mut all_draws = ValueSetType::default();

        //So we do a rolling count of 1s in a row on the value set
        //Note though there are some cases
        // [0] 1 1 1 1 [0] is the normal one
        // [0] 1 1 1 1 [1] means there are no open ended draws, since we have a straight
        // [1] 1 1 1 1 [0] means there are no open ended draws, since we have a straight
        // But we do have a gut shot draw to a higher straight in the 3rd case

        // The value of the draw is the highest value in the draw, so for
        // [0] 1 1 1 (1) ([0]) is the 2 values we add are in ()

        // for vs_it in value_set_iterator(board_metrics.value_set, 4, CardValue::Two, CardValue::King) {
            
        //     assert!(vs_it.value_count <= 4);

        //     //Basically we need to use at least one max 2 of the hole cards
        //     //So if the board already has the entire block set, there is no draw here
        //     if vs_it.value_count >= 2 && vs_it.value_count < 4 && 
        //     !vs_it.is_set_on_either_side(board_metrics.value_set)
        //     {
        //         all_draws.set(vs_it.window_stop.next_card() as usize, true);
        //         all_draws.set(vs_it.window_start.prev_card() as usize, true);
        //     } 
        // }

        //A gut shot is to a specific draw, so we look at all blocks of 5
        //and if the board has at least 2, but <= 3 (since we need a hole card to make it a draw)
        //If we had 4 on the board, then the 5th card would make a straight on the board without hole cards

        //start with wheel
        
        for vs_it in value_set_iterator(board_metrics.value_set, 5, CardValue::Ace, CardValue::Ace) {
            
            assert!(vs_it.value_count <= 5);

            //Basically we need to use at least one max 2 of the hole cards
            //So if the board already has the entire block set, there is no draw here
            if vs_it.value_count >= 2 && vs_it.value_count < 4  {
                all_draws.set(vs_it.window_stop.into(), true);
            } 
        }


        //Ok, now we need to figure out, do *we* have a draw

        //Last one wins, we only keep best draw

        //Count how many straights we could make with one card, once that is done we can classify the draw

        let combined_value_set = board_metrics.value_set.bitor(hole_metrics.value_set);

        //Stores the highest value in the straight draws that we have
        let mut hero_draws = ValueSetType::default();

        //Stores the card that would actually make the straight
        let mut hero_card_needed = ValueSetType::default();

        debug_print_value_set("Hero cards", hole_metrics.value_set);
        debug_print_value_set("Board cards", board_metrics.value_set);
        debug_print_value_set("Combined cards", combined_value_set);
        
        for (vs_it, bh_it) in 
            value_set_iterator(board_metrics.value_set, 5, CardValue::Ace, CardValue::Ace).
            zip(
                value_set_iterator(combined_value_set, 5, CardValue::Ace, CardValue::Ace)) {
            
            assert!(vs_it.value_count <= 5);

            //Same conditions except we also need the combined one to be 4
            if vs_it.value_count >= 2 && vs_it.value_count < 4 
                && bh_it.value_count == 4 //missing exactly 1 card, a gut shot
            {            
                hero_draws.set(vs_it.window_stop.into(), true);

                //Because the beginning can be the ace we test that specifically

                if !combined_value_set[bh_it.window_start as usize] {
                    let card_needed = bh_it.window_start;
                    trace!("To make a straight to {:?} we need a {:?}, first card in st8", bh_it.window_stop as CardValue, CardValue::from(card_needed));
                    hero_card_needed.set(card_needed.into(), true);
                } else {

                    let first_zero = combined_value_set[bh_it.window_start.next_card().into()..=bh_it.window_stop.into()].first_zero().unwrap();

                    //need to add 
                    let card_needed = bh_it.window_start.next_card() as usize + first_zero;
                    trace!("To make a straight to {:?} we need a {:?}", bh_it.window_stop as CardValue, CardValue::from(card_needed));
                    hero_card_needed.set(card_needed, true);
                
                }
                // self.straight_draw = Some(StraightDraw {
                //     straight_draw_type: StraightDrawType::GutShot,
                //     number_above: count_higher(all_draws, vs_it.window_stop.into())
                // });
            } 
        }

        let num_gut_shots = hero_card_needed.count_ones();
        assert!(num_gut_shots <= 2);

        debug_print_value_set("Hero draw values", hero_draws);

        if num_gut_shots == 1 {
            //If 1 card makes 2 straights, we want the best one
            let hi = hero_draws.last_one().unwrap();
            let card_needed = hero_card_needed.first_one().unwrap();
            trace!("A gut shot that needs {:?} to make a straight to {:?}", CardValue::from(card_needed), CardValue::from(hi));
            self.straight_draw = Some(StraightDraw {
                straight_draw_type: StraightDrawType::GutShot(card_needed.into()),
                number_above: count_higher(all_draws, hi)
            });
        } else if num_gut_shots == 2 {
            //if the diff is one, its an open ended draw, more than that it's a double gut shot
            let hi = hero_draws.last_one().unwrap();
            let lo = hero_draws.first_one().unwrap();
            
            if hi - lo == 1 {
                self.straight_draw = Some(StraightDraw {
                    straight_draw_type: StraightDrawType::OpenEnded,
                    number_above: count_higher(all_draws, hi )
                });
            } else {
                self.straight_draw = Some(StraightDraw {
                    straight_draw_type: StraightDrawType::DoubleGutShot,
                    number_above: count_higher(all_draws, hi)});
                
            }
        }
        
        // for (vs_it, bh_it) in 
        //     value_set_iterator(board_metrics.value_set, 4, CardValue::Two, CardValue::King).
        //     zip(
        //         value_set_iterator(combined_value_set, 4, CardValue::Two, CardValue::King)) {
            
        //     assert!(vs_it.value_count <= 4);

        //     //Same conditions except we also need the combined one to be 4
        //     if vs_it.value_count >= 2 && vs_it.value_count < 4 
        //         && !vs_it.is_set_on_either_side(board_metrics.value_set) 
        //         && bh_it.value_count == 4 
        //         && !bh_it.is_set_on_either_side(combined_value_set) {
                
        //         self.straight_draw = Some(StraightDraw {
        //             straight_draw_type: StraightDrawType::OpenEndedDraw,
        //             number_above: count_higher(all_draws, vs_it.window_stop.next_card().into())
        //         });
        //     } 
        // }
    }

    fn handle_pairs(&mut self, 
        hole_cards: &[Card],
        hole_metrics: &BitSetCardsMetrics,
        board_metrics: &BitSetCardsMetrics) 
    {
        if hole_cards[0].value != hole_cards[1].value {
            let higher_hole_value = if hole_cards[0].value > hole_cards[1].value {
                hole_cards[0].value
            } else {
                hole_cards[1].value
            };
            let lower_hole_value = if hole_cards[0].value < hole_cards[1].value {
                hole_cards[0].value
            } else {
                hole_cards[1].value
            };
    
            let is_paired_hi = board_metrics.count_to_value[1][higher_hole_value as usize];
            let is_paired_lo = board_metrics.count_to_value[1][lower_hole_value as usize];
    
            let num_above_hi = count_higher(board_metrics.count_to_value[1], higher_hole_value.into());
            let num_above_lo = count_higher(board_metrics.count_to_value[1], lower_hole_value.into());
    
            //This is actually counting non paired items on the board, so these are comparing potential enemy pairs
            let pairs_on_board = board_metrics.count_to_value[1].count_ones() as u8;
    
            if is_paired_hi && is_paired_lo {
                //Total number of 2 pairs is singles choose 2 
                self.two_pair = Some(PairFamilyRank {
                    //These are wrong for now
                    number_above: num_above_hi,
                    number_below: num_above_lo,
                });
            } else if is_paired_hi {
                self.pair = Some(PairFamilyRank {
                    number_above: num_above_hi,
                    number_below: pairs_on_board - 1 - num_above_hi,
                });
            } else if is_paired_lo {
                self.pair = Some(PairFamilyRank {
                    number_above: num_above_lo,
                    number_below: pairs_on_board - 1 - num_above_lo,
                });
            } else {
                
            }
        }
    }
}

pub fn partial_rank_cards(hole_cards: &[Card], board: &[Card]) -> PartialRankContainer {
    let mut partial_ranks: PartialRankContainer = Default::default();

    let board_metrics = calc_bitset_cards_metrics(board);
    let hole_metrics = calc_bitset_cards_metrics(hole_cards);

    assert_eq!(2, hole_cards.len());

    //Handle pocket pairs
    partial_ranks.handle_pocket_pairs(hole_cards, &board_metrics);
    
    //draws
    partial_ranks.handle_draws(hole_cards, &hole_metrics, &board_metrics);
    
    // Calculate pairs
    partial_ranks.handle_pairs(hole_cards, &hole_metrics, &board_metrics);

    partial_ranks
}


#[cfg(test)]
mod tests {

    use crate::cards_from_string;

    use super::*;

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();
    }

    #[test]
    fn test_partial_ranks() {

        init();

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
                //Gut shot to A 2 3 4 5
                straight_draw_type: StraightDrawType::GutShot(CardValue::Four),
                //Draw to 2 3 4 5 6
                //Draw to [3] 4 [5] 6 [7]
                //Draw to 4 [5] 6 [7] 8
                //Draw to [5] 6 [7] 8 9
                //Gutshot to 6 [7] 8 9 [T] 
                //Gutshot to [7] 8 9 [T] J
                //Gutshot Draw to [T] J Q K [A]
                number_above: 7
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, Some(PairFamilyRank { number_above: 0, number_below: 4 }));
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
                //Gut shot to [3] [4] 5* 6 [7]
                //Another gut shot to
                //A 2 [3] [4] 5* but that one does not count
                straight_draw_type: StraightDrawType::GutShot(CardValue::Five),
                //include made straights too, 5 6
                //other better gut shots made with 5 8, 6 8, 2 5, 2 6 
                //[4] 5 6 [7] 8
                //3 4 5 6 7
                
                number_above: 1
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
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
        assert_eq!(prc.pair, None);
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
                straight_draw_type: StraightDrawType::GutShot(CardValue::Nine),
                //T J Q K A -- This doesn't count because it puts a str8 on the board
                //9 T J [Q] [K] -- Also doesn't count 
                //8 9 [T] [J] [Q]
                //7 8 9* [T] [J]
                //Everything that beats us are made straights, not draws
                number_above: 0
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
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
                straight_draw_type: StraightDrawType::DoubleGutShot,
                //there is one better gut shot, but we only count better open ended draws
                number_above: 0
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
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
                straight_draw_type: StraightDrawType::OpenEnded,
                //we are drawing to 6 7 8 9 T
                //but have an open ended draw 8 T that is better
                number_above: 1
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("7c Kh");
        let board_cards = cards_from_string("9s Jc Td");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::DoubleGutShot,
                //we are drawing to 9 T J Q K and 7 8 9 T J
                number_above: 1
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);

        let hole_cards = cards_from_string("8c 6h");
        let board_cards = cards_from_string("4s Td 7c");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            Some(StraightDraw {
                straight_draw_type: StraightDrawType::DoubleGutShot,
                //we are drawing to 6 [7] 8 9* [T] and [4] 5* 6 [7] 8
                //better straight draws are [7] 8 9 [T] J
                number_above: 1
            })
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, None);
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);


        //Normal 2 pair
        let hole_cards = cards_from_string("6c 8h");
        let board_cards = cards_from_string("6s 8c 2d 9h");
        let prc = partial_rank_cards(&hole_cards, &board_cards);

        assert_eq!(prc.flush_draw, None);
        assert_eq!(
            prc.straight_draw,
            None
        );
        assert_eq!(prc.pocket_pair, None);
        assert_eq!(prc.pair, None);
        assert_eq!(prc.two_pair, Some(PairFamilyRank { 
            number_above: 3, //9 8, 9 6, 9 2 
            number_below: 1 //6 2
        }));
        assert_eq!(prc.unpaired_higher_card, None);
        assert_eq!(prc.unpaired_lower_card, None);
    }
}
