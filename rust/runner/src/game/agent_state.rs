use postflop_solver::{Hand, card_pair_to_index};
use poker_rs::core::{Card as PsCard, Suit as PsSuit};
use crate::{Position, ChipType};

#[derive(Debug)]
pub struct AgentState {
    //if stack is 0, is all in if not folded
    pub stack: ChipType,
    pub position: Position,
    
    
    //First 2 are hole tards, then flop (3 cards), turn, river
    pub cards: Vec<PsCard>,

    pub folded: bool,

    //in current round, reset to 0 each round
    pub already_bet: ChipType,
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState {
            stack: 100,
            position: Position::Button,
            cards: Vec::with_capacity(7),
            folded: false,
            already_bet: 0
        }
    }

    
}

fn poker_rs_card_to_eval_card(card: PsCard) -> u8 {

    //Use values from poker_evaluate
    let suit = match card.suit {
        PsSuit::Spade => 3,
        PsSuit::Heart => 2,
        PsSuit::Diamond => 1,
        PsSuit::Club => 0,        
    };
    let value = card.value as u8;

    (value << 2) | suit
}

impl AgentState {
    pub fn get_range_index_for_hole_cards(&self) -> usize {
        
        card_pair_to_index(
            poker_rs_card_to_eval_card(self.cards[0]),
            poker_rs_card_to_eval_card(self.cards[1]),
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
        }

        self.already_bet += chips_put_in_pot;

        chips_put_in_pot
    }
}