use worker::Request;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::jwt::Jwt;
use crate::users::{User, UserClaims, Users};

pub async fn authorize(users: &Users, req: Request) -> ApiResult<User> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or("".to_owned());
    let token_str = get_auth_token_from_header(&auth_header)?;

    let secret = users.get_jwt_secret()?;
    let jwt = Jwt::new(&secret);
    let token = jwt.verify::<UserClaims>(&token_str);

    match token {
        Ok(x) => match users.get_user_by_id(&x.claims().custom.subject).await {
            Ok(user) => Ok(user),
            Err(_) => Err(ApiError::Unauthorized),
        },
        Err(_) => Err(ApiError::Unauthorized),
    }
}

pub async fn optional_authorize(users: &Users, req: Request) -> ApiResult<Option<User>> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or("".to_owned());
    let token_str = get_auth_token_from_header(&auth_header);
    if let Err(_) = token_str {
        return Ok(None);
    }

    let secret = users.get_jwt_secret()?;
    let jwt = Jwt::new(&secret);
    let token = jwt.verify::<UserClaims>(&token_str.unwrap());

    match token {
        Ok(x) => {
            let user = users.get_user_by_id(&x.claims().custom.subject).await?;

            Ok(Some(user))
        }
        Err(_) => Ok(None),
    }
}

fn get_auth_token_from_header(header: &str) -> ApiResult<String> {
    let chunks = header.split(" ").collect::<Vec<&str>>();
    let prefix = match chunks.get(0) {
        Some(value) => value,
        None => "",
    };
    let token = match chunks.get(1) {
        Some(value) => value,
        None => "",
    };

    match &prefix[..] {
        "Bearer" => Ok(token.to_string()),
        _ => Err(ApiError::Unauthorized),
    }
}

#[cfg(test)]
mod get_auth_token_from_header_tests {
    use super::*;

    #[test]
    fn should_get_auth_token() {
        let header = "Bearer my_token";
        let result = get_auth_token_from_header(header).unwrap();

        assert_eq!(result, "my_token");
    }

    #[test]
    fn should_err_when_prefix_is_not_matches() {
        let header = "Token my_token";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert!(matches!(err, ApiError::Unauthorized));
    }

    #[test]
    fn should_err_when_auth_header_is_empty_string() {
        let header = "";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert!(matches!(err, ApiError::Unauthorized));
    }

    #[test]
    fn should_err_when_auth_header_is_incorrect() {
        let header = "Wrong authorization header";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert!(matches!(err, ApiError::Unauthorized));
    }
}
