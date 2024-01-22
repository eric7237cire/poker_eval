pub type InfoStateActionValueType = f32;


//When have an incoming bet
pub const FOLD: u8 = 0;
pub const CALL: u8 = 1;
pub const RAISE_3X: u8 = 2;

//With no incoming bet (though bb means put in pot == to call)
pub const CHECK: u8 = 0;    
pub const BET_HALF: u8 = 1;
pub const BET_POT: u8 = 2;

//pub const ALL_IN: u8 = 6;

pub const NUM_ACTIONS: usize = 3;

