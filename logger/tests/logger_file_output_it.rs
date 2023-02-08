use lightspeed_logger::config::{FileOutputConfig, LoggerConfig, Rotation, StandardOutputConfig};
use lightspeed_logger::setup_logger;
use log::{debug, warn};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use tracing::info;

#[test]
fn should_setup_logger_with_env_filter() -> Result<(), std::io::Error> {
    let tempdir = tempfile::tempdir().unwrap();
    let file_output_directory = tempdir.path().to_str().unwrap().to_owned();
    let file_output_name_prefix = format!("filename_{}.log", rand::random::<u64>());
    let log_filename = format!("{file_output_directory}/{file_output_name_prefix}");

    let config = LoggerConfig {
        stdout_output: StandardOutputConfig { stdout_enabled: true, stdout_use_ansi_colors: true },
        env_filter: "debug,logger_file_output_it=info".to_owned(),
        file_output: FileOutputConfig {
            file_output_directory,
            file_output_enabled: true,
            file_output_name_prefix,
            file_output_rotation: Rotation::Never,
        },
    };
    let _guard = setup_logger(&config).unwrap();

    debug!("main - this is debug");
    info!("main - this is info");
    warn!("main - this is warn");

    let path = Path::new(&log_filename);
    assert!(path.exists());

    sleep(Duration::from_millis(250));

    let log_content = std::fs::read_to_string(path).unwrap();
    assert!(log_content.contains("main - this is info"));
    assert!(!log_content.contains("main - this is debug"));

    Ok(())
}
