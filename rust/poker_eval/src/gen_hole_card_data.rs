use std::{
    cmp::{max, min},
    fs::File,
    io::Write,
    time::Instant,
};

use log::info;
use poker_eval::{
    board_hc_eval_cache_redb::{EvalCacheWithHcReDb, ProduceMonteCarloEval},
    init_logger,
    monte_carlo_equity::get_equivalent_hole_board,
    pre_calc::get_data_file_path,
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

    let mut check_index = 0;

    let mut monte_carlo_equity_db: EvalCacheWithHcReDb<ProduceMonteCarloEval> =
        EvalCacheWithHcReDb::new().unwrap();

    //let rcref_mcedb = Rc::new(RefCell::new(monte_carlo_equity_db));

    let mut deck = Deck::new();

    //let p = get_data_file_path("hole_card_data_river_4.csv");
    let num_players = 4;
    //let round = Round::Flop;
    let round = Round::River;
    let p = get_data_file_path(&format!("hole_card_data_{}_{}.csv", round, num_players));

    let cards_needed = match round {
        Round::Flop => 3,
        Round::Turn => 4,
        Round::River => 5,
        _ => panic!("Invalid round"),
    };

    let mut wtr = File::create(p).unwrap();

    let num_hands_to_simulate = 10_000;

    let mut last_output = Instant::now();

    for row in 0..13u8 {
        for col in 0..13 {
            let mut line_values = Vec::with_capacity(150);
            let hi_card = CardValue::try_from(12 - min(row, col))?;
            let lo_card = CardValue::try_from(12 - max(row, col))?;
            let is_suited = col > row;

            let card1 = Card::new(hi_card, Suit::Club);
            let card2 = Card::new(lo_card, if is_suited { Suit::Club } else { Suit::Diamond });
            let hole_cards = HoleCards::new(card1, card2)?;

            info!(
                "row {} col {} -- index {}: {}",
                row, col, check_index, hole_cards
            );

            assert_eq!(check_index, hole_cards.to_simple_range_index());

            line_values.push(format!("{}", check_index));
            line_values.push(format!("{}", hole_cards));

            check_index += 1;

            for i in 0..num_hands_to_simulate {
                if i % 100 == 0 && last_output.elapsed().as_secs() > 3 {
                    info!(
                        "{} hands simulated for #{}: {}",
                        i,
                        check_index - 1,
                        hole_cards
                    );
                    last_output = Instant::now();
                }

                deck.reset();

                deck.set_used_card(hole_cards.get_hi_card());
                deck.set_used_card(hole_cards.get_lo_card());

                let mut board = Board::new();
                for _ in 0..cards_needed {
                    let card = deck.get_unused_card()?;
                    board.add_card(card)?;
                }

                let (eq_hole_cards, mut eq_board) = get_equivalent_hole_board(&hole_cards, board.as_slice_card());
                eq_board.get_index();
                //board.get_index();

                //let eq = monte_carlo_equity_db.get_put(&board, &hole_cards, 4).unwrap();
                let eq = monte_carlo_equity_db
                    .get_put(&eq_board, &eq_hole_cards, 4)
                    .unwrap();
                //info!("eq {}", eq);
                line_values.push(format!("{}", eq));
            }

            line_values.push("\n".to_string());

            wtr.write_all(line_values.join(",").as_bytes()).unwrap();
        }
    }

    Ok(())

    // for suited in [false, true] {

    //     for lo_card in CardValueRange::new(CardValue::Two, CardValue::Ace) {
    //         for hi_card in CardValueRange::new(lo_card, CardValue::Ace) {

    //             info!("{}{}{}", hi_card, lo_card, if suited { "s" } else { "o" });
    //         }

    //     }
    // }
}
