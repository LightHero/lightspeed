use crate::error::LightSpeedError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter};

#[derive(Clone, Debug, Display, EnumIter, AsRefStr, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "poem_openapi_", derive(poem_openapi::Enum))]
pub enum Language {
    De,
    En,
    Es,
    Fr,
    It,
}

impl FromStr for Language {
    type Err = LightSpeedError;

    fn from_str(language: &str) -> Result<Self, Self::Err> {
        match language.to_lowercase().as_ref() {
            "de" => Ok(Language::De),
            "en" => Ok(Language::En),
            "es" => Ok(Language::Es),
            "fr" => Ok(Language::Fr),
            "it" => Ok(Language::It),
            _ => {
                Err(LightSpeedError::ConfigurationError { message: format!("Could not parse language [{}]", language) })
            }
        }
    }
}
