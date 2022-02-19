use async_trait::async_trait;
use serde::de::DeserializeOwned;
use worker::Request;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;

#[async_trait(?Send)]
pub trait ParseReqJson {
    async fn parse_json<B: DeserializeOwned>(&mut self) -> ApiResult<B>;
}

#[async_trait(?Send)]
impl ParseReqJson for Request {
    async fn parse_json<B: DeserializeOwned>(&mut self) -> ApiResult<B> {
        let body = self.json::<B>().await;

        match body {
            Ok(x) => Ok(x),
            Err(_) => Err(ApiError::BadRequest("invalid request fields".to_string())),
        }
    }
}
