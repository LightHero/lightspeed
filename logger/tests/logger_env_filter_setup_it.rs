use lightspeed_logger::config::LoggerConfig;
use lightspeed_logger::setup_logger;
use log::{debug, warn};
use tracing::{info, span, Level};

mod inner1 {
    use super::*;
    pub async fn log_smt() {
        let yaks = 2;
        let span = span!(Level::WARN, "shaving_yaks", yaks);
        let _enter = span.enter();

        debug!("inner1 - this is debug");
        info!("inner1 - this is info");
        warn!("inner1 - this is warn. Yaks {}", yaks);
    }
}

mod inner2 {
    use super::*;

    #[tracing::instrument]
    pub async fn log_smt(yak: u32) {
        debug!("inner2 - this is debug");
        info!("inner2 - this is info");
        warn!("inner2 - this is warn. Yak {}", yak);

        // info!(excitement = "yay!", "hello! I'm gonna shave a yak.");

        crate::inner1::log_smt().await;
    }
}

#[tokio::test]
async fn should_setup_logger_with_env_filter() -> Result<(), std::io::Error> {
    let config = LoggerConfig {
        stdout_output: true,
        env_filter:
            "warn,logger_env_filter_setup_it::inner1=info,logger_env_filter_setup_it::inner2=debug"
                .to_owned(),
    };
    setup_logger(&config).unwrap();

    debug!("main - this is debug");
    info!("main - this is info");
    warn!("main - this is warn");
    inner1::log_smt().await;
    inner2::log_smt(3).await;

    Ok(())
}
