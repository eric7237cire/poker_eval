use crate::{
    game::core::{CommentedAction, GameState, PlayerState},
    HoleCards,
};

use enum_dispatch::enum_dispatch;

use super::{PassiveCallingStation, InfoStateAgent, EqAgent, Tag, PanicAgent};

#[enum_dispatch]
pub enum AgentEnum {
    InfoStateAgent,
    EqAgent,
    Tag,
    PassiveCallingStation,
    PanicAgent,
}

#[enum_dispatch(AgentEnum)]
pub trait Agent {
    //Get hand cards with index_to_card_pair
    fn decide(
        //To be able to mutate internal state
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> CommentedAction;

    fn get_hole_cards(&self) -> HoleCards;

    fn set_hole_cards(&mut self, hole_cards: HoleCards);

    fn get_name(&self) -> &str;
}
