use poker_rs::{core::Hand, holdem::MonteCarloGame};


const GAMES_COUNT: i32 = 300_000;
const STARTING_HANDS: [&str; 2] = ["Adkh", "8c8s"];

fn main() {
    println!("Hello, world!");
    
    let hands = STARTING_HANDS
        .iter()
        .map(|s| Hand::new_from_str(s).expect("Should be able to create a hand."))
        .collect();
    let mut g = MonteCarloGame::new(hands).expect("Should be able to create a game.");
    let mut wins: [u64; 2] = [0, 0];
    for _ in 0..GAMES_COUNT {
        let r = g.simulate();
        g.reset();
        wins[r.0.ones().next().unwrap()] += 1
    }

    let normalized: Vec<f64> = wins
        .iter()
        .map(|cnt| *cnt as f64 / GAMES_COUNT as f64)
        .collect();

    println!("Starting Hands =\t{:?}", STARTING_HANDS);
    println!("Wins =\t\t\t{:?}", wins);
    println!("Normalized Wins =\t{:?}", normalized);
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use poker_rs::core::{Hand, Rankable, Rank};

    #[test]
    fn test_3rd_kicker() {
        //AA pair, ten, nine, four vs 5
        let hand1 =  Hand::new_from_str("AdAcTd9d2h3c4d").unwrap().rank();
        let hand2 =  Hand::new_from_str("AdAcTd9d2h3c5h").unwrap().rank();

        assert_eq!(hand2.cmp(&hand1), Ordering::Greater);
        if let Rank::OnePair(_val) = hand2 {
            assert!(true);
        } else {
            assert!(false);
        }

        //Test 4th kicker doesn't matter 

        //K Q T 9 8 7 vs 6
        let hand1 = Hand::new_from_str("KdQcTd9d8h7c3d2c").unwrap().rank();

        let hand2 = Hand::new_from_str("KhQcTd9d8h6c3d5c").unwrap().rank();

        assert_eq!(hand2.cmp(&hand1), Ordering::Equal);

        assert_eq!(1, 2);

        
    }
}