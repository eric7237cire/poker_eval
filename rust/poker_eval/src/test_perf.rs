use std::time::Instant;

use poker_eval::{exact_equity::calc_equity, Board, BoolRange};

fn main() {
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
