use crate::api_error::ApiError;
use crate::api_result::ApiResult;

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
