pub struct infostate {
    //For now limited to 0 1st position, 1 middle, 2 last
    //This depends on the round too
    //So Preflop this could be middle, and flop could be last
    // first, middle, middle, middle, last
    pub position: u8,

    //This is # of players in the round
    pub num_players: u8,

    //1 to 5
    pub hole_card_category: u8,

    //high/medium/low
    pub equity: u8,

    //unbet, facing bet, facing raise
    pub bet_situation: u8,
}

impl infostate {
    pub fn new() -> Self {
        infostate {
            position: 0,
            num_players: 0,
            hole_card_category: 0,
            equity: 0,
            bet_situation: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.position);
        bytes.push(self.num_players);
        bytes.push(self.hole_card_category);
        bytes.push(self.equity);
        bytes.push(self.bet_situation);
        bytes
    }
}
