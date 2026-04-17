use env_logger::Env;
use log::LevelFilter;

pub fn setup() {
    let env = Env::new().filter("ECCHAN_LOG").write_style("ECCHAN_STYLE");

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .parse_env(env)
        .init();
}
