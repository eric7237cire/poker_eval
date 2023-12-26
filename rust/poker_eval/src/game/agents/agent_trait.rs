use crate::{ActionEnum, GameState, HoleCards, PlayerState};

pub struct AgentDecision {
    pub action: ActionEnum,
    pub comment: Option<String>,
}

pub trait Agent {
    //Get hand cards with index_to_card_pair
    fn decide(
        //To be able to mutate internal state
        &mut self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> AgentDecision;

    fn get_hole_cards(&self) -> HoleCards;

    fn set_hole_cards(&mut self, hole_cards: HoleCards);

    fn get_name(&self) -> &str;
}
