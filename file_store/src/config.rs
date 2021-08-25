use std::error::Error;
use clap::Clap;

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), String>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse().map_err(|err| format!("{:?}", err))?, s[pos + 1..].parse().map_err(|err| format!("{:?}", err))?))
}

#[derive(Debug, Clone, Clap)]
#[clap(rename_all = "kebab-case")]
#[clap(setting = clap::AppSettings::AllowExternalSubcommands)]
pub struct FileStoreConfig {
    /// The base folder used in case of 'FS' FileStoreType.
    #[clap(long, env = "LS_FILE_STORE_FS_REP_BASE_FOLDERS", parse(try_from_str = parse_key_val))]
    pub fs_repo_base_folders: Vec<(String, String)>,
}

impl FileStoreConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        FileStoreConfig::build();
    }

    #[test]
    fn should_parse() {
        let (key, value) = parse_key_val::<String, String>("my_key=my_value").unwrap();
        assert_eq!("my_key", key);
        assert_eq!("my_value", value);
    }
}
