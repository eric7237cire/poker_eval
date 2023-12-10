use postflop_solver::{Hand, Card};

use crate::{AgentState, ChipType};


pub struct GameState {
    
        
    //pot from prev. betting rounds
    pub current_pot: ChipType,

}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            
                
            current_pot: 0,
        }
    }

    
}

impl GameState {
    pub fn new(num_players: u8) -> Self {
        

        GameState {
            current_pot: 0,
        }
    }
}