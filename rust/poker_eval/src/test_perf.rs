use std::time::Instant;

use poker_eval::{monte_carlo_equity::calc_equity, Board, BoolRange};
use poker_eval::web::{PlayerPreFlopState, flop_analyzer};

fn main2() {
    let board: Board = "9d 8h 9c".parse().unwrap();

    let start = Instant::now();

    let ranges: Vec<BoolRange> = vec![
            "Ks6s".parse().unwrap(),
            "22+,A2s+,K2s+,Q2s+,J2s+,T2s+,92s+,82s+,74s+,64s+,54s,A2o+,K2o+,Q2o+,J2o+,T2o+,94o+,85o+,75o+".parse().unwrap(),
            "33+,A2s+,K3s+,Q6s+,J8s+,T9s,A2o+,K6o+,Q8o+,JTo".parse().unwrap(),
        ];

    //let rank_db: EvalCacheReDb<ProduceRank> = EvalCacheReDb::new().unwrap();

    //let shared = Rc::new(RefCell::new(rank_db));

    let results = calc_equity(&board, &ranges, 10_000_000).unwrap();

    for i in 0..ranges.len() {
        println!("{}\n{:.2}", ranges[i].to_string(), results[i] * 100.0);
    }

    println!("time {:?}", start.elapsed());
}

fn main() {
    let mut analyzer = flop_analyzer::new();
    analyzer.reset();

    analyzer.set_player_state(0, PlayerPreFlopState::UseHoleCards as u8);
    analyzer.set_player_state(1, PlayerPreFlopState::UseRange as u8);
    analyzer.set_player_state(2, PlayerPreFlopState::UseRange as u8);

    analyzer
        .set_player_cards(0, &Board::try_from("9d 8h 9cs").unwrap().as_vec_u8())
        .unwrap();

    
    analyzer.set_player_cards(0, &Board::try_from("Ks 6s").unwrap().as_vec_u8()).unwrap();
    analyzer.set_player_range(1, "22+,A2s+,K2s+,Q2s+,J2s+,T2s+,92s+,82s+,74s+,64s+,54s,A2o+,K2o+,Q2o+,J2o+,T2o+,94o+,85o+,75o+").unwrap();
    analyzer.set_player_range(2, "33+,A2s+,K3s+,Q6s+,J8s+,T9s,A2o+,K6o+,Q8o+,JTo").unwrap();

    let num_it = 10_000;
    let results = analyzer.build_results();
    let _results = analyzer.simulate_flop(num_it, results, true).unwrap();

}
