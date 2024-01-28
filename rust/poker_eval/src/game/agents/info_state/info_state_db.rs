use std::fs;

use log::info;

use redb::{Database, Error as ReDbError, ReadTransaction, ReadableTable, TableDefinition};

use crate::{
    board_eval_cache_redb::{get_data_path, EvalCacheEnum},
    PokerError,
};

use crate::game::agents::info_state::{
    info_state_actions, InfoStateActionValueType, InfoStateDbTrait, InfoStateKey, InfoStateValue,
};

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("eval_cache");

impl InfoStateDbTrait for InfoStateDb {
    fn get(&self, key: &InfoStateKey) -> Result<Option<InfoStateValue>, PokerError> {
        let read_txn: ReadTransaction = self.db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();

        let index = key.to_bytes();
        let data = table.get(index.as_slice()).unwrap();

        if let Some(data) = data {            
            let value: InfoStateValue = bincode::deserialize(&data.value()).unwrap();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self, key: &InfoStateKey, value: &InfoStateValue) -> Result<(), PokerError> {
        let write_txn = self.db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();

            let index = key.to_bytes();

            let value_bytes: Vec<u8> = bincode::serialize(value).unwrap();

            table
                .insert(index.as_slice(), value_bytes.as_slice())
                .unwrap();
        }

        write_txn.commit().unwrap();
        Ok(())
    }
}
pub struct InfoStateDb {
    db: Database,
}

impl InfoStateDb {
    //each different struct should get its own db path
    pub fn new(clean: bool) -> Result<Self, ReDbError> {
        let db_name = get_data_path(EvalCacheEnum::InfostateTraining);

        if clean && db_name.exists() {
            info!("Deleting db {:?} since exists and clean=true", db_name);
            fs::remove_file(&db_name).unwrap();
        }

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

        Ok(Self { db })
    }

    pub fn normalize_array(
        arr: &[InfoStateActionValueType],
    ) -> [InfoStateActionValueType; info_state_actions::NUM_ACTIONS] {
        assert_eq!(arr.len(), info_state_actions::NUM_ACTIONS);

        let mut ret = [0.0; info_state_actions::NUM_ACTIONS];

        let min = arr.iter().cloned().fold(
            InfoStateActionValueType::INFINITY,
            InfoStateActionValueType::min,
        );
        let max = arr.iter().cloned().fold(
            InfoStateActionValueType::NEG_INFINITY,
            InfoStateActionValueType::max,
        );

        if (max - min).abs() < InfoStateActionValueType::EPSILON {
            // Avoid division by zero if all elements are the same
            return ret;
        }

        for i in 0..info_state_actions::NUM_ACTIONS {
            ret[i] = (arr[i] - min) / (max - min);
        }
        ret
    }

    pub fn normalized_array_to_string(
        arr: &[InfoStateActionValueType],
        incoming_bet: bool,
    ) -> String {
        assert_eq!(arr.len(), info_state_actions::NUM_ACTIONS);

        let mut ret = String::new();

        for i in 0..info_state_actions::NUM_ACTIONS as u8 {
            let action_name = if incoming_bet {
                match i {
                    info_state_actions::FOLD => "FOLD",
                    info_state_actions::CALL => "CALL",
                    info_state_actions::RAISE_3X => "RAISE_3X",
                    //info_state_actions::ALL_IN => "ALL_IN",
                    _ => "UNKNOWN",
                }
            } else {
                match i {
                    info_state_actions::CHECK => "CHECK",
                    info_state_actions::BET_HALF => "BET_HALF",
                    info_state_actions::BET_POT => "BET_POT",
                    _ => "UNKNOWN",
                }
            };

            ret.push_str(&format!(
                ";{} -> {:.1}",
                action_name,
                100.0 * arr[i as usize]
            ));
        }
        ret
    }
}
