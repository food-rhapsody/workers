use worker::*;
use shared::users::{User, UsersStore};

pub async fn authorize(
    req: Request,
    ctx: RouteContext<()>
) -> Result<Option<User>> {
    let users_store = UsersStore::new(ctx);
    let auth_header = match req.headers().get("Authorization")? {
        Some(value) => value,
        None => String::from(""),
    };
    let token = get_auth_token_from_header(&auth_header)?;

    users_store.get_user_by_token(&token).await
}

pub fn get_auth_token_from_header(header: &str) -> Result<String> {
    let chunks = header.split(" ").collect::<Vec<&str>>();
    let prefix = match chunks.get(0) {
        Some(value) => value,
        None => ""
    };
    let token = match chunks.get(1) {
        Some(value) => value,
        None => ""
    };

    match &prefix[..] {
        "Bearer" => Ok(token.to_string()),
        _ => Err(Error::from("Unauthorized"))
    }
}

#[cfg(test)]
mod authorize_tests {
    use super::*;

    #[test]
    fn get_auth_token_from_header_토큰을_성공적으로_불러온다() {
        let header = "Bearer my_token";
        let result = get_auth_token_from_header(header).unwrap();

        assert_eq!(result, "my_token");
    }

    #[test]
    fn get_auth_token_from_header_prefix가_일치하지_않아_오류가_발생한다() {
        let header = "Token my_token";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn get_auth_token_from_header_빈문자열이라_오류가_발생한다() {
        let header = "";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn get_auth_token_from_header_잘못된_값이라_오류가_발생한다() {
        let header = "잘못된 인증 헤더";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }
}

