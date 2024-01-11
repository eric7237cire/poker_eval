use std::cmp::min;

use serde::Serialize;

use crate::Board;

use crate::CommentedAction;
use crate::HoleCards;
use crate::PlayerAction;
use crate::Position;

use crate::ChipType;
use crate::Round;

/*
Everything that is known to all players,
which is why hole cards are not here
*/
#[derive(Default)]
pub struct PlayerState {
    pub position: Position,
    pub player_name: String,
    pub initial_stack: ChipType,

    //what has not yet been put in the middle
    pub stack: ChipType,

    //None means not yet acted this round
    //already deducted from stack
    pub cur_round_putting_in_pot: Option<ChipType>,

    pub total_put_in_pot: ChipType,

    //In current betting round, so == remaining stack
    pub all_in: bool,

    pub final_state: Option<FinalPlayerState>,
}

#[derive(Serialize)]
pub struct InitialPlayerState {
    pub stack: ChipType,
    pub player_name: String,

    //0 -- sb, 1 bb, 2 utg, 3 hj, 4 btn
    pub position: Position,

    pub cards: Option<HoleCards>,
}

#[repr(u8)]
#[derive(Serialize, Copy, Clone, Eq, PartialEq)]
pub enum FinalPlayerState {
    //The round in which they folded
    Folded(Round),
    WonShowdown, //perhaps a tie
    LostShowdown,
    EveryoneElseFolded,
}

impl FinalPlayerState {
    pub fn is_folded(&self) -> bool {
        match self {
            FinalPlayerState::Folded(_) => true,
            _ => false,
        }
    }
}

//Helpers for common things that agents need to know to decide
pub struct AgentDecisionHelpers {
    //How much extra we need to put in to call the current bet, can be less than the total call required
    //if we are calling a raise to our bet
    //Bounded by our stack
    pub call_amount: ChipType,
    pub max_can_raise: ChipType,
    pub min_can_raise: ChipType,
    pub can_raise: bool,
}

impl PlayerState {
    pub fn new(initial_player_state: &InitialPlayerState) -> Self {
        PlayerState {
            position: initial_player_state.position,
            stack: initial_player_state.stack,
            initial_stack: initial_player_state.stack,
            final_state: None,
            cur_round_putting_in_pot: None,
            all_in: false,
            player_name: initial_player_state.player_name.clone(),
            total_put_in_pot: 0,
        }
    }

    //Still in the hand, able to act
    pub fn is_active(&self) -> bool {
        !self.is_folded() && !self.all_in
    }

    pub fn is_folded(&self) -> bool {
        self.final_state.is_some() && self.final_state.as_ref().unwrap().is_folded()
    }

    pub fn player_index(&self) -> usize {
        self.position.into()
    }

    pub fn get_helpers(&self, game_state: &GameState) -> AgentDecisionHelpers {
        let call_amt = min(
            game_state.current_to_call - self.cur_round_putting_in_pot.unwrap_or(0),
            self.stack,
        );
        //we can raise to a stack more that what we've already put in
        //Both these values are the total amount, not the increase
        let max_can_raise = self.stack + self.cur_round_putting_in_pot.unwrap_or(0);
        let min_can_raise = min(
            game_state.min_raise + game_state.current_to_call,
            max_can_raise,
        );

        //let third_pot = max(min_can_raise, min(max_can_raise, current_pot / 3));

        let can_raise = max_can_raise > call_amt + self.cur_round_putting_in_pot.unwrap_or(0);

        AgentDecisionHelpers {
            call_amount: call_amt,
            max_can_raise,
            min_can_raise,
            can_raise,
        }
    }
}

impl AgentDecisionHelpers {
    pub fn build_raise_to(
        &self,
        game_state: &GameState,
        raise_to: ChipType,
        comment: String,
    ) -> CommentedAction {
        //Apply max
        let raise_to = min(raise_to, self.max_can_raise);

        if self.can_raise {
            CommentedAction {
                action: crate::ActionEnum::Raise(raise_to - game_state.current_to_call, raise_to),
                comment: Some(comment),
            }
        } else {
            CommentedAction {
                action: crate::ActionEnum::Call(self.call_amount),
                comment: Some(comment),
            }
        }
    }
}

pub struct GameState {
    //sb first order
    pub player_states: Vec<PlayerState>,

    pub current_to_act: Position,
    pub current_round: Round,

    //Note this 'resets' when there is a bet or raise as we need to go around again
    //pub num_acted_this_round: u8,
    //acted this round == total_active_players - num_left_to_act
    //note that if a player folds or goes all in, they are no longer counted as active
    //the last to act (unless they bet/raise) will have this == 0 when they decide
    pub num_left_to_act: u8,
    //active means not folded and not all in
    pub total_active_players: u8,
    pub total_players_all_in: u8,
    //so number non folded == total_active_players + total_players_all_in
    pub prev_round_pot: ChipType,

    //Until current rounds are finished, is not added to pot
    pub round_pot: ChipType,

    //Amount next player needs to call (total, not diff what they need to put in)
    pub current_to_call: ChipType,

    //Initial bet is also considered a raise for this value
    //This is the smallest increase of the bet value allowed, unless going all in
    pub min_raise: ChipType,

    pub board: Board,

    pub sb: ChipType,
    pub bb: ChipType,

    //All actions in the game, posting blinds not considered an action
    //The state is as it was when the player acted
    pub actions: Vec<PlayerAction>,
}

impl GameState {
    pub fn pot(&self) -> ChipType {
        self.prev_round_pot + self.round_pot
    }

    pub fn non_folded_players(&self) -> u8 {
        self.player_states
            .iter()
            .filter(|ps| !ps.is_folded())
            .count() as u8
    }
}
