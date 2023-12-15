use postflop_solver::{card_pair_to_index};

use crate::{card_to_eval_card, core::Card, ChipType, Position};

#[derive(Debug)]
pub struct AgentState {
    //Stack they had when cards dealt
    pub initial_stack: ChipType,

    //if stack is 0, is all in if not folded
    pub stack: ChipType,
    pub position: Position,

    //First 2 are hole tards, then flop (3 cards), turn, river
    pub cards: Vec<Card>,

    pub folded: bool,

    //in current round, reset to 0 each round
    pub already_bet: ChipType,
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState {
            stack: 100,
            initial_stack: 100,
            position: Position::Button,
            cards: Vec::with_capacity(7),
            folded: false,
            already_bet: 0,
        }
    }
}
impl AgentState {
    pub fn get_range_index_for_hole_cards(&self) -> usize {
        card_pair_to_index(
            card_to_eval_card(self.cards[0]),
            card_to_eval_card(self.cards[1]),
        )
    }

    //Used for sb, bb as well, handles if puts them all in
    //returns amount actually put in pot
    //Will bet just the delta of what they've already put up in the betting round
    pub fn handle_put_money_in_pot(&mut self, total_call_amt: ChipType) -> ChipType {
        assert!(total_call_amt >= self.already_bet);
        let chips_needed_in_pot = total_call_amt - self.already_bet;
        let mut chips_put_in_pot = chips_needed_in_pot;

        if self.stack <= chips_needed_in_pot {
            //all in
            self.already_bet += self.stack;

            chips_put_in_pot = self.stack;
            self.stack = 0;
        } else {
            self.stack -= chips_needed_in_pot;
            self.already_bet += chips_put_in_pot;
        }

        chips_put_in_pot
    }
}
