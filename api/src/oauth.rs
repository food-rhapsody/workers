use worker::{
    Fetch, Headers, Method, Request, RequestInit, Response as WorkerResponse,
    Result as WorkerResult,
};

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

    // TODO: check email
    pub async fn verify_token(&self, token: &str) -> ApiResult<()> {
        match self.request_verify_token(token).await {
            Ok(response) => match response.status_code() {
                200 => Ok(()),
                _ => Err(ApiError::InvalidOAuthToken),
            },
            Err(e) => Err(ApiError::WorkerError { source: e }),
        }
    }

    async fn request_verify_token(&self, token: &str) -> WorkerResult<WorkerResponse> {
        match self {
            OAuthProvider::Kakao => {
                let auth_header = format!("Bearer {}", token);

                let mut req_headers = Headers::new();
                req_headers.append("Authorization", &auth_header)?;

                let mut req_init = RequestInit::new();
                req_init.with_method(Method::Get).with_headers(req_headers);

                let req = Request::new_with_init(KAKAO_USER_URL, &req_init)?;

                Fetch::Request(req).send().await
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
