use serde::{Deserialize, Serialize};
use worker::{Fetch, Headers, Method, Request, RequestInit};

use crate::api_error::ApiError;
use crate::api_result::ApiResult;

const KAKAO_PROVIDER_NAME: &str = "kakao";
const KAKAO_USER_URL: &str = "https://kapi.kakao.com/v2/user/me";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KakaoUser {
    pub kakao_account: KakaoAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KakaoAccount {
    pub email: Option<String>,
}

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

    pub async fn verify_token(&self, token: &str, email: &str) -> ApiResult<()> {
        let oauth_email = self.fetch_email_with_token(token).await?;

        if !email.eq(&oauth_email) {
            return Err(ApiError::InvalidOAuthToken);
        }

        Ok(())
    }

    async fn fetch_email_with_token(&self, token: &str) -> ApiResult<String> {
        match self {
            OAuthProvider::Kakao => {
                let auth_header = format!("Bearer {}", token);

                let mut req_headers = Headers::new();
                req_headers.append("Authorization", &auth_header)?;

                let mut req_init = RequestInit::new();
                req_init.with_method(Method::Get).with_headers(req_headers);

                let req = Request::new_with_init(KAKAO_USER_URL, &req_init)?;

                match Fetch::Request(req).send().await {
                    Ok(mut res) => match res.status_code() {
                        200 => {
                            let kakao_user = res.json::<KakaoUser>().await?;
                            let kakao_account = kakao_user.kakao_account;

                            Ok(kakao_account.email.unwrap_or("NO_EMAIL".to_owned()))
                        }
                        _ => Err(ApiError::InvalidOAuthToken),
                    },
                    Err(e) => Err(ApiError::WorkerError { source: e }),
                }
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
