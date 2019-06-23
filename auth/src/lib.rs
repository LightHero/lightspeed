pub mod service;

#[cfg(test)]
pub mod test_root {

    use lazy_static::lazy_static;
    use std::sync::Mutex;
    use ls_logger::config::LoggerConfig;
    use ls_logger::setup_logger;

    lazy_static! {
        static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
    }

    pub fn init_context() {
        let mut init = INITIALIZED.lock().unwrap();
        if !*init {
            println!("Initialize context");
            start_logger();
            *init = true;
        }
    }

    fn start_logger() {
        println!("Init logger");

        let conf = LoggerConfig {
            level: String::from("trace"),
            stdout_output: true,
            stderr_output: false,
            file_output_path: None,
        };
        setup_logger(&conf).unwrap();
    }

}
