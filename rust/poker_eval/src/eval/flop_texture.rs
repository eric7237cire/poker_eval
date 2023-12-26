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

use std::{cmp::max, collections::HashMap};

use jammdb::{Data, Error as JammDbError, DB};
use log::trace;
use num_integer::binomial;
use serde::{Deserialize, Serialize};
// use rmps crate to serialize structs using the MessagePack format
use crate::{
    calc_cards_metrics, partial_rank_cards, rank_cards, Card, CardValue, HoleCards,
    StraightDrawType,
};
use redb::{Database, Error as ReDbError, ReadableTable, TableDefinition};
use rmp_serde::{Deserializer, Serializer};

const TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("flop_texture");

#[derive(Serialize, Deserialize)]
pub struct BoardTexture {
    // Highest same suited count, 1 is a raindbow board
    pub same_suited_max_count: u8,

    pub gaps: Vec<u8>, // a flop will have 2

    //56 on board gives straight draw to
    //43 and 78 and 4 7
    //5 7 gives straight draw to 46, 68
    //5 8 gives straight draw to 67
    pub perc_with_str8: f64,
    pub perc_with_str8_draw: f64,
    pub perc_with_gut_shot: f64,

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
        high_value_count: 0,
        med_value_count: 0,
        low_value_count: 0,
        perc_with_str8: 0.0,
        perc_with_str8_draw: 0.0,
        perc_with_gut_shot: 0.0,
    };

    let cards_metrics = calc_cards_metrics(cards);

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

    if cards_metrics.count_to_value[3] != 0 {
        texture.has_trips = true;
    }
    let pair_count = cards_metrics.count_to_value[2].count_ones();
    if pair_count >= 2 {
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

            let rank = rank_cards(&eval_cards);

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
                        num_str8_draw += 1;
                    }
                    StraightDrawType::GutShot(_) => {
                        num_gut_shot += 1;
                    }
                    StraightDrawType::DoubleGutShot => {
                        num_str8_draw += 1;
                    }
                }
            }
        }
    }

    texture.perc_with_str8 = num_str8 as f64 / total_eval as f64;
    texture.perc_with_str8_draw = num_str8_draw as f64 / total_eval as f64;
    texture.perc_with_gut_shot = num_gut_shot as f64 / total_eval as f64;

    texture
}

struct CombinatorialIndex {
    cache: HashMap<u16, u32>,
}

impl CombinatorialIndex {
    pub fn new() -> Self {
        CombinatorialIndex {
            cache: HashMap::new(),
        }
    }

    fn get_binomial(&mut self, n: u16, k: u16) -> u32 {
        let key = (n << 8) | k;
        if let Some(val) = self.cache.get(&key) {
            return *val;
        }

        let val = binomial(n as usize, k as usize) as u32;
        self.cache.insert(key, val);
        val
    }

    pub fn get_index(&mut self, cards: &[Card]) -> u32 {
        let mut cards = cards.to_vec();
        cards.sort();

        let mut index = 0;

        for i in 0..cards.len() {
            let num_possible_before: u8 = cards[i].into(); // 0 to card[i] - 1
            let dim = i + 1;
            let ncr = self.get_binomial(num_possible_before as u16, dim as u16);
            index += ncr;
        }

        index
    }

    // fn combinatorial_index(cards: &[usize]) -> usize {

    //     //we want the smallest index 1st
    //     let mut cards = cards.to_vec();
    //     cards.sort();

    //     //so if I have 7 x, I want to add
    //     //how many combinations of C(6, 2) to add

    //     let mut index = 0;

    //     for i in 0..cards.len() {
    //         //trace!("Card {} is {}", i, cards[i]);
    //         let num_possible_before = cards[i]; // 0 to card[i] - 1
    //         let dim = i+1;
    //         //Example cards 50 12 5
    //         // (50 C 3) + (12 C 2) + (5 C 1)
    //         // # of ways to choose cards 0-49 in  3 cards
    //         // # of ways to choose cards 0-11 in  2 cards
    //         // # of ways to choose cards 0-4  in  1 card
    //         let ncr = binomial(num_possible_before, dim);
    //         //trace!("Adding {} choose {} == {} to index", num_possible_before, dim, ncr);
    //         index += ncr;
    //     }

    //     // https://en.wikipedia.org/wiki/Combinatorial_number_system
    //     // https://math.stackexchange.com/questions/1227409/indexing-all-combinations-without-making-list

    //     index
    // }
}

struct FlopTextureJamDb {
    db_name: String,
    db: DB,
    c_index: CombinatorialIndex,
    cache_hits: u32,
    cache_misses: u32,
}

impl FlopTextureJamDb {
    pub fn new(db_name: &str) -> Result<Self, JammDbError> {
        let db = DB::open(db_name)?;

        {
            let tx = db.tx(true)?;
            let bucket = tx.get_bucket("flop_texture");

            if bucket.is_err() {
                tx.create_bucket("flop_texture")?;
                tx.commit()?;
            }
        }

        Ok(FlopTextureJamDb {
            db_name: db_name.to_string(),
            db,
            c_index: CombinatorialIndex::new(),
            cache_hits: 0,
            cache_misses: 0,
        })
    }

    pub fn get_put(&mut self, cards: &[Card]) -> Result<BoardTexture, JammDbError> {
        let opt = self.get(cards)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let texture = calc_board_texture(cards);
        self.cache_misses += 1;
        self.put(cards, &texture)?;

        Ok(texture)
    }

    pub fn get(&mut self, cards: &[Card]) -> Result<Option<BoardTexture>, JammDbError> {
        let tx = self.db.tx(false)?;
        let bucket = tx.get_bucket("flop_texture")?;
        let index = self.c_index.get_index(cards);
        if let Some(data) = bucket.get(index.to_be_bytes()) {
            let texture: BoardTexture = rmp_serde::from_slice(data.kv().value()).unwrap();
            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    pub fn put(&mut self, cards: &[Card], texture: &BoardTexture) -> Result<(), JammDbError> {
        let tx = self.db.tx(true)?;
        let bucket = tx.get_bucket("flop_texture")?;
        let index = self.c_index.get_index(cards);
        let texture_bytes = rmp_serde::to_vec(texture).unwrap();
        bucket.put(index.to_be_bytes(), texture_bytes)?;
        tx.commit()?;
        Ok(())
    }
}

struct FlopTextureReDb {
    db_name: String,
    db: Database,
    c_index: CombinatorialIndex,
    cache_hits: u32,
    cache_misses: u32,
}

impl FlopTextureReDb {
    pub fn new(db_name: &str) -> Result<Self, ReDbError> {
        let db = Database::create(db_name)?;

        {
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self {
            db_name: db_name.to_string(),
            db,
            c_index: CombinatorialIndex::new(),
            cache_hits: 0,
            cache_misses: 0,
        })
    }

    pub fn get_put(&mut self, cards: &[Card]) -> Result<BoardTexture, ReDbError> {
        let opt = self.get(cards)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let texture = calc_board_texture(cards);
        self.cache_misses += 1;
        self.put(cards, &texture)?;

        Ok(texture)
    }

    /*
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert("my_key", &123)?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(table.get("my_key")?.unwrap().value(), 123);
     */

    pub fn get(&mut self, cards: &[Card]) -> Result<Option<BoardTexture>, ReDbError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let index = self.c_index.get_index(cards);
        let data = table.get(index)?;
        if let Some(data) = data {
            let texture: BoardTexture = rmp_serde::from_slice(data.value()).unwrap();
            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    pub fn put(&mut self, cards: &[Card], texture: &BoardTexture) -> Result<(), ReDbError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;

            let index = self.c_index.get_index(cards);
            let texture_bytes = rmp_serde::to_vec(texture).unwrap();
            table.insert(index, texture_bytes.as_slice())?;
        }

        write_txn.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use log::info;
    use postflop_solver::card_pair_to_index;

    use crate::{init_test_logger, AgentDeck, CardVec};

    use super::*;

    #[test]
    fn test_cache_indexing() {
        init_test_logger();

        // let mut index_check = 0;
        // for card1 in 0..52 {
        //     for card2 in card1 + 1.. 52 {

        //         let idx = combinatorial_index(&vec![card1 as usize, card2 as usize]);
        //         println!("card1: {} card 2: {} == {}, {} == {} + {}",
        //         card1, card2, card_pair_to_index(card1, card2), index_check,
        //         //THis is the sum formula (51+50 / 2)
        //         card1 as usize * (101 - card1 as usize) / 2 ,
        //         card2
        //         );
        //         println!("Idx is {} but should be {}", idx, index_check);
        //         assert_eq!(idx, index_check);
        //         index_check += 1;
        //     }
        // }

        //0 X Y has 51 Choose 2 -- 1275

        let mut index_check = 0;
        // 2 1 0
        // 3 1 0
        let mut ci = CombinatorialIndex::new();
        for card1 in 0..52u8 {
            let card1_obj: Card = card1.try_into().unwrap();
            for card2 in 0..card1 {
                let card2_obj: Card = card2.try_into().unwrap();
                for card3 in 0..card2 {
                    let card3_obj: Card = card3.try_into().unwrap();
                    let idx = ci.get_index(&vec![card1_obj, card2_obj, card3_obj]);
                    // println!("{} {} {} ==> Idx is {} but should be {}",
                    //     card1, card2, card3,
                    //     idx, index_check);
                    assert_eq!(idx, index_check);
                    index_check += 1;
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        let mut ci = CombinatorialIndex::new();
        let mut index_check = 0;
        for card1 in 0..52u8 {
            let card1_obj: Card = card1.try_into().unwrap();
            for card2 in 0..card1 {
                let card2_obj: Card = card2.try_into().unwrap();
                for card3 in 0..card2 {
                    let card3_obj: Card = card3.try_into().unwrap();
                    for card4 in 0..card3 {
                        let card4_obj: Card = card4.try_into().unwrap();
                        for card5 in 0..card4 {
                            let card5_obj: Card = card5.try_into().unwrap();
                            for card6 in 0..card5 {
                                let card6_obj: Card = card6.try_into().unwrap();
                                for card7 in 0..card6 {
                                    let card7_obj: Card = card7.try_into().unwrap();

                                    // let idx = combinatorial_index(&vec![card1 as usize, card2 as usize, card3 as usize,
                                    //     card4 as usize, card5 as usize, card6 as usize, card7 as usize]);
                                    let idx = ci.get_index(&vec![
                                        card1_obj, card2_obj, card3_obj, card4_obj, card5_obj,
                                        card6_obj, card7_obj,
                                    ]);
                                    assert_eq!(idx, index_check);
                                    index_check += 1;

                                    if index_check % 1_000_000 == 0 {
                                        println!(
                                            "Index is {} cache size {}",
                                            index_check,
                                            ci.cache.len()
                                        );
                                    }

                                    assert!(index_check < 400_000_000);
                                }
                            }
                        }
                    }
                }
            }
            //println!("For card {}, we have {} combos", card1, count);
        }

        //52 choose 5 == 2_598_960 ; 47 choose 2 == 1081 ; produce = 2_810_503_360
        //52 choose 7 -- 133_784_560
        //7 choose 5 == 21
        assert_eq!(133_784_560, index_check);
    }

    // cargo test cache_perf --lib --release -- --nocapture

    #[test]
    fn test_cache_perf() {
        init_test_logger();

        let mut agent_deck = AgentDeck::new();
        let mut cards: Vec<Card> = Vec::new();
        cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
        let now = Instant::now();
        let iter_count = 1_000;
        // Code block to measure.
        {
            for _ in 0..iter_count {
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                let texture = calc_board_texture(&cards);
                agent_deck.clear_used_card(cards[1]);
                agent_deck.clear_used_card(cards[2]);
                cards.pop();
                cards.pop();
            }
        }

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        let db_name = "/home/eric/git/poker_eval/data/flop_texture.db";

        cards.clear();
        agent_deck.reset();
        //delete if exists
        //std::fs::remove_file(db_name).unwrap_or_default();

        //let mut flop_texture_db = FlopTextureJamDb::new(db_name).unwrap();

        let re_db_name = "/home/eric/git/poker_eval/data/flop_texture_re.db";
        let mut flop_texture_db = FlopTextureReDb::new(re_db_name).unwrap();
        let now = Instant::now();
        let iter_count = 1_000_000;
        // Code block to measure.
        {
            for i in 0..iter_count {
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                let texture = flop_texture_db.get_put(&cards).unwrap();
                agent_deck.clear_used_card(cards[0]);
                agent_deck.clear_used_card(cards[1]);
                agent_deck.clear_used_card(cards[2]);
                cards.pop();
                cards.pop();
                cards.pop();

                if flop_texture_db.cache_misses > 0 && flop_texture_db.cache_misses % 1000 == 0 {
                    println!("Iter {}", i);
                    info!(
                        "Cache hits {} misses {}",
                        flop_texture_db.cache_hits, flop_texture_db.cache_misses
                    );
                }
                if flop_texture_db.cache_hits > 0 && flop_texture_db.cache_hits % 100_000 == 0 {
                    println!("Iter {}", i);
                    info!(
                        "Cache hits {} misses {}",
                        flop_texture_db.cache_hits, flop_texture_db.cache_misses
                    );
                }
            }
        }

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        info!(
            "Cache hits {} misses {}",
            flop_texture_db.cache_hits, flop_texture_db.cache_misses
        );
    }

    #[test]
    fn test_board_texture() {
        let cards = CardVec::try_from("3c 2s As").unwrap().0;
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

        let cards = CardVec::try_from("Ac Ah As").unwrap().0;
        let texture = calc_board_texture(&cards);

        assert_eq!(texture.same_suited_max_count, 1);
        assert_eq!(texture.gaps.len(), 0);
        assert_eq!(texture.has_trips, true);
        assert_eq!(texture.has_pair, false);
        assert_eq!(texture.has_two_pair, false);
        assert_eq!(texture.high_value_count, 3);
        assert_eq!(texture.med_value_count, 0);
        assert_eq!(texture.low_value_count, 0);

        let cards = CardVec::try_from("Qc Kh Qd As Ks").unwrap().0;
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
    }
}
