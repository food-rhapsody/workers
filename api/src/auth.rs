use std::str::FromStr;

use worker::{Headers, Method, Request, RequestInit, Result as WorkerResult, Url};

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::users::{User, UserClaims, Users};

pub async fn authorize_access_token(users: &Users, req: Request) -> ApiResult<User> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or("".to_owned());
    let token_str = get_auth_token_from_header(&auth_header)?;

    let jwt = users.get_jwt_for_access_token()?;
    let token = jwt.verify::<UserClaims>(&token_str);
    if let Err(_) = token {
        return Err(ApiError::Unauthorized);
    }

    let user = users
        .get_user_by_id(&token.unwrap().claims().custom.subject)
        .await;
    if let Err(_) = user {
        return Err(ApiError::Unauthorized);
    }

    let user = user.unwrap();
    let access_token = user.access_token.clone().unwrap_or("NOOP".to_string());

    if access_token.eq(&token_str) {
        Ok(user)
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub async fn authorize_refresh_token(users: &Users, req: Request) -> ApiResult<User> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or("".to_owned());
    let token_str = get_auth_token_from_header(&auth_header)?;

    let jwt = users.get_jwt_for_refresh_token()?;
    let token = jwt.verify::<UserClaims>(&token_str);
    if let Err(_) = token {
        return Err(ApiError::Unauthorized);
    }

    let user = users
        .get_user_by_refresh_id(&token.unwrap().claims().custom.subject)
        .await;
    if let Err(_) = user {
        return Err(ApiError::Unauthorized);
    }

    let user = user.unwrap();
    let refresh_token = user.refresh_token.clone().unwrap_or("NOOP".to_string());

    if refresh_token.eq(&token_str) {
        Ok(user)
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub fn build_admin_auth_req(req: &Request) -> WorkerResult<Request> {
    let auth = req.headers().get("Authorization")?.unwrap_or("".to_owned());
    let mut headers = Headers::new();
    headers.append("Authorization", &auth)?;

    let mut init = RequestInit::new();
    init.with_headers(headers).with_method(Method::Get);

    let mut url = Url::from_str(req.url()?.as_str())?;
    url.set_path("/me/admin");

    let admin_auth_req = Request::new_with_init(url.as_str(), &init)?;

    Ok(admin_auth_req)
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
