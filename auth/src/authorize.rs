use worker::*;

use shared::users::User;
use shared::users_store::UsersStore;

pub async fn authorize(req: Request, ctx: RouteContext<()>) -> Result<Option<User>> {
    let users_store = UsersStore::new(ctx);
    let auth_header = match req.headers().get("Authorization")? {
        Some(value) => value,
        None => String::from(""),
    };
    let token = get_auth_token_from_header(&auth_header)?;

    users_store.get_user_by_token(&token).await
}

fn get_auth_token_from_header(header: &str) -> Result<String> {
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
        _ => Err(Error::from("Unauthorized")),
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

        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn should_err_when_auth_header_is_empty_string() {
        let header = "";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn should_err_when_auth_header_is_incorrect() {
        let header = "Wrong authorization header";
        let err = get_auth_token_from_header(header).unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }
}
