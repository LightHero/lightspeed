use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CmsConfig {}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let _config: CmsConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
    }
}
