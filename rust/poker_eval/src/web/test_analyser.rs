
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{card_u8s_from_string, web::{PlayerPreFlopState, flop_analyzer}, Rank, HoleCards};

    fn assert_equity(equity: f64, target: f64, tolerance: f64) {
        let passed = (equity - target).abs() < tolerance;
        if !passed {
            println!("assert_equity failed: {} != {}", equity, target);
        }
        assert!(passed);
    }

    #[test]
    fn test_heads_up_both_with_hole_cards() {
        let mut analyzer = flop_analyzer::new();
        analyzer.reset();
        let tolerance = 0.1;

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer
            .set_player_cards(0, card_u8s_from_string("7h 6s").as_slice())
            .unwrap();

        analyzer
            .set_player_cards(3, card_u8s_from_string("Th 9h").as_slice())
            .unwrap();

        analyzer
            .set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice())
            .unwrap();

        let num_it = 10_000;
        let f_results = analyzer.build_results();
        let f_results = analyzer.simulate_flop(num_it, f_results).unwrap();

        let results = &f_results.flop_results;

        // let results = analyzer
        //     .player_info
        //     .iter()
        //     .map(|p| &p.results)
        //     .collect_vec();

        assert_eq!(results[0].street_rank_results[2].num_iterations, num_it);
        assert_eq!(0, results[0].player_index);
        assert_equity(
            100.0 * results[0].street_rank_results[2].win_eq / num_it as f64,
            21.92,
            tolerance,
        );

        assert_eq!(results[1].street_rank_results[2].num_iterations, num_it);
        assert_eq!(3, results[1].player_index);

        assert_equity(
            100.0 * results[1].street_rank_results[2].win_eq / num_it as f64,
            78.08,
            tolerance,
        );
    }

    #[test]
    fn test_3way_with_ranges() {
        let mut analyzer = flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseRange as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);

        analyzer
            .set_player_cards(0, card_u8s_from_string("8d 7s").as_slice())
            .unwrap();

        analyzer
            .set_player_cards(3, card_u8s_from_string("Qd 5c").as_slice())
            .unwrap();

        analyzer.set_player_range(
            2,
            "22+, A2s+, K2s+, Q2s+, J6s+, 94s, A2o+, K7o+, QJo, J7o, T4o",
        ).unwrap();

        analyzer
            .set_board_cards(card_u8s_from_string("Qs Ts 7c").as_slice())
            .unwrap();

        let num_it = 4_000;

        let tolerance = 0.5;
        //let tolerance = 0.1;

        let f_results = analyzer.build_results();
        let f_results = analyzer.simulate_flop(num_it, f_results).unwrap();

        let results = &f_results.flop_results;

        assert_eq!(results[0].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[0].player_index, 0);

        assert_equity(
            100.0 * results[0].street_rank_results[2].win_eq
                / results[0].street_rank_results[2].num_iterations as f64,
            21.03,
            tolerance,
        );
        assert_equity(
            100.0 * results[0].street_rank_results[2].tie_eq
                / results[0].street_rank_results[2].num_iterations as f64,
            0.12,
            0.05,
        );

        assert_eq!(results[2].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[2].player_index, 3);

        // assert_equity(
        //     100.0 * results[3].eq_not_folded / not_folded as f64,
        //     50.93 + 0.82,
        //     0.7,
        // );
        assert_equity(
            100.0 * results[2].street_rank_results[2].win_eq / num_it as f64,
            50.93,
            tolerance,
        );
        assert_equity(
            100.0 * results[2].street_rank_results[2].tie_eq / num_it as f64,
            0.82,
            tolerance,
        );

        assert_eq!(results[1].street_rank_results[2].num_iterations, num_it);
        assert_eq!(results[1].player_index, 2);
        //let not_folded = results[3].num_iterations;

        assert_equity(
            100.0 * results[1].street_rank_results[2].win_eq / num_it as f64,
            26.14,
            tolerance,
        );
        assert_equity(
            100.0 * results[1].street_rank_results[2].tie_eq / num_it as f64,
            0.95,
            tolerance,
        );
    }

    #[test]
    fn test_villian_draws() {
        let mut analyzer = flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(4, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseHoleCards as u8);

        analyzer
            .set_player_cards(0, card_u8s_from_string("Td 8s").as_slice())
            .unwrap();

        analyzer
            .set_player_cards(3, card_u8s_from_string("Ad Kc").as_slice())
            .unwrap();
        analyzer
            .set_player_cards(4, card_u8s_from_string("5s 5c").as_slice())
            .unwrap();
        analyzer
            .set_player_cards(2, card_u8s_from_string("Qd 7d").as_slice())
            .unwrap();

        analyzer
            .set_board_cards(card_u8s_from_string("9s 8c Ah 5h 6h").as_slice())
            .unwrap();

        let num_it = 1;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

        let v_r = &results.all_villians;
        assert_eq!(
            1,
            v_r.street_rank_results[0].rank_family_count[Rank::OnePair(0).get_family_index()]
        );
        assert_eq!(
            1u32,
            v_r.street_rank_results[0].rank_family_count.iter().sum()
        );
        assert_eq!(0, v_r.street_draws[0].gut_shot);
        assert_eq!(0, v_r.street_draws[0].two_overcards);
        assert_eq!(0, v_r.street_draws[0].one_overcard);
        assert_eq!(
            1.0,
            v_r.street_rank_results[0].win_eq / v_r.street_rank_results[0].num_iterations as f64
        );

        //Turn villian picks up gut shot
        assert_eq!(
            1,
            v_r.street_rank_results[1].rank_family_count[Rank::ThreeOfAKind(0).get_family_index()]
        );
        assert_eq!(
            1u32,
            v_r.street_rank_results[1].rank_family_count.iter().sum()
        );
        assert_eq!(1, v_r.street_draws[1].gut_shot);
        assert_eq!(0, v_r.street_draws[1].two_overcards);
        assert_eq!(0, v_r.street_draws[1].one_overcard);

        assert_eq!(
            0,
            v_r.street_rank_results[2].rank_family_count[Rank::OnePair(0).get_family_index()]
        );
        assert_eq!(
            1,
            v_r.street_rank_results[2].rank_family_count[Rank::Straight(0).get_family_index()]
        );
        assert_eq!(
            1u32,
            v_r.street_rank_results[2].rank_family_count.iter().sum()
        );
        assert_eq!(2, v_r.street_draws.len());
    }

    #[test]
    fn test_villian_overcards() {
        let mut analyzer = flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(4, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(2, PlayerPreFlopState::UseHoleCards as u8);

        analyzer
            .set_player_cards(0, card_u8s_from_string("Tc 8s").as_slice())
            .unwrap();

        analyzer
            .set_player_cards(3, card_u8s_from_string("Ad Jc").as_slice())
            .unwrap();
        analyzer
            .set_player_cards(4, card_u8s_from_string("Ks Qc").as_slice())
            .unwrap();
        analyzer
            .set_player_cards(2, card_u8s_from_string("Jd Td").as_slice())
            .unwrap();

        analyzer
            .set_board_cards(card_u8s_from_string("2s 4c 7h Qh Ah").as_slice())
            .unwrap();

        let num_it = 1;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();

        let v_r = &results.all_villians;
        assert_eq!(
            1,
            v_r.street_rank_results[0].rank_family_count[Rank::HighCard(0).get_family_index()]
        );
        assert_eq!(
            1,
            v_r.street_rank_results[0]
                .rank_family_count
                .iter()
                .sum::<u32>()
        );
        assert_eq!(1, v_r.street_draws[0].two_overcards);
        assert_eq!(0, v_r.street_draws[0].one_overcard);

        assert_eq!(
            1,
            v_r.street_rank_results[1].rank_family_count[Rank::OnePair(0).get_family_index()]
        );
        assert_eq!(
            1,
            v_r.street_rank_results[1]
                .rank_family_count
                .iter()
                .sum::<u32>()
        );
        assert_eq!(0, v_r.street_draws[1].two_overcards);
        assert_eq!(1, v_r.street_draws[1].one_overcard);

        assert_eq!(
            1,
            v_r.street_rank_results[2].rank_family_count[Rank::OnePair(0).get_family_index()]
        );
        assert_eq!(
            1,
            v_r.street_rank_results[2]
                .rank_family_count
                .iter()
                .sum::<u32>()
        );
        assert_eq!(2, v_r.street_draws.len());
    }

    #[test]
    fn test_range_equity() {
        let mut analyzer = flop_analyzer::new();
        analyzer.reset();

        analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
        analyzer.set_player_state(3, PlayerPreFlopState::UseRange as u8);
        analyzer.set_player_state(4, PlayerPreFlopState::UseRange as u8);
        

        analyzer
            .set_player_cards(0, card_u8s_from_string("2d 8s").as_slice())
            .unwrap();

        analyzer
            .set_player_range(3, "87, KJo, T2s")
                        .unwrap();
        analyzer
        .set_player_range(4, "22, 99")
            .unwrap();
        
        // analyzer
        //     .set_board_cards(card_u8s_from_string("9s 8c Ah 5h 6h").as_slice())
        //     .unwrap();

        let num_it = 200;

        let results = analyzer.build_results();
        let results = analyzer.simulate_flop(num_it, results).unwrap();


        let kj_index = HoleCards::from_str("Kd Jc").unwrap().to_simple_range_index();
        let t2_index = HoleCards::from_str("Th 2h").unwrap().to_simple_range_index();
        let e7_index = HoleCards::from_str("8d 7c").unwrap().to_simple_range_index();
        let e7s_index = HoleCards::from_str("8d 7d").unwrap().to_simple_range_index();
        
        assert_eq!(3, results.flop_results[1].player_index);

        //results are only active players
        let p3_results = &results.flop_results[1].street_rank_results[0];

        assert_eq!(num_it, p3_results.num_iterations);

        //get total equity for p3
        let total_eq = p3_results.win_eq + p3_results.tie_eq;

        let total_of_4 = p3_results.win_or_tie_eq_by_range_index[kj_index] + p3_results.win_or_tie_eq_by_range_index[t2_index] + p3_results.win_or_tie_eq_by_range_index[e7_index]
        + p3_results.win_or_tie_eq_by_range_index[e7s_index];

        println!("total_eq {} total_of_4 {}", total_eq, total_of_4);
        assert!( (total_eq - total_of_4).abs() < 0.000001);
        
    }
}
