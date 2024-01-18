use std::io::Write;

#[cfg(not(target_arch = "wasm32"))]
use log::debug;

#[cfg(not(target_arch = "wasm32"))]
use crate::game::runner::{GameLog, GameRunner, GameLogSource, GameRunnerSourceEnum,};

#[cfg(not(target_arch = "wasm32"))]
use crate::PokerError;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .filter_module("poker_eval::game::game_log_parser", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::game_log_source", log::LevelFilter::Debug)
        //.filter_module("poker_eval::game::game_runner", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::game_runner", log::LevelFilter::Trace)
        .filter_module(
            "poker_eval::game::agents::agent_source",
            log::LevelFilter::Debug,
        )
        .filter_module("poker_eval::game::game_log", log::LevelFilter::Trace)
        .filter_module("poker_eval::eval::rank", log::LevelFilter::Debug)
        .filter_module("poker_eval::eval::board_texture", log::LevelFilter::Debug)
        .filter_module("poker_eval::core::bool_range", log::LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                record.module_path().unwrap_or("unknown"),
                //take_after_last_slash(record.file().unwrap_or("unknown")),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .try_init();
}

pub fn init_logger() {
    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Trace)
        .filter_module("poker_eval::game::game_log_parser", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::game_log_source", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::game_runner", log::LevelFilter::Debug)
        .filter_module(
            "poker_eval::game::agents::agent_source",
            log::LevelFilter::Debug,
        )
        .filter_module("poker_eval::game::game_log", log::LevelFilter::Debug)
        .filter_module("poker_eval::eval::rank", log::LevelFilter::Debug)
        .filter_module("poker_eval::eval::board_texture", log::LevelFilter::Debug)
        .filter_module("poker_eval::core::bool_range", log::LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                record.module_path().unwrap_or("unknown"),
                //take_after_last_slash(record.file().unwrap_or("unknown")),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .try_init();
}

#[allow(dead_code)]
fn take_after_last_slash(s: &str) -> &str {
    let mut last_slash = 0;
    for (i, c) in s.chars().enumerate() {
        if c == '/' {
            last_slash = i;
        }
    }
    &s[last_slash + 1..]
}

#[cfg(not(target_arch = "wasm32"))]
pub fn test_game_runner(game_runner: &mut GameRunner) -> Result<(), PokerError> {
    for _ in 0..200 {
        let action_count_before = game_runner.game_state.actions.len();
        let r = game_runner.process_next_action()?;
        if r {
            break;
        }
        let action_count_after = game_runner.game_state.actions.len();
        // debug!(
        //     "Last action: {}",
        //     &game_runner.game_state.actions.last().as_ref().unwrap()
        // );
        assert_eq!(action_count_before + 1, action_count_after);
    }

    //let log_display = game_runner.to_game_log_string(true);
    let game_log = game_runner.to_game_log().unwrap();
    let check_log = game_log.to_game_log_string(false, false, 0);
    debug!("log\n{}", check_log);

    let parsed_game_log: GameLog = check_log.parse().unwrap();
    let game_log_source: GameLogSource = GameLogSource::new(parsed_game_log);

    //Run the game again with the log
    let mut game_runner2 = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

    for _ in 0..200 {
        let action_count_before = game_runner2.game_state.actions.len();
        let r = game_runner2.process_next_action()?;
        if r {
            break;
        }
        let action_count_after = game_runner2.game_state.actions.len();
        assert_eq!(action_count_before + 1, action_count_after);
    }

    let log2 = game_runner2
        .to_game_log()
        .unwrap()
        .to_game_log_string(false, false, 0);

    //info!("log2:\n{}", log2);

    assert_eq!(check_log, log2);

    Ok(())
}
