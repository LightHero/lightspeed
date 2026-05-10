use crate::config::JwtConfig;
use crate::error::LsError;
use crate::utils::current_epoch_seconds;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

/// JWT signing/verification service.
#[derive(Clone)]
pub struct LsJwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_validity_seconds: i64,
    header_default: jsonwebtoken::Header,
    validation_default: jsonwebtoken::Validation,
}

impl LsJwtService {
    pub fn new(jwt_config: &JwtConfig) -> Result<LsJwtService, LsError> {
        let alg = jwt_config.signature_algorithm;

        // Only HMAC-family algorithms produce keys via `from_secret`.
        if !matches!(alg, Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512) {
            return Err(LsError::ConfigurationError {
                message: format!(
                    "JWT signature_algorithm [{alg:?}] is not HMAC-family. \
                     LsJwtService only supports HS256/HS384/HS512 (symmetric \
                     secrets). For RS*/ES*/PS*/EdDSA, build EncodingKey and \
                     DecodingKey directly from PEM/DER and use jsonwebtoken \
                     primitives instead of this service."
                ),
            });
        }

        let secret_bytes = jwt_config.secret.expose_secret().as_bytes();
        if secret_bytes.is_empty() {
            return Err(LsError::ConfigurationError { message: "JWT secret key cannot be empty".to_owned() });
        }

        // Once these are built the original `SecretString` can be dropped.
        let encoding_key = EncodingKey::from_secret(secret_bytes);
        let decoding_key = DecodingKey::from_secret(secret_bytes);

        let mut validation_default = jsonwebtoken::Validation::new(alg);
        validation_default.leeway = 0;

        Ok(LsJwtService {
            encoding_key,
            decoding_key,
            token_validity_seconds: i64::from(jwt_config.token_validity_minutes) * 60,
            header_default: jsonwebtoken::Header { alg, ..jsonwebtoken::Header::default() },
            validation_default,
        })
    }

    pub fn generate_from_payload<'a, T: serde::ser::Serialize>(
        &self,
        payload: &'a T,
    ) -> Result<(JWT<&'a T>, String), LsError> {
        let issued_at = current_epoch_seconds();
        let token = JWT { payload, sub: "".to_string(), exp: issued_at + self.token_validity_seconds, iat: issued_at };
        self.generate_from_token(&token).map(|jwt| (token, jwt))
    }

    pub fn generate_from_token<T: serde::ser::Serialize>(&self, token: &JWT<T>) -> Result<String, LsError> {
        let result = jsonwebtoken::encode(&self.header_default, &token, &self.encoding_key);
        match result {
            Ok(t) => Ok(t),
            Err(e) => {
                //let err = e.to_string();
                Err(LsError::GenerateTokenError { message: e.to_string() })
            }
        }
    }

    pub fn parse_payload<T: serde::de::DeserializeOwned>(&self, jwt_string: &str) -> Result<T, LsError> {
        let result = self.parse_token(jwt_string);
        match result {
            Ok(t) => Ok(t.payload),
            Err(e) => Err(e),
        }
    }

    pub fn parse_token<T: serde::de::DeserializeOwned>(&self, jwt_string: &str) -> Result<JWT<T>, LsError> {
        let result: Result<jsonwebtoken::TokenData<JWT<T>>, jsonwebtoken::errors::Error> =
            jsonwebtoken::decode(jwt_string, &self.decoding_key, &self.validation_default);
        match result {
            Ok(t) => Ok(t.claims),
            Err(e) => match *e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(LsError::ExpiredTokenError { message: e.to_string() })
                }
                _ => Err(LsError::InvalidTokenError { message: e.to_string() }),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use chrono::prelude::Local;

    #[test]
    fn service_should_be_send_and_sync() {
        call_me_with_send_and_sync(new());
    }

    fn call_me_with_send_and_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn should_create_jwt_string_from_token() {
        let jwt = new();

        let payload = MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() };

        let token = super::JWT {
            payload,
            sub: "".to_string(),
            exp: Local::now().timestamp() + 3600,
            iat: Local::now().timestamp(),
        };

        let jwt_string = jwt.generate_from_token(&token).unwrap();
        println!("Jwt string: [{jwt_string}]");
    }

    #[test]
    fn should_create_jwt_string_from_payload() {
        let jwt = new();

        let payload = MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() };

        let (jwt, jwt_string) = jwt.generate_from_payload(&payload).unwrap();

        assert_eq!(payload.id, jwt.payload.id);

        println!("Jwt string: [{jwt_string}]");
    }

    #[test]
    fn should_parse_the_token() {
        let jwt = new();

        let payload = MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() };

        let jwt_string = jwt.generate_from_payload(&payload).unwrap().1;
        let parsed: MyTestClaym = jwt.parse_payload(&jwt_string).unwrap();

        assert_eq!(payload.id, parsed.id);
        assert_eq!(payload.name, parsed.name);
    }

    #[test]
    fn should_parse_the_expiration_date() {
        let jwt = new();

        let payload = MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() };

        let time_before = Local::now().timestamp();
        let jwt_string = jwt.generate_from_payload(&payload).unwrap().1;
        let time_after = Local::now().timestamp();

        let token: super::JWT<MyTestClaym> = jwt.parse_token(&jwt_string).unwrap();

        assert_eq!(payload.id, token.payload.id);
        assert_eq!(&payload.name, &token.payload.name);

        let issued_at = token.iat;
        let expiration = token.exp;
        let timeout = 60_i64 * 60;

        assert!(issued_at >= time_before);
        assert!(issued_at <= time_after);
        assert_eq!(issued_at + timeout, expiration);
    }

    #[test]
    fn should_fail_parsing_tampered_token() {
        let jwt = new();

        let payload = MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() };

        let mut jwt_string = jwt.generate_from_payload(&payload).unwrap().1;
        jwt_string.push('1');

        let result: Result<super::JWT<MyTestClaym>, super::LsError> = jwt.parse_token(&jwt_string);
        let mut is_invalid = false;
        match result {
            Ok(r) => println!("Ok: {r:?}"),
            Err(e) => match e {
                super::LsError::InvalidTokenError { message: mes } => {
                    println!("Error message: {:?}", &mes);
                    is_invalid = true;
                }
                _ => println!("Other kind of error: {e:?}"),
            },
        };
        assert!(is_invalid)
    }

    #[test]
    fn should_fail_parsing_expired_token() {
        let jwt = new();

        let token = super::JWT {
            payload: MyTestClaym { id: Local::now().timestamp(), name: "Red".to_string() },
            sub: "".to_string(),
            exp: Local::now().timestamp() - 10,
            iat: Local::now().timestamp() - 100,
        };

        let jwt_string = jwt.generate_from_token(&token).unwrap();

        let result: Result<MyTestClaym, super::LsError> = jwt.parse_payload(&jwt_string);
        let mut is_expired = false;
        match result {
            Ok(r) => println!("Ok: {r:?}"),
            Err(e) => match e {
                super::LsError::ExpiredTokenError { message: mes } => {
                    println!("Expired: {:?}", &mes);
                    is_expired = true;
                }
                _ => println!("Other kind of error: {e:?}"),
            },
        };
        assert!(is_expired)
    }

    #[test]
    fn should_not_build_if_secret_key_empty() {
        assert!(
            super::LsJwtService::new(&JwtConfig {
                secret: "".into(),
                signature_algorithm: jsonwebtoken::Algorithm::HS512,
                token_validity_minutes: 60,
            })
            .is_err()
        );
    }

    #[test]
    fn should_reject_non_hmac_algorithm() {
        // `EncodingKey::from_secret` is only valid for HS256/HS384/HS512.
        // Any other algorithm must be rejected at construction so a
        // misconfiguration can't ship.
        for alg in [
            jsonwebtoken::Algorithm::RS256,
            jsonwebtoken::Algorithm::RS384,
            jsonwebtoken::Algorithm::RS512,
            jsonwebtoken::Algorithm::ES256,
            jsonwebtoken::Algorithm::ES384,
            jsonwebtoken::Algorithm::PS256,
            jsonwebtoken::Algorithm::EdDSA,
        ] {
            let result = super::LsJwtService::new(&JwtConfig {
                secret: "mySecret".into(),
                signature_algorithm: alg,
                token_validity_minutes: 60,
            });
            match result {
                Err(super::LsError::ConfigurationError { .. }) => {}
                Err(other) => panic!("expected ConfigurationError for {alg:?}, got {other:?}"),
                Ok(_) => panic!("expected ConfigurationError for {alg:?}, got Ok"),
            }
        }
    }

    fn new() -> super::LsJwtService {
        super::LsJwtService::new(&JwtConfig {
            secret: "mySecret".into(),
            signature_algorithm: jsonwebtoken::Algorithm::HS512,
            token_validity_minutes: 60,
        })
        .unwrap()
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct MyTestClaym {
        id: i64,
        name: String,
    }
}
