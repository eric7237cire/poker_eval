use std::{collections::HashMap, cell::RefCell, rc::Rc};

use log::info;
use once_cell::sync::Lazy;
use redb::{Database, TableDefinition, Error as ReDbError, ReadTransaction, ReadableTable};


use crate::{game::core::{PlayerState, PlayerAction, GameState}, ALL_HOLE_CARDS, HoleCards, board_hc_eval_cache_redb::{ProduceMonteCarloEval, EvalCacheWithHcReDb}, monte_carlo_equity::get_equivalent_hole_board, board_eval_cache_redb::{get_data_path, EvalCacheEnum}};

#[derive(Eq, PartialEq, Hash)]
pub struct InfoState {
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

    pub round: u8,
}


pub static HOLE_CARDS_CATEGORY: Lazy<Vec<u8>> = Lazy::new(|| {

  let mut catMap = HashMap::new();

  let catStrings = [
    "AA, KK, QQ, JJ, TT",
    "AKs, AQs, AJs, ATs, A9s, A8s, AKo, KQs, KJs, KTs, AQo, KQo, AJo, ATo, A9o, 99, 88, 77, 66, 55",
    "A7s, A6s, A5s, A4s, A3s, A2s, K9s, K8s, K7s, K6s, K5s, K4s, K3s, QJs, QTs, Q9s, Q8s, Q7s, Q6s, KJo, QJo, JTs, J9s, J8s, KTo, QTo, JTo, T9s, K9o, Q9o, J9o, A8o, K8o, Q8o, A7o, K7o, A6o, K6o, A5o, K5o, A4o, 44, A3o, 33, A2o, 22" ,
    "K2s, Q5s, Q4s, Q3s, Q2s, J7s, J6s, J5s, J4s, J3s, J2s, T8s, T7s, T6s, T5s, T4s, T3s, T2s, T9o, 98s, 97s, 96s, 95s, 94s, J8o, T8o, 98o, 87s, 86s, 85s, Q7o, J7o, T7o, 97o, 87o, 76s, Q6o, J6o, T6o, 96o, Q5o, J5o, T5o, K4o, Q4o, J4o, K3o, Q3o, J3o, K2o, Q2o, J2o",
    "93s, 92s, 84s, 83s, 82s, 75s, 74s, 73s, 72s, 86o, 76o, 65s, 64s, 63s, 62s, 95o, 85o, 75o, 65o, 54s, 53s, 52s, T4o, 94o, 84o, 74o, 64o, 54o, 43s, 42s, T3o, 93o, 83o, 73o, 63o, 53o, 43o, 32s, T2o, 92o, 82o, 72o, 62o, 52o, 42o, 32o"
  ];

    for (i, catString) in catStrings.iter().enumerate() {
        let cards = catString.split(",");
        for card in cards {
            let card = card.trim();
            catMap.insert(card.to_string(), i);
        }
    }

    let mut ret = vec![u8::MAX; ALL_HOLE_CARDS.len()];
    
    for hc in ALL_HOLE_CARDS.iter() {
        let simplyRangeString = hc.simple_range_string();
        assert!(catMap.contains_key(&simplyRangeString));
        ret[hc.simple_range_index()] = *catMap.get(&simplyRangeString).unwrap() as u8;
        
    }

    ret
});



impl InfoState {
    

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.position);
        bytes.push(self.num_players);
        bytes.push(self.hole_card_category);
        bytes.push(self.equity);
        bytes.push(self.bet_situation);
        bytes.push(self.round);
        bytes
    }

    pub fn from(
        ps: &PlayerAction, 
        game_state: &GameState, 
        player_hole_cards: &HoleCards,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>
    ) -> Self {

        let position = if ps.players_left_to_act == 0 {
            2
        } else if ps.players_left_to_act == ps.non_folded_players {
            0
        } else {
            1
        };

        let hole_card_category = HOLE_CARDS_CATEGORY[player_hole_cards.simple_range_index()];

        let bet_situation = if ps.current_amt_to_call == 0 {
            0 //unbet
        } else if ps.amount_put_in_pot_this_round > 0 {
            2 //facing raise
        } else {
            1 //facing bet
        };

        let board = game_state.board.as_slice_card();
        assert_eq!(board.len(), game_state.current_round.get_num_board_cards());
        let (eq_hole_cards, mut eq_board) =
            get_equivalent_hole_board(&player_hole_cards, board);
        eq_board.get_index();

        assert!(ps.non_folded_players >= 2);
        assert!(ps.non_folded_players <= 10);

        let eq = monte_carlo_db
            .borrow_mut()
            .get_put(&eq_board, &eq_hole_cards, ps.non_folded_players)
            .unwrap();

        Self {
            position,
            num_players: ps.non_folded_players,
            hole_card_category, 
            equity: 1, //
            bet_situation,
            round: game_state.current_round as usize as u8,
        }
    }
}


const TABLE: TableDefinition<&[u8], f64> = TableDefinition::new("eval_cache");

pub struct InfoStateDb {
    db: Database,    
}

impl InfoStateDb
{
    //each different struct should get its own db path
    pub fn new() -> Result<Self, ReDbError> {
        let db_name = get_data_path(EvalCacheEnum::InfostateTraining);
        info!("Opening db {:?}", db_name);
        let db = Database::create(db_name)?;
        {
            //Make sure table exists
            let write_txn = db.begin_write()?;
            {
                let _table = write_txn.open_table(TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self {
            db,
        })
    }

    

    fn get(&mut self, index: &[u8]) -> Result<Option<f64>, ReDbError> {
        let read_txn: ReadTransaction = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let data = table.get(index)?;
        if let Some(data) = data {
            
            Ok(Some(data.value()))
        } else {
            Ok(None)
        }
    }

    fn put(&mut self, index: &[u8], result: f64) -> Result<(), ReDbError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            
            table.insert(index, result)?;
        }

        write_txn.commit()?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::game::agents::HOLE_CARDS_CATEGORY;

    #[test]
    fn test_hole_cards_category() {
        assert_eq!(169, HOLE_CARDS_CATEGORY.len());
        for i in 0..169 {
            assert!(HOLE_CARDS_CATEGORY[i] < 5);
        }
    }
}
