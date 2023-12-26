use redb::{Database, Error as ReDbError, ReadableTable, TableDefinition, ReadTransaction, RedbKey};
use serde::{Deserializer, Serialize, Deserialize, de::DeserializeOwned};

use crate::{CombinatorialIndex, Card, HoleCards};

//u32 is usually  enough
//In the worst case we have 5 cards * 2 cards
//52 choose 5 is 2,598,960 * 47 choose 2 is 1,081 == 2,810,503,360 with fits in a u32
//To do the above though needs an ordering, which is a pain
// need 22 bits for 52 choose 5
// need 11 bits for 52 choose 2

pub const PARTIAL_RANK_PATH :&str = "/home/eric/git/poker_eval/data/partial_rank_re.db";

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("eval_cache");

pub trait ProduceEvalWithHcResult<R> {
    fn produce_eval_result(cards: &[Card], hole_cards: HoleCards) -> R;

}


//P is producer type
//R is result
//K is the key type
pub struct EvalCacheWithHcReDb< P, R> 
{
    db_name: String,
    db: Database,
    c_index: CombinatorialIndex,
    pub cache_hits: u32,
    pub cache_misses: u32,

    producer: P,
    phantom: std::marker::PhantomData<R>
}


impl <P, R> EvalCacheWithHcReDb< P, R> 
where P : ProduceEvalWithHcResult<R>, R :  Serialize + DeserializeOwned,
{
    //each different struct should get its own db path
    pub fn new(db_name: &str, producer: P) -> Result<Self, ReDbError> {
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
            db_name: db_name.to_string(),
            db,
            cache_hits: 0,
            cache_misses: 0,
            producer,
            phantom: std::marker::PhantomData,
            c_index: CombinatorialIndex::new(),

        })
    }

    pub fn get_put(&mut self, cards: &[Card], hole_cards: HoleCards   ) -> Result<R, ReDbError> {
        let index = self.c_index.get_index(cards);

        let mut index_bytes: [u8;6] = [0;6];
         // Packing the u32 into the first 4 bytes of the array
         index_bytes[0] = (index >> 24) as u8; // Extracts the first byte
            index_bytes[1] = (index >> 16) as u8; // Extracts the second byte
            index_bytes[2] = (index >> 8) as u8;  // Extracts the third byte
            index_bytes[3] = index as u8;         // Extracts the fourth byte
            index_bytes[4] = hole_cards.get_hi_card().into();
            index_bytes[5] = hole_cards.get_lo_card().into();
        
        let opt = self.get(&index_bytes)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let result = P::produce_eval_result(cards, hole_cards);
        self.cache_misses += 1;
        
        self.put(&index_bytes, &result)?;

        Ok(result)
    }


    fn get(&mut self, index: &[u8]) -> Result<Option<R>, ReDbError> {
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

    fn put(&mut self, index: &[u8], result: &R) -> Result<(), ReDbError> {
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
