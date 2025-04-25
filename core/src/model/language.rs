use crate::error::LightSpeedError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter};

#[derive(Clone, Debug, Display, EnumIter, AsRefStr, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Language {
    DE,
    EN,
    ES,
    FR,
    IT,
}

impl FromStr for Language {
    type Err = LightSpeedError;

    fn from_str(language: &str) -> Result<Self, Self::Err> {
        match language.to_lowercase().as_ref() {
            "de" => Ok(Language::DE),
            "en" => Ok(Language::EN),
            "es" => Ok(Language::ES),
            "fr" => Ok(Language::FR),
            "it" => Ok(Language::IT),
            _ => {
                Err(LightSpeedError::ConfigurationError { message: format!("Could not parse language [{}]", language) })
            }
        }
    }
}
