use enum_dispatch::enum_dispatch;

use crate::PokerError;

use crate::game::agents::info_state::{
    InfoStateKey, InfoStateValue, InfoStateDb, InfoStateMemory,
};

#[enum_dispatch]
pub enum InfoStateDbEnum {
    InfoStateDb,
    InfoStateMemory,
}

#[enum_dispatch(InfoStateDbEnum)]
pub trait InfoStateDbTrait {
    fn get(
        &self,
        key: &InfoStateKey,
    ) -> Result<Option<InfoStateValue>, PokerError>;

    fn put(
        &mut self,
        key: &InfoStateKey,
        result: &InfoStateValue,
    ) -> Result<(), PokerError>;
}
