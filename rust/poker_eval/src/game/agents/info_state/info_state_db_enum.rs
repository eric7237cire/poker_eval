use enum_dispatch::enum_dispatch;

use crate::PokerError;

use crate::game::agents::info_state::{
    info_state_actions, InfoState, InfoStateActionValueType, InfoStateDb, InfoStateMemory,
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
        infostate: &InfoState,
    ) -> Result<Option<[InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>, PokerError>;

    fn put(
        &mut self,
        infostate: &InfoState,
        result: [InfoStateActionValueType; info_state_actions::NUM_ACTIONS],
    ) -> Result<(), PokerError>;
}
