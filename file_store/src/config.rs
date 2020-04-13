use crate::repository::FileStoreType;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct FileStoreConfig {
    /// The FileStoreType. It can be 'FS' for filesystem or 'DB' for database
    #[structopt(long, env = "LS_FILE_STORE_TYPE", default_value = "DB")]
    pub file_store_type: FileStoreType,

    /// The base folder used in case of 'FS' FileStoreType.
    #[structopt(long, env = "LS_FILE_STORE_FS_BASE_FOLDER")]
    pub file_store_fs_base_folder: Option<String>,
}

impl FileStoreConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        FileStoreConfig::build();
    }
}
