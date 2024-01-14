use std::{
    cmp::{max, min},
    fs::{File, self},
    io::Write,
    time::Instant,
};

use log::info;
use poker_eval::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    init_logger,
    monte_carlo_equity::{get_equivalent_hole_board, calc_equity_vs_random},
    pre_calc::{get_data_file_path, get_repo_root},
    Board, Card, CardValue, Deck, HoleCards, PokerError, Round, Suit,
};

/*
cargo run --release --bin gen_hole_card_data
python3 /home/eric/git/poker_eval/python/try_clustering.py
*/

fn main() {
    main_impl().unwrap();
}

fn main_impl() -> Result<(), PokerError> {
    init_logger();

    for num_players in 2..=9 {
        for round in [Round::Flop, Round::Turn, Round::River].iter() {
            simulate(*round, num_players)?;
        }
    }

    Ok(())
}

fn simulate(round: Round, num_players: u8) -> Result<(), PokerError>
{

    //let round = Round::River;
    let p  = get_repo_root().join(format!("python/from_rust/hole_card_data_{}_{}.csv", round, num_players));
    
    let num_hands_to_simulate = 10_000;

    info!("Creating {:?} for round {}, {} players {} simulations", &p, round, num_players, num_hands_to_simulate);
    
    fs::create_dir_all( p.parent().unwrap() ).unwrap();

    let mut wtr = File::create(p).unwrap();

    

    let board = Board::new();

    for row in 0..13u8 {
        for col in 0..13 {
            let mut line_values = Vec::with_capacity(150);
            let hi_card = CardValue::try_from(12 - min(row, col))?;
            let lo_card = CardValue::try_from(12 - max(row, col))?;
            let is_suited = col > row;

            let card1 = Card::new(hi_card, Suit::Club);
            let card2 = Card::new(lo_card, if is_suited { Suit::Club } else { Suit::Diamond });
            let hole_cards = HoleCards::new(card1, card2)?;

            // info!(
            //     "row {} col {} -- {}",
            //     row, col, hole_cards
            // );

            line_values.push(format!("{}", hole_cards.to_simple_range_index()));
            line_values.push(format!("{}", hole_cards));

            let eq = calc_equity_vs_random(&board, &hole_cards, num_players as usize, num_hands_to_simulate, round.get_num_board_cards())?;
               
            line_values.push(format!("{}", eq));

            line_values.push("\n".to_string());

            wtr.write_all(line_values.join(",").as_bytes()).unwrap();
        }
    }

    Ok(())

   
}
