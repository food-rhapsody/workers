use serde::Serialize;
use worker::{Headers, Response, Result as WorkerResult};

pub fn response<B: Serialize>(value: &B) -> WorkerResult<Response> {
    let res = Response::from_json(value)?;
    let mut headers = Headers::new();
    headers.set("content-type", "application/json; charset=utf-8")?;

    Ok(res.with_headers(headers))
}

pub fn response_with_cache<B: Serialize>(value: &B, cache: i32) -> WorkerResult<Response> {
    let res = Response::from_json(value)?;
    let mut headers = Headers::new();
    headers.append("cache-control", &format!("public, max-age={}", cache))?;
    headers.set("content-type", "application/json; charset=utf-8")?;

    Ok(res.with_headers(headers))
}
