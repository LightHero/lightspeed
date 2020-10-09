use lightspeed_core::config::CoreConfig;
use lightspeed_core::module::Module;
use lightspeed_logger::config::LoggerConfig;
use lightspeed_logger::setup_logger;
use oregold_cms::config::OregoldCmsConfig;
use oregold_cms::OregoldCmsModule;
use oregold_core::config::OregoldCoreConfig;
use oregold_core::OregoldCoreModule;
use oregold_sms::config::OregoldSmsConfig;
use oregold_sms::repository::SmsRepositoryType;
use oregold_sms::OregoldSmsModule;

mod service;

#[allow(dead_code)]
async fn init() -> OregoldSmsModule {
    let conf = LoggerConfig { env_filter: String::from("debug"), stdout_output: true };
    if let Err(err) = setup_logger(&conf) {
        println!("Warning: {}", err)
    };

    let mut core_config = CoreConfig::build();
    core_config.jwt.secret = "secret".to_owned();

    let oregold_core_config = OregoldCoreConfig::build();

    let mut oregold_core_module = OregoldCoreModule::new(core_config, oregold_core_config).unwrap();
    oregold_core_module.start().await.unwrap();

    let mut cms_module = OregoldCmsModule::new(&oregold_core_module.core_config.global, OregoldCmsConfig::build()).unwrap();
    cms_module.start().await.unwrap();

    let mut sms_config = OregoldSmsConfig::build();
    sms_config.sms_repository_type = SmsRepositoryType::NoOps;

    let mut sms_module = OregoldSmsModule::new(sms_config, &oregold_core_module, &cms_module).unwrap();
    sms_module.start().await.unwrap();

    sms_module
}
