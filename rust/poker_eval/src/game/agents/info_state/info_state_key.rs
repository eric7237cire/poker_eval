use std::{
    cell::RefCell,
    cmp::min,
    collections::HashMap,
    fmt::{Display, Formatter},
    rc::Rc,
};

use once_cell::sync::Lazy;

use crate::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    game::core::{ActionEnum, ChipType, GameState, PlayerAction, PlayerState, Round},
    monte_carlo_equity::get_equivalent_hole_board,
    pre_calc::NUMBER_OF_SIMPLE_HOLE_CARDS,
    Card, HoleCards, ALL_HOLE_CARDS,
};

use crate::game::agents::info_state::info_state_actions;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct InfoStateKey {
    //For now limited to 0 1st position, 1 middle, 2 last
    //This depends on the round too
    //So Preflop this could be middle, and flop could be last
    // first, middle, middle, middle, last
    pub position: u8,

    //This is # of players in the round, capped at 4
    pub num_players: u8,

    //1 to 5
    pub hole_card_category: u8,

    //high/medium/low/very low
    pub equity: u8,

    //unbet, facing bet, facing raise
    pub bet_situation: u8,

    pub round: u8,
}

impl Display for InfoStateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos_str = match self.position {
            0 => "first",
            1 => "middle",
            2 => "last",
            _ => "unknown",
        };

        let eq_str = match self.equity {
            0 => "< 10%",
            1 => "10 - 30%",
            2 => "30 - 50%",
            3 => "50 - 70%",
            4 => "70+%",
            _ => "unknown",
        };

        let round_str = match self.round {
            0 => "preflop",
            1 => "flop",
            2 => "turn",
            3 => "river",
            _ => "unknown",
        };

        let bet_situation_str = match self.bet_situation {
            0 => "unbet",
            1 => "facing bet",
            2 => "facing raise",
            _ => "unknown",
        };

        write!(
            f,
            "InfoState: {} Num Players: {} Hole Card Cat: {} {} {} {}",
            pos_str,
            min(4, self.num_players),
            self.hole_card_category,
            eq_str,
            bet_situation_str,
            round_str
        )
    }
}

pub static HOLE_CARDS_CATEGORY: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut cat_map = HashMap::new();

    let cat_strings = [
    "AA, KK, QQ, JJ, TT",
    "AKs, AQs, AJs, ATs, A9s, A8s, AKo, KQs, KJs, KTs, AQo, KQo, AJo, ATo, A9o, 99, 88, 77, 66, 55",
    "A7s, A6s, A5s, A4s, A3s, A2s, K9s, K8s, K7s, K6s, K5s, K4s, K3s, QJs, QTs, Q9s, Q8s, Q7s, Q6s, KJo, QJo, JTs, J9s, J8s, KTo, QTo, JTo, T9s, K9o, Q9o, J9o, A8o, K8o, Q8o, A7o, K7o, A6o, K6o, A5o, K5o, A4o, 44, A3o, 33, A2o, 22" ,
    "K2s, Q5s, Q4s, Q3s, Q2s, J7s, J6s, J5s, J4s, J3s, J2s, T8s, T7s, T6s, T5s, T4s, T3s, T2s, T9o, 98s, 97s, 96s, 95s, 94s, J8o, T8o, 98o, 87s, 86s, 85s, Q7o, J7o, T7o, 97o, 87o, 76s, Q6o, J6o, T6o, 96o, Q5o, J5o, T5o, K4o, Q4o, J4o, K3o, Q3o, J3o, K2o, Q2o, J2o",
    "93s, 92s, 84s, 83s, 82s, 75s, 74s, 73s, 72s, 86o, 76o, 65s, 64s, 63s, 62s, 95o, 85o, 75o, 65o, 54s, 53s, 52s, T4o, 94o, 84o, 74o, 64o, 54o, 43s, 42s, T3o, 93o, 83o, 73o, 63o, 53o, 43o, 32s, T2o, 92o, 82o, 72o, 62o, 52o, 42o, 32o"
  ];

    for (i, cat_str) in cat_strings.iter().enumerate() {
        let cards = cat_str.split(",");
        for card in cards {
            let card = card.trim();
            cat_map.insert(card.to_string(), i);
        }
    }

    let mut ret = vec![u8::MAX; NUMBER_OF_SIMPLE_HOLE_CARDS];

    for hc in ALL_HOLE_CARDS.iter() {
        let simple_range_string = hc.simple_range_string();
        assert!(cat_map.contains_key(&simple_range_string));
        ret[hc.simple_range_index()] = *cat_map.get(&simple_range_string).unwrap() as u8;
    }

    ret
});

impl InfoStateKey {
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
    //Used in training once poker hand is over
    pub fn from_player_action(
        ps: &PlayerAction,
        game_state: &GameState,
        player_hole_cards: &HoleCards,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    ) -> (Self, u8) {
        let info_state_action: u8 = match ps.action {
            ActionEnum::Fold => info_state_actions::FOLD,
            ActionEnum::Call(_) => info_state_actions::CALL,
            ActionEnum::Check => info_state_actions::CHECK,
            ActionEnum::Bet(amt) => {
                if amt <= ps.pot / 2 {
                    info_state_actions::BET_HALF
                } else {
                    info_state_actions::BET_POT
                }
            }
            ActionEnum::Raise(_, _) => info_state_actions::RAISE_3X,
        };

        (
            Self::new(
                ps.non_folded_players,
                ps.players_left_to_act,
                player_hole_cards,
                monte_carlo_db.clone(),
                ps.current_amt_to_call,
                ps.amount_put_in_pot_this_round,
                &game_state.board.as_slice_card()[0..ps.round.get_num_board_cards()],
                ps.round,
            ),
            info_state_action,
        )
    }

    //Used by the agent when game_state is right before infostate agent acts
    pub fn from_game_state(
        game_state: &GameState,
        player_state: &PlayerState,
        player_hole_cards: &HoleCards,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
    ) -> Self {
        let non_folded_players = game_state.total_active_players + game_state.total_players_all_in;

        assert!(non_folded_players >= 2);
        assert!(non_folded_players <= 10);

        assert!(game_state.num_left_to_act > 0);
        assert!(game_state.num_left_to_act <= non_folded_players);

        Self::new(
            non_folded_players,
            //Game state is before player action has been taken into account
            game_state.num_left_to_act - 1,
            player_hole_cards,
            monte_carlo_db.clone(),
            game_state.current_to_call,
            player_state.cur_round_putting_in_pot,
            game_state.board.as_slice_card(),
            game_state.current_round,
        )
    }

    pub fn new(
        num_non_folded_players: u8,
        num_left_to_act: u8,
        player_hole_cards: &HoleCards,
        monte_carlo_db: Rc<RefCell<EvalCacheWithHcReDb<ProduceMonteCarloEval>>>,
        current_to_call: ChipType,
        cur_round_putting_in_pot: ChipType,
        board: &[Card],
        current_round: Round,
    ) -> Self {
        assert!(num_left_to_act <= num_non_folded_players);

        let position = if num_left_to_act == 0 {
            2
        } else if num_left_to_act == num_non_folded_players {
            0
        } else {
            1
        };

        let hole_card_category = HOLE_CARDS_CATEGORY[player_hole_cards.simple_range_index()];

        let bet_situation = if current_to_call == 0 {
            0 //unbet
        } else if cur_round_putting_in_pot > 0 {
            2 //facing raise
        } else {
            1 //facing bet
        };

        assert_eq!(board.len(), current_round.get_num_board_cards());

        assert!(num_non_folded_players >= 2);
        assert!(num_non_folded_players <= 10);

        let eq = if current_round > Round::Preflop {
            let (eq_hole_cards, mut eq_board) =
                get_equivalent_hole_board(&player_hole_cards, board);
            eq_board.get_index();
            monte_carlo_db
                .borrow_mut()
                .get_put(&eq_board, &eq_hole_cards, num_non_folded_players)
                .unwrap()
        } else {
            //Don't calculate equity for preflop
            0.0
        };

        let equity = if eq < 0.10 {
            0
        } else if eq < 0.30 {
            1
        } else if eq < 0.50 {
            2
        } else if eq < 0.70 {
            3
        } else {
            4
        };

        Self {
            position,
            num_players: min(4, num_non_folded_players),
            hole_card_category,
            equity,
            bet_situation,
            round: current_round as usize as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::agents::info_state::{InfoStateDb, HOLE_CARDS_CATEGORY};

    #[test]
    fn test_hole_cards_category() {
        assert_eq!(169, HOLE_CARDS_CATEGORY.len());
        for i in 0..169 {
            assert!(HOLE_CARDS_CATEGORY[i] < 5);
        }
    }

    #[test]
    fn test_normalize_array() {
        let test_values = [0.5, 5.0, -3.0];

        let normalized = InfoStateDb::normalize_array(&test_values);

        assert_eq!(normalized[0], 3.5 / 8.0);
        assert_eq!(normalized[1], 1.0);
        assert_eq!(normalized[2], 0.0);

        // let test_values = [f32::MAX, -3.0, f32::MIN];

        // let normalized = InfoStateDb::normalize_array(&test_values);

        // assert_eq!(normalized[0], 1.0);
        // assert_eq!(normalized[1], 0.5);
        // assert_eq!(normalized[2], 0.0);
    }
}
