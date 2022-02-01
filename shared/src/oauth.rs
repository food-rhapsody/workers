use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder, StatusCode};

use crate::api_error::ApiError;
use crate::api_result::ApiResult;

const KAKAO_PROVIDER_NAME: &str = "kakao";
const KAKAO_USER_URL: &str = "https://kapi.kakao.com/v2/user/me";

#[derive(Debug, PartialEq)]
pub enum OAuthProvider {
    Kakao,
}

impl OAuthProvider {
    pub fn from_str(name: &str) -> ApiResult<Self> {
        match name {
            KAKAO_PROVIDER_NAME => Ok(OAuthProvider::Kakao),
            _ => Err(ApiError::InvalidOAuthProvider),
        }
    }

    pub async fn verify_token(&self, token: &str) -> ApiResult<()> {
        let request = self.build_verifying_request(token);

        match request.send().await {
            Ok(response) => match response.status() {
                StatusCode::OK => Ok(()),
                _ => Err(ApiError::InvalidOAuthToken),
            },
            Err(_) => Err(ApiError::InvalidOAuthToken),
        }
    }

    fn build_verifying_request(&self, token: &str) -> RequestBuilder {
        match self {
            OAuthProvider::Kakao => {
                let auth_header = format!("Bearer {}", token);
                let auth_header_value = HeaderValue::from_str(&auth_header).unwrap();

                let mut headers = HeaderMap::new();
                headers.insert("Authorization", auth_header_value);

                Client::builder()
                    .default_headers(headers)
                    .build()
                    .unwrap()
                    .get(KAKAO_USER_URL)
            }
        }
    }
}

#[cfg(test)]
mod oauth_provider_tests {
    use super::*;

    #[test]
    fn should_parse_kakao_provider_name() {
        let name = "kakao";
        let provider = OAuthProvider::from_str(name).unwrap();

        assert_eq!(provider, OAuthProvider::Kakao)
    }

    #[test]
    fn should_err_when_provider_name_is_incorrect() {
        let try1 = OAuthProvider::from_str("KAKAO").unwrap_err();
        let try2 = OAuthProvider::from_str("undefined provider").unwrap_err();

        assert!(matches!(try1, ApiError::InvalidOAuthProvider));
        assert!(matches!(try2, ApiError::InvalidOAuthProvider));
    }
}
