use core::num;

use boomphf::Mphf;

use crate::pre_calc::fast_eval::fast_hand_eval;
use crate::pre_calc::rank::Rank;
use crate::web::{
    PlayerFlopResults, PlayerPreFlopState, PreflopPlayerInfo, ResultType, MAX_PLAYERS,
};
use crate::{Card, HoleCards, PokerError, NUM_RANK_FAMILIES, SIMPLE_RANGE_INDEX_LEN};

pub struct RankResults {
    pub(crate) num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    pub(crate) win_eq: f64,
    pub(crate) tie_eq: f64,

    //Also track equity by the simplified hole card range index
    pub(crate) eq_by_range_index: Vec<f64>,
    pub(crate) num_it_by_range_index: Vec<ResultType>,

    //This is a win or lose tally of hand ranks, [0] for high card .. [8] for straight flush
    //float because a tie counts for a fraction of a win
    pub(crate) win_rank_family_count: [f64; NUM_RANK_FAMILIES],
    pub(crate) lose_rank_family_count: [ResultType; NUM_RANK_FAMILIES],
}

impl Default for RankResults {
    fn default() -> Self {
        Self {
            eq_by_range_index: vec![0.0; SIMPLE_RANGE_INDEX_LEN],
            num_it_by_range_index: vec![0; SIMPLE_RANGE_INDEX_LEN],
            win_rank_family_count: [0.0; NUM_RANK_FAMILIES],
            lose_rank_family_count: [0; NUM_RANK_FAMILIES],
            num_iterations: 0,
            win_eq: 0.0,
            tie_eq: 0.0,
        }
    }
}

/*
Assumes all players have either hole cards or their ranges chosen
*/
pub fn eval_current(
    active_players: &[(usize, &PreflopPlayerInfo)],
    player_cards: &[HoleCards],
    eval_cards: &mut Vec<Card>,
    flop_results: &mut Vec<PlayerFlopResults>,
    //treat first active player as the hero, all others as villians
    villian_results: &mut PlayerFlopResults,
    street_index: usize,
    hash_func: &Mphf<u32>,
) -> Result<(), PokerError> {
    if eval_cards.len() < 3 {
        return Err(PokerError::from_string(format!(
            "eval_current: eval_cards needs at least 3 cards, but had {} cards",
            eval_cards.len()
        )));
    }
    if eval_cards.len() > 5 {
        return Err(PokerError::from_string(format!(
            "eval_current: too many eval_cards, should be 5 max, but had {} cards",
            eval_cards.len()
        )));
    }

    let n_players = active_players.len();
    assert!(n_players > 1);
    assert_eq!(player_cards.len(), n_players);

    let mut hand_evals: Vec<Rank> = Vec::with_capacity(n_players);

    for (active_index, (_p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        player_cards[active_index].add_to_eval(eval_cards);

        let rank = fast_hand_eval(eval_cards.iter(), hash_func);

        

        flop_results[active_index].street_rank_results[street_index].num_it_by_range_index
            [player_cards[active_index].to_simple_range_index()] += 1;

        hand_evals.push(rank);

        player_cards[active_index].remove_from_eval(eval_cards)?;
    }

    //Best villian hand
    let best_villian_rank = hand_evals[1..]
        .iter()
        .fold(Rank::lowest_rank(), |acc, &x| acc.max(x));
    
        update_results_from_rank(
            &mut villian_results.street_rank_results[street_index],
            &best_villian_rank,
            1.0);
    

    let (max_value, num_with_max) = winning_rank(&hand_evals);

    for (active_player_index, rank) in hand_evals.iter().enumerate() {

        if *rank == max_value {
            let results = &mut flop_results[active_player_index].street_rank_results[street_index];
            if num_with_max == 1 {
                results.win_eq += 1.0;
                if active_player_index > 0 {
                    villian_results.street_rank_results[street_index].win_eq += 1.0;
                }
            } else {
                results.tie_eq += 1.0 / num_with_max as f64;

                if active_player_index > 0 {
                    villian_results.street_rank_results[street_index].tie_eq +=
                        1.0 / num_with_max as f64;
                }
            }

            //Update equity by range index
            let range_index = player_cards[active_player_index].to_simple_range_index();
            results.eq_by_range_index[range_index] += 1.0 / active_player_index as f64;

            update_results_from_rank(
                &mut flop_results[active_player_index].street_rank_results[street_index],
                rank,
                1.0 / num_with_max as f64,
            );
        } else {
            //losing
            update_results_from_rank(
                &mut flop_results[active_player_index].street_rank_results[street_index],
                rank,
                0.0
            );
        }
    }

    Ok(())
}

pub(crate) fn update_results_from_rank(results: &mut RankResults, rank: &Rank, amt: f64) {
    results.num_iterations += 1;
    if amt == 0.0 {
        results.lose_rank_family_count[rank.get_rank_enum() as u8 as usize] += 1;
    } else {
        results.win_rank_family_count[rank.get_rank_enum() as u8 as usize] += amt;
    } 
    
}

//returns winners and how many players were considered (non None rank)
pub(crate) fn winning_rank(arr: &[Rank]) -> (Rank, usize) {
    
    let mut max_value = Rank::lowest_rank();
    let mut num_with_max = 0;

    for (index, &value) in arr.iter().enumerate() {
        if value > max_value {
            max_value = value;
            num_with_max = 1;            
        } else if value == max_value {
            num_with_max += 1;
        }
    }

    (max_value, num_with_max)
}
