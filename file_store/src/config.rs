use std::error::Error;
use structopt::StructOpt;

// See: https://github.com/TeXitoi/structopt/blob/master/examples/keyvalue.rs
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct FileStoreConfig {
    /// The base folder used in case of 'FS' FileStoreType.
    #[structopt(long, env = "LS_FILE_STORE_FS_REP_BASE_FOLDERS", parse(try_from_str = parse_key_val))]
    pub fs_repo_base_folders: Vec<(String, String)>,
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

    #[test]
    fn should_parse() {
        let (key, value) = parse_key_val::<String, String>("my_key=my_value").unwrap();
        assert_eq!("my_key", key);
        assert_eq!("my_value", value);
    }
}
