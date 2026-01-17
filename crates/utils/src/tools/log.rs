use std::io::Write;
use std::sync::Once;

static INIT: Once = Once::new();
/// because logger can only init once, so we use Once to ensure it
pub fn init_logger() {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {}:{} {}] {}",
                    chrono::Local::now()
                        .format("%Y-%m-%d %H:%M:%S"),
                    record
                        .module_path()
                        .unwrap_or("<unnamed>"),
                    record.line().unwrap_or(0),
                    record.level(),
                    record.args()
                )
            })
            .init();
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use log::info;

    #[test]
    fn test_1() -> anyhow::Result<()> {
        init_logger();
        info!("hello world");
        Ok(())
    }
}
