use std::{
    collections::HashMap,
};



use redb::{ReadableTable};

use crate::{
    PokerError,
};

use crate::game::agents::info_state::{InfoState, InfoStateActionValueType, info_state_actions, InfoStateDbTrait};

//For testing, to have the infostate action values
pub struct InfoStateMemory {
    hash_map: HashMap<InfoState, [InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>,
}

impl InfoStateDbTrait for InfoStateMemory {
    fn get(&self,infostate: &InfoState,) -> Result<Option<[InfoStateActionValueType;
info_state_actions::NUM_ACTIONS]>,PokerError> {
        let v = self.hash_map.get(infostate);
        if v.is_some() {
            Ok(Some(*v.unwrap()))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self,infostate: &InfoState,result:[InfoStateActionValueType;
info_state_actions::NUM_ACTIONS],) -> Result<(),PokerError> {
        let info_state: InfoState = infostate.clone();
        self.hash_map.insert(info_state, result);

        Ok(())
    }
}

impl InfoStateMemory {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }
}