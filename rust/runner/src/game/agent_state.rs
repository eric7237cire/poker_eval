use crate::{Position, ChipType};

#[derive(Clone, Copy, Debug)]
pub struct AgentState {
    //if stack is 0, is all in if not folded
    pub stack: ChipType,
    pub position: Position,
    //Index into range
    pub hole_cards: usize,

    pub folded: bool,

    //in current round, reset to 0 each round
    pub already_bet: ChipType,
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState {
            stack: 100,
            position: Position::Button,
            hole_cards: 0,
            folded: false,
            already_bet: 0
        }
    }
}

impl AgentState {
    pub fn new(stack: ChipType, position: Position, hole_cards: usize) -> Self {
        AgentState {
            stack,
            position,
            hole_cards,
            folded: false,
            already_bet: 0
        }
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
        }

        self.already_bet += chips_put_in_pot;

        chips_put_in_pot
    }
}