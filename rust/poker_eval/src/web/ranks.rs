use crate::{
    rank_cards, Card, HoleCards, PlayerFlopResults, PlayerPreFlopState, PokerError,
    PreflopPlayerInfo, Rank, RankResults, MAX_PLAYERS,
};

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

        let rank = rank_cards(&eval_cards);

        update_results_from_rank(
            &mut flop_results[active_index].street_rank_results[street_index],
            rank,
        );

        hand_evals.push(rank);

        player_cards[active_index].remove_from_eval(eval_cards)?;
    }

    //Best villian hand
    let best_villian_rank = hand_evals[1..]
        .iter()
        .fold(Rank::HighCard(0), |acc, &x| acc.max(x));
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
    }

    Ok(())
}

pub(crate) fn update_results_from_rank(results: &mut RankResults, rank: Rank) {
    results.num_iterations += 1;
    results.rank_family_count[rank.get_family_index()] += 1;
}

//returns winners and how many players were considered (non None rank)
pub(crate) fn indices_of_max_values(arr: &[Rank]) -> Vec<usize> {
    let mut max_indices = Vec::with_capacity(MAX_PLAYERS);
    let mut max_value = Rank::HighCard(0);

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
