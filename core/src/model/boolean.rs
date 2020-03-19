use crate::error::LightSpeedError;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Boolean {
    True,
    False,
}

impl From<bool> for Boolean {
    fn from(val: bool) -> Self {
        if val {
            Boolean::True
        } else {
            Boolean::False
        }
    }
}

impl From<Boolean> for bool {
    fn from(val: Boolean) -> Self {
        val.value()
    }
}

impl Boolean {
    pub fn value(&self) -> bool {
        match self {
            Boolean::True => true,
            Boolean::False => false,
        }
    }
}

impl FromStr for Boolean {
    type Err = LightSpeedError;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val.to_lowercase().as_ref() {
            "true" => Ok(Boolean::True),
            "false" => Ok(Boolean::False),
            _ => Err(LightSpeedError::ConfigurationError { message: format!("Could not parse boolean [{}]", val) }),
        }
    }
}
