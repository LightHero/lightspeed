use lightspeed_logger::config::{LoggerConfig, StandardOutputConfig, FileOutputConfig, Rotation};
use lightspeed_logger::setup_logger;

mod inner1 {
    pub async fn log_smt() {
        let yaks = 2;
        let span = tracing::span!(tracing::Level::WARN, "shaving_yaks", yaks);
        let _enter = span.enter();

        log::debug!("inner1 - this is debug");
        tracing::info!("inner1 - this is info");
        log::warn!("inner1 - this is warn. Yaks {}", yaks);
    }
}

mod inner2 {
    use super::*;

    #[tracing::instrument(skip(data), fields(id=data.id, show=true))]
    pub async fn log_smt(yak: u32, data: Data) {
        log::debug!("inner2 - id: {} - this is debug", data.id);
        tracing::info!("inner2 - id: {} - this is info", data.id);
        log::warn!("inner2 - id: {} - this is warn. Yak {}", data.id, yak);

        // info!(excitement = "yay!", "hello! I'm gonna shave a yak.");

        crate::inner1::log_smt().await;
    }
}

pub struct Data {
    id: u32,
}

#[tokio::test]
async fn should_setup_logger_with_env_filter() -> Result<(), std::io::Error> {
    let config = LoggerConfig {
        stdout_output: StandardOutputConfig {
           stdout_enabled: true,
            stdout_use_ansi_colors: true
        },
        env_filter: "debug".to_owned(),
        file_output_path: FileOutputConfig {
            file_output_directory: "../target".to_owned(),
            file_output_enabled: true,
            file_output_name_prefix: "logger.log".to_owned(),
            file_output_rotation: Rotation::Daily,
            file_output_use_ansi_colors: false
        }
    };
    let _guard = setup_logger(&config).unwrap();

    log::debug!("main - this is debug");
    tracing::info!("main - this is info");
    log::warn!("main - this is warn");
    inner1::log_smt().await;
    inner2::log_smt(3, Data { id: 789 }).await;

    Ok(())
}
