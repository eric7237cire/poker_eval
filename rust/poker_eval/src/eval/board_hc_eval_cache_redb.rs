use redb::{Database, Error as ReDbError, ReadableTable, TableDefinition, ReadTransaction, RedbKey};
use serde::{Deserializer, Serialize, Deserialize, de::DeserializeOwned};

use crate::{CombinatorialIndex, Card, HoleCards};

//u32 is usually  enough
//In the worst case we have 5 cards * 2 cards
//52 choose 5 is 2,598,960 * 47 choose 2 is 1,081 == 2,810,503,360 with fits in a u32
//To do the above though needs an ordering, which is a pain
// need 22 bits for 52 choose 5
// need 11 bits for 52 choose 2

pub const FLOP_TEXTURE_PATH :&str = "/home/eric/git/poker_eval/data/flop_texture_re.db";

pub trait ProduceEvalResult<R, K> {
    fn produce_eval_result(cards: &[Card], hole_cards: Option<HoleCards>) -> R;

    //converts cards to a key
    fn get_key(&mut self, cards: &[Card], hole_cards: Option<HoleCards>) -> K;
}

type U8Slice<'a> = &'a [u8];

pub trait EvalCacheKey {}

impl <'a> EvalCacheKey for U8Slice<'a> {}
impl EvalCacheKey for u32 {}

//P is producer type
//R is result
//K is the key type
pub struct EvalCacheReDb< P, R, K> 
where K : EvalCacheKey + RedbKey + 'static
{
    db_name: String,
    db: Database,
    //c_index: CombinatorialIndex,
    pub cache_hits: u32,
    pub cache_misses: u32,

    producer: P,

    table: TableDefinition<'static, K, &'static [u8]>,

    phantom: std::marker::PhantomData<R>
}


impl <P, R, K> EvalCacheReDb< P, R, K> 
where P : ProduceEvalResult<R, K>, R :  Serialize + DeserializeOwned,
K : EvalCacheKey + RedbKey + 'static + std::borrow::Borrow<<K as redb::RedbValue>::SelfType<'static>>

{
    //each different struct should get its own db path
    pub fn new(db_name: &str, producer: P) -> Result<Self, ReDbError> {
        let db = Database::create(db_name)?;
        let table = TableDefinition::new("eval_cache");
        {
            //Make sure table exists
            let write_txn = db.begin_write()?;
            {
                let _table = write_txn.open_table(table)?;
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
            table

        })
    }

    pub fn get_put(&mut self, cards: &[Card], hole_cards: Option<HoleCards>   ) -> Result<R, ReDbError> {
        //let index = self.c_index.get_index(cards);
        let index = self.producer.get_key(cards, hole_cards);

        let opt = self.get(index)?;
        if opt.is_some() {
            self.cache_hits += 1;
            return Ok(opt.unwrap());
        }

        let result = P::produce_eval_result(cards, hole_cards);
        self.cache_misses += 1;
        let index = self.producer.get_key(cards, hole_cards);
        self.put(index, &result)?;

        Ok(result)
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

    fn get(&mut self, index: K) -> Result<Option<R>, ReDbError> {
        let read_txn: ReadTransaction = self.db.begin_read()?;
        let table = read_txn.open_table(self.table)?;

        let data = table.get(index)?;
        if let Some(data) = data {
            //let texture: BoardTexture = rmp_serde::from_slice(data.value()).unwrap();
            let texture: R = bincode::deserialize(&data.value()).unwrap();

            Ok(Some(texture))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self, index: K, result: &R) -> Result<(), ReDbError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.table)?;
            //let texture_bytes = rmp_serde::to_vec(texture).unwrap();
            let texture_bytes: Vec<u8> = bincode::serialize(&result).unwrap();

            table.insert(index, texture_bytes.as_slice())?;
        }

        write_txn.commit()?;
        Ok(())
    }
}
