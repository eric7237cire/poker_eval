use std::io::Write;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                take_after_last_slash(record.file().unwrap_or("unknown")),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .try_init();
}

fn take_after_last_slash(s: &str) -> &str {
    let mut last_slash = 0;
    for (i, c) in s.chars().enumerate() {
        if c == '/' {
            last_slash = i;
        }
    }
    &s[last_slash + 1..]
}
