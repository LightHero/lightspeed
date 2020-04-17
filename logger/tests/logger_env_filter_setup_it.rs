use lightspeed_logger::config::LoggerConfig;
use lightspeed_logger::setup_logger;
use log::*;

mod inner1 {
    use super::*;
    pub fn log_smt() {
        debug!("inner1 - this is debug");
        info!("inner1 - this is info");
        warn!("inner1 - this is warn");
    }
}

mod inner2 {
    use super::*;
    pub fn log_smt() {
        debug!("inner2 - this is debug");
        info!("inner2 - this is info");
        warn!("inner2 - this is warn");
    }
}

#[test]
fn should_setup_logger_with_env_filter() {
    let config = LoggerConfig {
        stdout_output: true,
        level: "info".to_owned(),
        env_filter: Some(
            "logger_env_filter_setup_it::inner1=debug,logger_env_filter_setup_it::inner2=warn"
                .to_owned(),
        ),
    };
    setup_logger(&config).unwrap();

    debug!("main - this is debug");
    info!("main - this is info");
    warn!("main - this is warn");
    inner1::log_smt();
    inner2::log_smt();
}
