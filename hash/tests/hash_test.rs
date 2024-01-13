use lightspeed_core::config::CoreConfig;
use lightspeed_core::module::LsModule;
use lightspeed_core::LsCoreModule;
use lightspeed_hash::LsHashModule;
use lightspeed_logger::config::LoggerConfig;
use lightspeed_logger::setup_logger;

mod service;

#[allow(dead_code)]
async fn init() -> LsHashModule {
    let conf = LoggerConfig::default();
    if let Err(err) = setup_logger(&conf) {
        println!("Warning: {err:?}")
    };

    let mut core_config = CoreConfig::default();
    core_config.jwt.secret = "secret".to_owned();

    let mut core_module = LsCoreModule::new(core_config).unwrap();
    core_module.start().await.unwrap();

    let mut hash_module = LsHashModule::new(&core_module).unwrap();
    hash_module.start().await.unwrap();

    hash_module
}
