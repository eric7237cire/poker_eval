use crate::web::{
    PlayerFlopResults, PlayerPreFlopState, PreflopPlayerInfo, ResultType, MAX_PLAYERS,
};
use crate::{
    rank_cards, Card, HoleCards, OldRank, PokerError, NUM_RANK_FAMILIES, SIMPLE_RANGE_INDEX_LEN,
};

pub struct RankResults {
    pub(crate) num_iterations: ResultType,

    //win = 1, tie = 1 / num players in tie, loss = 0
    pub(crate) win_eq: f64,
    pub(crate) tie_eq: f64,

    //Also track equity by the simplified hole card range index
    pub(crate) eq_by_range_index: Vec<f64>,
    pub(crate) num_it_by_range_index: Vec<ResultType>,

    //This is a win or lose tally of hand ranks, [0] for high card .. [8] for straight flush
    pub(crate) rank_family_count: [ResultType; NUM_RANK_FAMILIES],
}

impl Default for RankResults {
    fn default() -> Self {
        Self {
            eq_by_range_index: vec![0.0; SIMPLE_RANGE_INDEX_LEN],
            num_it_by_range_index: vec![0; SIMPLE_RANGE_INDEX_LEN],
            rank_family_count: [0; NUM_RANK_FAMILIES],
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
    //hash_func: &Mphf<u32>
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

    let mut hand_evals: Vec<OldRank> = Vec::with_capacity(n_players);

    for (active_index, (_p_idx, p)) in active_players.iter().enumerate() {
        assert!(p.state != PlayerPreFlopState::Disabled);

        //For players with ranges we already chose their cards

        player_cards[active_index].add_to_eval(eval_cards);

        let rank = rank_cards(eval_cards.iter());

        update_results_from_rank(
            &mut flop_results[active_index].street_rank_results[street_index],
            rank,
        );

        flop_results[active_index].street_rank_results[street_index].num_it_by_range_index
            [player_cards[active_index].to_simple_range_index()] += 1;

        hand_evals.push(rank);

        player_cards[active_index].remove_from_eval(eval_cards)?;
    }

    //Best villian hand
    let best_villian_rank = hand_evals[1..]
        .iter()
        .fold(OldRank::HighCard(0), |acc, &x| acc.max(x));
    update_results_from_rank(
        &mut villian_results.street_rank_results[street_index],
        best_villian_rank,
    );

    let winner_indexes = indices_of_max_values(&hand_evals);

    assert!(winner_indexes.len() > 0);

    for winner_idx in winner_indexes.iter() {
        let results = &mut flop_results[*winner_idx].street_rank_results[street_index];
        if winner_indexes.len() == 1 {
            results.win_eq += 1.0;
            if *winner_idx > 0 {
                villian_results.street_rank_results[street_index].win_eq += 1.0;
            }
        } else {
            results.tie_eq += 1.0 / winner_indexes.len() as f64;

            if *winner_idx > 0 {
                villian_results.street_rank_results[street_index].tie_eq +=
                    1.0 / winner_indexes.len() as f64;
            }
        }

        //Update equity by range index
        let range_index = player_cards[*winner_idx].to_simple_range_index();
        results.eq_by_range_index[range_index] += 1.0 / winner_indexes.len() as f64;
    }

    Ok(())
}

pub(crate) fn update_results_from_rank(results: &mut RankResults, rank: OldRank) {
    results.num_iterations += 1;
    results.rank_family_count[rank.get_family_index()] += 1;
}

//returns winners and how many players were considered (non None rank)
pub(crate) fn indices_of_max_values(arr: &[OldRank]) -> Vec<usize> {
    let mut max_indices = Vec::with_capacity(MAX_PLAYERS);
    let mut max_value = OldRank::HighCard(0);

    for (index, &value) in arr.iter().enumerate() {
        if value > max_value {
            max_value = value;
            max_indices.clear();
            max_indices.push(index);
        } else if value == max_value {
            max_indices.push(index);
        }
    }

    max_indices
}
