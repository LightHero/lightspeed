use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FileStoreConfig {
    /// The base folder used in case of 'FS' FileStoreType.
    #[serde(default)]
    pub fs_repo_base_folders: Vec<(String, String)>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FsStore {
    pub key: String,
    pub folder: String,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let _config: FileStoreConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
    }
}
