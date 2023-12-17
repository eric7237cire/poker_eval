use crate::{hand_table::*, card_to_string, Card};

#[derive(Clone, Copy, Default, Debug)]
pub struct Hand {
    pub cards: [usize; 7],
    pub num_cards: usize,
}

#[inline]
fn keep_n_msb(mut x: i32, n: i32) -> i32 {
    let mut ret = 0;
    for _ in 0..n {
        let bit = 1 << (x.leading_zeros() ^ 31);
        x ^= bit;
        ret |= bit;
    }
    ret
}

#[inline]
fn find_straight(rankset: i32) -> i32 {
    const WHEEL: i32 = 0b1_0000_0000_1111;
    let is_straight = rankset & (rankset << 1) & (rankset << 2) & (rankset << 3) & (rankset << 4);
    if is_straight != 0 {
        keep_n_msb(is_straight, 1)
    } else if (rankset & WHEEL) == WHEEL {
        1 << 3
    } else {
        0
    }
}

impl Hand {
    #[inline]
    pub fn new() -> Hand {
        Hand::default()
    }

    #[inline]
    pub fn add_card(&self, card: usize) -> Hand {
        let mut hand = *self;
        hand.cards[hand.num_cards] = card;
        hand.num_cards += 1;
        hand
    }

    #[inline]
    pub fn from_hole_cards(card1: Card, card2: Card) -> Hand {
        Hand {
            cards: [card1 as usize, card2 as usize, 0, 0, 0, 0, 0],
            num_cards: 2,
        }
    }

    #[inline]
    pub fn contains(&self, card: usize) -> bool {
        self.cards[0..self.num_cards].contains(&card)
    }

    #[inline]
    pub fn evaluate(&self) -> u16 {
        HAND_TABLE.binary_search(&self.evaluate_internal()).unwrap() as u16
    }

    pub fn evaluate_internal(&self) -> i32 {
        assert_eq!(7, self.num_cards);
        
        let mut rankset = 0i32;
        let mut rankset_suit = [0i32; 4];
        let mut rankset_of_count = [0i32; 5];
        let mut rank_count = [0i32; 13];

        let mut c_idx = 0;
        for &card in &self.cards {
            let rank = card / 4;
            assert!(rank < 13);

            let suit = card % 4;
            rankset |= 1 << rank;
            rankset_suit[suit] |= 1 << rank;
            rank_count[rank] += 1;

            if rank_count[rank] > 4 {
                println!("Card count {}", self.num_cards);
                for i in 0..self.num_cards {
                    println!("Card {}={}", self.cards[i],
                    card_to_string(self.cards[i] as u8).unwrap());
                }
            }
            assert!(rank_count[rank] <= 4);

            c_idx += 1;
            if c_idx == self.num_cards {
                break;
            }
        }

        for rank in 0..13 {
            rankset_of_count[rank_count[rank] as usize] |= 1 << rank;
        }

        let mut flush_suit: i32 = -1;
        for suit in 0..4 {
            if rankset_suit[suit as usize].count_ones() >= 5 {
                flush_suit = suit;
            }
        }

        let is_straight = find_straight(rankset);

        if flush_suit >= 0 {
            let is_straight_flush = find_straight(rankset_suit[flush_suit as usize]);
            if is_straight_flush != 0 {
                // straight flush
                (8 << 26) | is_straight_flush
            } else {
                // flush
                (5 << 26) | keep_n_msb(rankset_suit[flush_suit as usize], 5)
            }
        } else if rankset_of_count[4] != 0 {
            // four of a kind
            let remaining = keep_n_msb(rankset ^ rankset_of_count[4], 1);
            (7 << 26) | (rankset_of_count[4] << 13) | remaining
        } else if rankset_of_count[3].count_ones() == 2 {
            // full house
            let trips = keep_n_msb(rankset_of_count[3], 1);
            let pair = rankset_of_count[3] ^ trips;
            (6 << 26) | (trips << 13) | pair
        } else if rankset_of_count[3] != 0 && rankset_of_count[2] != 0 {
            // full house
            let pair = keep_n_msb(rankset_of_count[2], 1);
            (6 << 26) | (rankset_of_count[3] << 13) | pair
        } else if is_straight != 0 {
            // straight
            (4 << 26) | is_straight
        } else if rankset_of_count[3] != 0 {
            // three of a kind
            let remaining = keep_n_msb(rankset_of_count[1], 2);
            (3 << 26) | (rankset_of_count[3] << 13) | remaining
        } else if rankset_of_count[2].count_ones() >= 2 {
            // two pair
            let pairs = keep_n_msb(rankset_of_count[2], 2);
            let remaining = keep_n_msb(rankset ^ pairs, 1);
            (2 << 26) | (pairs << 13) | remaining
        } else if rankset_of_count[2] != 0 {
            // one pair
            let remaining = keep_n_msb(rankset_of_count[1], 3);
            (1 << 26) | (rankset_of_count[2] << 13) | remaining
        } else {
            // high card
            keep_n_msb(rankset, 5)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //this test is a bit long
    #[allow(dead_code)]
    fn test_all_hands() {
        let mut appeared = vec![false; HAND_TABLE.len()];
        let mut counter = [0; 9];

        for i in 0..52 {
            let hand = Hand::new().add_card(i);
            for j in (i + 1)..52 {
                let hand = hand.add_card(j);
                for k in (j + 1)..52 {
                    let hand = hand.add_card(k);
                    for m in (k + 1)..52 {
                        let hand = hand.add_card(m);
                        for n in (m + 1)..52 {
                            let hand = hand.add_card(n);
                            for p in (n + 1)..52 {
                                let hand = hand.add_card(p);
                                for q in (p + 1)..52 {
                                    let hand = hand.add_card(q);
                                    let raw_value = hand.evaluate_internal();
                                    let index_result = HAND_TABLE.binary_search(&raw_value);
                                    assert!(index_result.is_ok());
                                    appeared[index_result.unwrap()] = true;
                                    counter[(raw_value >> 26) as usize] += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(appeared.iter().all(|&x| x));
        assert_eq!(counter[8], 41_584); // straight flush
        assert_eq!(counter[7], 224_848); // four of a kind
        assert_eq!(counter[6], 3_473_184); // full house
        assert_eq!(counter[5], 4_047_644); // flush
        assert_eq!(counter[4], 6_180_020); // straight
        assert_eq!(counter[3], 6_461_620); // three of a kind
        assert_eq!(counter[2], 31_433_400); // two pair
        assert_eq!(counter[1], 58_627_800); // one pair
        assert_eq!(counter[0], 23_294_460); // high card

    }
}
