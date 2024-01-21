use std::io::Write;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .filter_module("poker_eval::game::game_log_parser", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::game_log_source", log::LevelFilter::Debug)
        .filter_module("poker_eval::game::runner", log::LevelFilter::Debug)
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
        //.filter_module("poker_eval::game::runner", log::LevelFilter::Trace)        
        .filter_module("poker_eval", log::LevelFilter::Debug)
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
