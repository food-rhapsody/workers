use async_trait::async_trait;
use serde::Deserialize;
use worker::{Error, Storage};

use crate::api_result::ApiResult;
use crate::ApiError;

#[async_trait(? Send)]
pub trait DurableStorageFind {
    async fn find<T: for<'a> Deserialize<'a>>(&self, key: &str) -> ApiResult<Option<T>>;
}

#[async_trait(? Send)]
impl DurableStorageFind for Storage {
    async fn find<T: for<'a> Deserialize<'a>>(&self, key: &str) -> ApiResult<Option<T>> {
        let data = self.get::<T>(key).await;

        match data {
            Ok(x) => Ok(Some(x)),
            Err(e) => match e {
                Error::JsError(msg) => match &msg[..] {
                    "No such value in storage." => Ok(None),
                    _ => Err(ApiError::ServerError("storage error".to_string())),
                },
                _ => Err(ApiError::WorkerError { source: e }),
            },
        }
    }
}
