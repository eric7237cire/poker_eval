use std::io::Write;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .try_init();
}
