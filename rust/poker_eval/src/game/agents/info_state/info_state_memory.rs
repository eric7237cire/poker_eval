use std::collections::HashMap;

use crate::PokerError;

use crate::game::agents::info_state::{
    InfoStateKey, InfoStateValue, InfoStateDbTrait,
};

//For testing, to have the infostate action values
pub struct InfoStateMemory {
    hash_map: HashMap<InfoStateKey, InfoStateValue>,
}

impl InfoStateDbTrait for InfoStateMemory {
    fn get(
        &self,
        infostate: &InfoStateKey,
    ) -> Result<Option<InfoStateValue>, PokerError>
    {
        let v = self.hash_map.get(infostate);
        if v.is_some() {
            Ok(Some(v.unwrap().clone()))
        } else {
            Ok(None)
        }
    }

    fn put(
        &mut self,
        infostate: &InfoStateKey,
        result: &InfoStateValue,
    ) -> Result<(), PokerError> {
        let info_state: InfoStateKey = infostate.clone();
        self.hash_map.insert(info_state, result.clone());

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
