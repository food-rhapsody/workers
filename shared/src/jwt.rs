use chrono::Duration;
use jwt_compact::{
    alg::{Hs256, Hs256Key},
    CreationError,
    ParseError, prelude::*, ValidationError,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("jwt creation error")]
    CreationError(CreationError),
    #[error("jwt parsed error")]
    ParseError(ParseError),
    #[error("jwt validation error")]
    ValidationError(ValidationError),
}

pub struct Jwt {
    secret: Vec<u8>,
}

impl Jwt {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string().into_bytes(),
        }
    }

    pub fn create_claims<T: Serialize>(&self, data: T, exp: Duration) -> Claims<T> {
        Claims::<T>::new(data).set_duration_and_issuance(&TimeOptions::default(), exp)
    }

    pub fn sign<T: Serialize>(&self, claims: &Claims<T>) -> Result<String, JwtError> {
        let signing_key = Hs256Key::new(&self.secret);
        let header = Header::default();

        match Hs256.token(header, &claims, &signing_key) {
            Ok(token) => Ok(token),
            Err(e) => Err(JwtError::CreationError(e)),
        }
    }

    pub fn verify<T: DeserializeOwned>(&self, token_str: &str) -> Result<Token<T>, JwtError> {
        let verifying_key = Hs256Key::new(&self.secret);
        let parsed_token = UntrustedToken::new(&token_str);
        if let Err(e) = parsed_token {
            return Err(JwtError::ParseError(e));
        }

        let parsed_token = parsed_token.unwrap();
        let token = Hs256.validate_integrity::<T>(&parsed_token, &verifying_key);
        if let Err(e) = token {
            return Err(JwtError::ValidationError(e));
        }

        let token = token.unwrap();
        match token.claims().validate_expiration(&TimeOptions::default()) {
            Ok(_) => Ok(token),
            Err(e) => Err(JwtError::ValidationError(e)),
        }
    }
}

#[cfg(test)]
mod jwt_tests {
    use serde::*;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct CustomClaims {
        #[serde(rename = "sub")]
        subject: String,
    }

    #[test]
    fn should_sign_claims() {
        let jwt = Jwt::new("this_is_secret");
        let custom = CustomClaims {
            subject: "alice".to_owned(),
        };

        let claims = jwt.create_claims(custom, Duration::hours(1));
        let token = jwt.sign(&claims).expect("failed to sign claims.");

        assert_eq!(token.len(), 131);
    }

    #[test]
    fn should_verify_token() {
        let jwt = Jwt::new("this_is_secret");
        let custom = CustomClaims {
            subject: "alice".to_owned(),
        };

        let claims = jwt.create_claims(custom, Duration::hours(1));
        let token = jwt.sign(&claims).unwrap();
        let verified = jwt
            .verify::<CustomClaims>(&token)
            .expect("failed to verify token.");

        assert_eq!(verified.claims().custom.subject, "alice");
    }

    #[test]
    fn should_verify_fail_with_expired_token() {
        let jwt = Jwt::new("this_is_secret");
        let custom = CustomClaims {
            subject: "alice".to_owned(),
        };

        let claims = jwt.create_claims(custom, Duration::hours(-1));
        let token = jwt.sign(&claims).unwrap();
        let verified = jwt.verify::<CustomClaims>(&token);

        assert!(verified.is_err());
        assert!(matches!(
            verified.unwrap_err(),
            JwtError::ValidationError(ValidationError::Expired)
        ));
    }
}
