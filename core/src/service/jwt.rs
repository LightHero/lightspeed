use crate::config::JwtConfig;
use crate::error::LightSpeedError;
use serde_derive::{Deserialize, Serialize};
use crate::utils::current_epoch_seconds;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWT<T> {
    pub payload: T,
    // The subject of the token
    pub sub: String,
    // The expiration date of the token
    pub exp: i64,
    // The issued at field
    pub iat: i64,
    // The token id
    //jti: String,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    token_validity_seconds: i64,
    header_default: jsonwebtoken::Header,
    validation_default: jsonwebtoken::Validation,
}

impl JwtService {
    pub fn new(jwt_config: &JwtConfig) -> JwtService {
        //let alg = alg_from_str(&jwt_config.signature_algorithm);

        let alg = jwt_config.signature_algorithm;

        JwtService {
            secret: jwt_config.secret.clone(),
            token_validity_seconds: i64::from(jwt_config.token_validity_minutes) * 60,
            header_default: jsonwebtoken::Header {
                alg,
                ..jsonwebtoken::Header::default()
            },
            validation_default: jsonwebtoken::Validation::new(alg),
        }
    }

    pub fn generate_from_payload<T: serde::ser::Serialize>(
        &self,
        payload: &T,
    ) -> Result<String, LightSpeedError> {
        let issued_at = current_epoch_seconds();
        let token = JWT {
            payload,
            sub: "".to_string(),
            exp: issued_at + self.token_validity_seconds,
            iat: issued_at,
        };
        self.generate_from_token(&token)
    }

    pub fn generate_from_token<T: serde::ser::Serialize>(
        &self,
        token: &JWT<T>,
    ) -> Result<String, LightSpeedError> {
        let result = jsonwebtoken::encode(&self.header_default, &token, &self.secret.as_ref());
        match result {
            Ok(t) => Ok(t),
            Err(e) => {
                //let err = e.to_string();
                Err(LightSpeedError::GenerateTokenError {
                    message: e.to_string(),
                })
            }
        }
    }

    pub fn parse_payload<T: serde::de::DeserializeOwned>(
        &self,
        jwt_string: &str,
    ) -> Result<T, LightSpeedError> {
        let result = self.parse_token(jwt_string);
        match result {
            Ok(t) => Ok(t.payload),
            Err(e) => Err(e),
        }
    }

    pub fn parse_token<T: serde::de::DeserializeOwned>(
        &self,
        jwt_string: &str,
    ) -> Result<JWT<T>, LightSpeedError> {
        let result: Result<jsonwebtoken::TokenData<JWT<T>>, jsonwebtoken::errors::Error> =
            jsonwebtoken::decode(jwt_string, &self.secret.as_ref(), &self.validation_default);
        match result {
            Ok(t) => Ok(t.claims),
            Err(e) => match *e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(LightSpeedError::ExpiredTokenError {
                        message: e.to_string(),
                    })
                }
                _ => Err(LightSpeedError::InvalidTokenError {
                    message: e.to_string(),
                }),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use chrono::prelude::Local;
    use super::*;

    #[test]
    fn service_should_be_send_and_sync() {
        call_me_with_send_and_sync(new());
    }

    fn call_me_with_send_and_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn should_create_jwt_string_from_token() {
        let jwt = new();

        let payload = MyTestClaym {
            id: Local::now().timestamp(),
            name: "Red".to_string(),
        };

        let token = super::JWT {
            payload,
            sub: "".to_string(),
            exp: Local::now().timestamp() + 3600,
            iat: Local::now().timestamp(),
        };

        let jwt_string = jwt.generate_from_token(&token).unwrap();
        println!("Jwt string: [{}]", jwt_string);
    }

    #[test]
    fn should_create_jwt_string_from_payload() {
        let jwt = new();

        let payload = MyTestClaym {
            id: Local::now().timestamp(),
            name: "Red".to_string(),
        };

        let jwt_string = jwt.generate_from_payload(&payload).unwrap();
        println!("Jwt string: [{}]", jwt_string);
    }

    #[test]
    fn should_parse_the_token() {
        let jwt = new();

        let payload = MyTestClaym {
            id: Local::now().timestamp(),
            name: "Red".to_string(),
        };

        let jwt_string = jwt.generate_from_payload(&payload).unwrap();
        let parsed: MyTestClaym = jwt.parse_payload(&jwt_string).unwrap();

        assert_eq!(payload.id, parsed.id);
        assert_eq!(payload.name, parsed.name);
    }

    #[test]
    fn should_parse_the_expiration_date() {
        let jwt = new();

        let payload = MyTestClaym {
            id: Local::now().timestamp(),
            name: "Red".to_string(),
        };

        let time_before = Local::now().timestamp();
        let jwt_string = jwt.generate_from_payload(&payload).unwrap();
        let time_after = Local::now().timestamp();

        let token: super::JWT<MyTestClaym> = jwt.parse_token(&jwt_string).unwrap();

        assert_eq!(payload.id, token.payload.id);
        assert_eq!(&payload.name, &token.payload.name);

        let issued_at = token.iat;
        let expiration = token.exp;
        let timeout = (60 as i64) * 60;

        assert!(issued_at >= time_before);
        assert!(issued_at <= time_after);
        assert_eq!(issued_at + timeout, expiration);
    }

    #[test]
    fn should_fail_parsing_tampered_token() {
        let jwt = new();

        let payload = MyTestClaym {
            id: Local::now().timestamp(),
            name: "Red".to_string(),
        };

        let mut jwt_string = jwt.generate_from_payload(&payload).unwrap();
        jwt_string.push_str("1");

        let result: Result<super::JWT<MyTestClaym>, super::LightSpeedError> =
            jwt.parse_token(&jwt_string);
        let mut is_invalid = false;
        match result {
            Ok(r) => println!("Ok: {:?}", r),
            Err(e) => match e {
                super::LightSpeedError::InvalidTokenError { message: mes } => {
                    println!("Error message: {:?}", &mes);
                    is_invalid = true;
                }
                _ => println!("Other kind of error: {:?}", e),
            },
        };
        assert!(is_invalid)
    }

    #[test]
    fn should_fail_parsing_expired_token() {
        let jwt = new();

        let token = super::JWT {
            payload: MyTestClaym {
                id: Local::now().timestamp(),
                name: "Red".to_string(),
            },
            sub: "".to_string(),
            exp: Local::now().timestamp() - 10,
            iat: Local::now().timestamp() - 100,
        };

        let jwt_string = jwt.generate_from_token(&token).unwrap();

        let result: Result<MyTestClaym, super::LightSpeedError> = jwt.parse_payload(&jwt_string);
        let mut is_expired = false;
        match result {
            Ok(r) => println!("Ok: {:?}", r),
            Err(e) => match e {
                super::LightSpeedError::ExpiredTokenError { message: mes } => {
                    println!("Expired: {:?}", &mes);
                    is_expired = true;
                }
                _ => println!("Other kind of error: {:?}", e),
            },
        };
        assert!(is_expired)
    }

    fn new() -> super::JwtService {
        super::JwtService::new(&JwtConfig {
            secret: "mySecret".to_string(),
            signature_algorithm: jsonwebtoken::Algorithm::HS512,
            token_validity_minutes: 60,
        })
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MyTestClaym {
        id: i64,
        name: String,
    }
}
