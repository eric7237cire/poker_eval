
use jammdb::{Error as JammDbError, DB};
use serde::{Serialize, de::DeserializeOwned};

use crate::{CombinatorialIndex, eval_cache_redb::ProduceEvalResult, Card};

//Example of using another cache store
#[allow(dead_code)]
struct EvalCacheJamDb<P, R>  {
    db_name: String,
    db: DB,
    c_index: CombinatorialIndex,
    cache_hits: u32,
    cache_misses: u32,

    producer: P,

    phantom: std::marker::PhantomData<R>
}

#[allow(dead_code)]
impl <P, R> EvalCacheJamDb<P, R> 
where P : ProduceEvalResult<R, u32>, R :  Serialize + DeserializeOwned {
    pub fn new(db_name: &str, producer: P) -> Result<Self, JammDbError> {
        let db = DB::open(db_name)?;

        {
            let tx = db.tx(true)?;
            let bucket = tx.get_bucket("flop_texture");

            if bucket.is_err() {
                tx.create_bucket("flop_texture")?;
                tx.commit()?;
            }
        }

        Ok(EvalCacheJamDb {
            db_name: db_name.to_string(),
            db,
            c_index: CombinatorialIndex::new(),
            cache_hits: 0,
            cache_misses: 0,
            producer,
            phantom: std::marker::PhantomData
        })
    }

    pub fn get_put(&mut self, cards: &[Card]) -> Result<R, JammDbError> {

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

    pub fn get(&mut self, index: u32) -> Result<Option<R>, JammDbError> {
        let tx = self.db.tx(false)?;
        let bucket = tx.get_bucket("flop_texture")?;
        
        if let Some(data) = bucket.get(index.to_be_bytes()) {
            let result: R = rmp_serde::from_slice(data.kv().value()).unwrap();
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub fn put(&mut self, index: u32, result: &R) -> Result<(), JammDbError> {
        let tx = self.db.tx(true)?;
        let bucket = tx.get_bucket("flop_texture")?;
        
        let texture_bytes = rmp_serde::to_vec(result).unwrap();
        bucket.put(index.to_be_bytes(), texture_bytes)?;
        tx.commit()?;
        Ok(())
    }
}
