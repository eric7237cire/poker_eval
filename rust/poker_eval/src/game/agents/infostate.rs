use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Formatter},
    mem,
    rc::Rc,
};

use log::info;
use once_cell::sync::Lazy;
use redb::{Database, Error as ReDbError, ReadTransaction, ReadableTable, TableDefinition};

use crate::{
    board_eval_cache_redb::{get_data_path, EvalCacheEnum},
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::core::{ActionEnum, GameState, PlayerAction, PlayerState},
    monte_carlo_equity::get_equivalent_hole_board,
    HoleCards, ALL_HOLE_CARDS,
};

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

impl Display for InfoState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos_str = match self.position {
            0 => "first",
            1 => "middle",
            2 => "last",
            _ => "unknown",
        };

        let eq_str = match self.equity {
            0 => "low eq",
            1 => "medium eq",
            2 => "high eq",
            _ => "unknown",
        };

        let round_str = match self.round {
            0 => "preflop",
            1 => "flop",
            2 => "turn",
            3 => "river",
            _ => "unknown",
        };

        write!(
            f,
            "InfoState: {} {} {} {} {} {}",
            pos_str,
            self.num_players,
            self.hole_card_category,
            eq_str,
            self.bet_situation,
            round_str
        )
    }
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

    //returns action as well
    pub fn from(
        ps: &PlayerAction,
        game_state: &GameState,
        player_hole_cards: &HoleCards,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    ) -> (Self, u8) {
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
        let (eq_hole_cards, mut eq_board) = get_equivalent_hole_board(&player_hole_cards, board);
        eq_board.get_index();

        assert!(ps.non_folded_players >= 2);
        assert!(ps.non_folded_players <= 10);

        let eq = monte_carlo_db
            .borrow_mut()
            .get_put(&eq_board, &eq_hole_cards, ps.non_folded_players)
            .unwrap();

        let equity = if eq < 0.33 { 0 } else if eq < 0.66 { 1 } else { 2 };

        let info_state_action: u8 = match ps.action {
            ActionEnum::Fold => info_state_actions::FOLD,
            ActionEnum::Call(_) => info_state_actions::CALL,
            ActionEnum::Check => info_state_actions::CHECK,
            ActionEnum::Bet(amt) => {
                if amt <= game_state.pot() / 2 {
                    info_state_actions::BET_HALF
                } else {
                    info_state_actions::BET_POT
                }
            }
            ActionEnum::Raise(_, _) => info_state_actions::RAISE_3X,
        };

        (
            Self {
                position,
                num_players: ps.non_folded_players,
                hole_card_category,
                equity,
                bet_situation,
                round: ps.round as usize as u8,
            },
            info_state_action,
        )
    }
}

pub type InfoStateActionValueType = f32;

pub mod info_state_actions {
    pub const FOLD: u8 = 0;
    pub const CHECK: u8 = 1;
    pub const CALL: u8 = 2;
    pub const BET_HALF: u8 = 3;
    pub const BET_POT: u8 = 4;
    pub const RAISE_3X: u8 = 5;
    pub const ALL_IN: u8 = 6;

    pub const NUM_ACTIONS: usize = 7;
}

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("eval_cache");

pub struct InfoStateDb {
    db: Database,
}

impl InfoStateDb {
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

        Ok(Self { db })
    }

    fn get(
        &mut self,
        infostate: &InfoState,
    ) -> Result<Option<[InfoStateActionValueType; info_state_actions::NUM_ACTIONS]>, ReDbError>
    {
        let read_txn: ReadTransaction = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let index = infostate.to_bytes();
        let data = table.get(index.as_slice())?;
        if let Some(data) = data {
            let bytes = data.value();
            let mut ret = [0.0; info_state_actions::NUM_ACTIONS];
            for i in 0..info_state_actions::NUM_ACTIONS {
                ret[i] = InfoStateActionValueType::from_le_bytes(
                    bytes[i * 8..(i + 1) * 8].try_into().unwrap(),
                );
            }
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }

    fn put(
        &mut self,
        infostate: &InfoState,
        result: [InfoStateActionValueType; info_state_actions::NUM_ACTIONS],
    ) -> Result<(), ReDbError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;

            let index = infostate.to_bytes();

            let mut bytes = Vec::with_capacity(
                info_state_actions::NUM_ACTIONS * mem::size_of::<InfoStateActionValueType>(),
            );

            for i in 0..info_state_actions::NUM_ACTIONS {
                bytes.extend_from_slice(&result[i].to_le_bytes());
            }

            table.insert(index.as_slice(), bytes.as_slice())?;
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
