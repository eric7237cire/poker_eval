use redb::{Database, Error as ReDbError, ReadTransaction, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};

use crate::{calc_board_texture, BoardTexture, Card, CombinatorialIndex};

//u32 is usually  enough
//In the worst case we have 5 cards * 2 cards
//52 choose 5 is 2,598,960 * 47 choose 2 is 1,081 == 2,810,503,360 with fits in a u32
//To do the above though needs an ordering, which is a pain
// need 22 bits for 52 choose 5
// need 11 bits for 52 choose 2

pub const FLOP_TEXTURE_PATH: &str = "/home/eric/git/poker_eval/data/flop_texture_re.db";

const TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("eval_cache");

pub trait ProduceEvalResult<R> {
    fn produce_eval_result(cards: &[Card]) -> R;
}

//P is producer type
//R is result
//K is the key type
pub struct EvalCacheReDb<P, R> {
    db: Database,
    c_index: CombinatorialIndex,
    pub cache_hits: u32,
    pub cache_misses: u32,

    //We don't actually need an instance
    phantom1: std::marker::PhantomData<P>,
    phantom2: std::marker::PhantomData<R>,
}

impl<P, R> EvalCacheReDb<P, R>
where
    P: ProduceEvalResult<R>,
    R: Serialize + DeserializeOwned,
{
    //each different struct should get its own db path
    pub fn new(db_name: &str) -> Result<Self, ReDbError> {
        let db = Database::create(db_name)?;
        {
            //Make sure table exists
            let write_txn = db.begin_write()?;
            {
                let _table = write_txn.open_table(TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self {
            db,
            cache_hits: 0,
            cache_misses: 0,
            phantom1: std::marker::PhantomData,
            phantom2: std::marker::PhantomData,
            c_index: CombinatorialIndex::new(),
        })
    }

    pub fn get_put(&mut self, cards: &[Card]) -> Result<R, ReDbError> {
        let index = self.c_index.get_index(cards);

        let opt = self.get(index)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let result = P::produce_eval_result(cards);
        self.cache_misses += 1;

        self.put(index, &result)?;

        Ok(result)
    }

    fn get(&mut self, index: u32) -> Result<Option<R>, ReDbError> {
        let read_txn: ReadTransaction = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let data = table.get(index)?;
        if let Some(data) = data {
            //let texture: BoardTexture = rmp_serde::from_slice(data.value()).unwrap();
            let texture: R = bincode::deserialize(&data.value()).unwrap();

            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self, index: u32, result: &R) -> Result<(), ReDbError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            //let texture_bytes = rmp_serde::to_vec(texture).unwrap();
            let texture_bytes: Vec<u8> = bincode::serialize(&result).unwrap();

            table.insert(index, texture_bytes.as_slice())?;
        }

        write_txn.commit()?;
        Ok(())
    }
}

pub struct ProduceFlopTexture {}

impl ProduceFlopTexture {
    pub fn new() -> Self {
        ProduceFlopTexture {}
    }
}

impl ProduceEvalResult<BoardTexture> for ProduceFlopTexture {
    fn produce_eval_result(cards: &[Card]) -> BoardTexture {
        calc_board_texture(cards)
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use log::info;

    use crate::{
        board_eval_cache_redb::{EvalCacheReDb, FLOP_TEXTURE_PATH},
        init_test_logger, Deck,
    };

    use super::*;

    // cargo test cache_perf --lib --release -- --nocapture

    //a bit slow
    #[test]
    fn test_cache_perf() {
        init_test_logger();

        let mut agent_deck = Deck::new();
        let mut cards: Vec<Card> = Vec::new();

        cards.clear();
        agent_deck.reset();
        //delete if exists
        //std::fs::remove_file(db_name).unwrap_or_default();

        //let mut flop_texture_db = FlopTextureJamDb::new(db_name).unwrap();

        //let mut flop_texture_db = FlopTextureReDb::new(re_db_name).unwrap();
        let mut flop_texture_db: EvalCacheReDb<ProduceFlopTexture, _> =
            EvalCacheReDb::new(FLOP_TEXTURE_PATH).unwrap();
        let now = Instant::now();
        let iter_count = 100_000;
        // Code block to measure.
        {
            for i in 0..iter_count {
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                cards.push(agent_deck.get_unused_card().unwrap().try_into().unwrap());
                let _texture = flop_texture_db.get_put(&cards).unwrap();
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
}
