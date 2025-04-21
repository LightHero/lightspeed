use lightspeed_file_store::config::{FileStoreConfig, RepositoryType};
use opendal::{services::Fs, Operator};

pub mod repository;
pub mod service;

pub fn get_config() -> FileStoreConfig { 
    let mut file_store_config = FileStoreConfig::default();
    file_store_config.repositories.insert(
        "FS_ONE".to_owned(),
        Operator::new(Fs::default().root("../target/repo_one")).unwrap().finish().into(),
    );
    file_store_config.repositories.insert(
        "FS_TWO".to_owned(),
        Operator::new(Fs::default().root("../target/repo_two")).unwrap().finish().into(),
    );
    file_store_config.repositories.insert(
        "DB_ONE".to_owned(),
        RepositoryType::DB,
    );
    file_store_config.repositories.insert(
        "DB_TWO".to_owned(),
        RepositoryType::DB,
    );

    file_store_config
} 