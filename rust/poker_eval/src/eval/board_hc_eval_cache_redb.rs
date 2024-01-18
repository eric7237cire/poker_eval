use log::info;
use redb::{Database, Error as ReDbError, ReadTransaction, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    board_eval_cache_redb::{get_data_path, EvalCacheEnum},
    monte_carlo_equity::calc_equity_vs_random,
    partial_rank_cards, Board, Card, HoleCards, PartialRankContainer,
};

use crate::game::core::Round;

//u32 is usually  enough
//In the worst case we have 5 cards * 2 cards
//52 choose 5 is 2,598,960 * 47 choose 2 is 1,081 == 2,810,503,360 with fits in a u32
//To do the above though needs an ordering, which is a pain
// need 22 bits for 52 choose 5
// need 11 bits for 52 choose 2

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("eval_cache");

pub trait ProduceEvalWithHcResult {
    type Result;

    fn produce_eval_result(cards: &[Card], hole_cards: &HoleCards, num_players: u8)
        -> Self::Result;

    fn get_cache_name() -> EvalCacheEnum;
}

//P is producer type
//R is result
//K is the key type
pub struct EvalCacheWithHcReDb<P> {
    db: Database,
    pub cache_hits: u32,
    pub cache_misses: u32,

    phantom1: std::marker::PhantomData<P>,
}

impl<P> EvalCacheWithHcReDb<P>
where
    P: ProduceEvalWithHcResult,
    P::Result: Serialize + DeserializeOwned,
{
    //each different struct should get its own db path
    pub fn new() -> Result<Self, ReDbError> {
        let db_name = get_data_path(P::get_cache_name());
        info!("Opening db {:?}", db_name);
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
        })
    }

    pub fn get_put(
        &mut self,
        cards: &Board,
        hole_cards: &HoleCards,
        num_players: u8,
    ) -> Result<P::Result, ReDbError> {
        let index = cards.get_precalc_index().unwrap();

        let mut index_bytes: [u8; 7] = [0; 7];
        // Packing the u32 into the first 4 bytes of the array
        index_bytes[0] = (index >> 24) as u8; // Extracts the first byte
        index_bytes[1] = (index >> 16) as u8; // Extracts the second byte
        index_bytes[2] = (index >> 8) as u8; // Extracts the third byte
        index_bytes[3] = index as u8; // Extracts the fourth byte
        index_bytes[4] = hole_cards.hi_card().into();
        index_bytes[5] = hole_cards.lo_card().into();
        index_bytes[6] = num_players;

        let opt = self.get(&index_bytes)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let result = P::produce_eval_result(cards.as_slice_card(), hole_cards, num_players);
        self.cache_misses += 1;

        self.put(&index_bytes, &result)?;

        Ok(result)
    }

    fn get(&mut self, index: &[u8]) -> Result<Option<P::Result>, ReDbError> {
        let read_txn: ReadTransaction = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let data = table.get(index)?;
        if let Some(data) = data {
            //let texture: BoardTexture = rmp_serde::from_slice(data.value()).unwrap();
            let texture: P::Result = bincode::deserialize(&data.value()).unwrap();

            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self, index: &[u8], result: &P::Result) -> Result<(), ReDbError> {
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

pub struct ProducePartialRankCards {}

//Link the generic cache db with the partial rank cards function
impl ProduceEvalWithHcResult for ProducePartialRankCards {
    type Result = PartialRankContainer;

    fn produce_eval_result(
        board: &[Card],
        hole_cards: &HoleCards,
        num_players: u8,
    ) -> PartialRankContainer {
        //Num players has no effect, to make sure we aren't bloating the cache we enforce it is 0
        assert_eq!(num_players, 0);
        partial_rank_cards(&hole_cards, board)
    }

    fn get_cache_name() -> EvalCacheEnum {
        EvalCacheEnum::PartialRank
    }
}

//Link with monte carlo evaluation function

pub struct ProduceMonteCarloEval {}

const NUM_SIMULATIONS: usize = 10_000;

impl ProduceEvalWithHcResult for ProduceMonteCarloEval {
    type Result = f64;

    //num_players -- This is indcluing the hero
    fn produce_eval_result(cards: &[Card], hole_cards: &HoleCards, num_players: u8) -> f64 {
        let board: Board = Board::new_from_cards(cards);
        let eq = calc_equity_vs_random(
            &board,
            &hole_cards,
            num_players as usize,
            NUM_SIMULATIONS,
            Round::River.get_num_board_cards(),
        )
        .unwrap();

        eq
    }

    fn get_cache_name() -> EvalCacheEnum {
        EvalCacheEnum::MonteCarloEval
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use log::info;

    use crate::{
        board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProducePartialRankCards},
        init_test_logger, Board, Card, Deck, HoleCards,
    };

    //// cargo test cache_perf --lib --release -- --nocapture

    //a bit slow
    //#[test]
    #[allow(dead_code)]
    fn test_cache_partial_rank() {
        init_test_logger();

        let mut agent_deck = Deck::new();
        let mut cards = Board::new();

        agent_deck.reset();
        //delete if exists
        //std::fs::remove_file(db_name).unwrap_or_default();

        //let mut flop_texture_db = FlopTextureJamDb::new(db_name).unwrap();

        //let mut flop_texture_db = FlopTextureReDb::new(re_db_name).unwrap();
        let mut partial_rank_db: EvalCacheWithHcReDb<ProducePartialRankCards> =
            EvalCacheWithHcReDb::new().unwrap();
        let now = Instant::now();
        let iter_count = 500_000;
        // Code block to measure.
        {
            for i in 0..iter_count {
                cards.add_cards_from_deck(&mut agent_deck, 3).unwrap();
                let hole1: Card = agent_deck.get_unused_card().unwrap().try_into().unwrap();
                let hole2: Card = agent_deck.get_unused_card().unwrap().try_into().unwrap();
                let hole_cards: HoleCards = HoleCards::new(hole1, hole2).unwrap();
                let _texture = partial_rank_db.get_put(&mut cards, &hole_cards, 4).unwrap();
                cards.clear_cards();
                agent_deck.reset();

                if partial_rank_db.cache_misses > 0 && partial_rank_db.cache_misses % 1000 == 0 {
                    println!("Iter {}", i);
                    info!(
                        "Cache hits {} misses {}",
                        partial_rank_db.cache_hits, partial_rank_db.cache_misses
                    );
                }
                if partial_rank_db.cache_hits > 0 && partial_rank_db.cache_hits % 100_000 == 0 {
                    println!("Iter {}", i);
                    info!(
                        "Cache hits {} misses {}",
                        partial_rank_db.cache_hits, partial_rank_db.cache_misses
                    );
                }
            }
        }

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        info!(
            "Cache hits {} misses {}",
            partial_rank_db.cache_hits, partial_rank_db.cache_misses
        );
    }
}
