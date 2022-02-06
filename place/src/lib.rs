use worker::*;

use shared::api_error::ApiError;
use shared::place::PlaceSearchResult;
use shared::routes::{health_route, version_route};
use shared::utils::wasm::set_panic_hook;

const KAKAO_MAP_API: &str = "https://dapi.kakao.com/v2/local/search/keyword.json";

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    router
        .get("/health", health_route)
        .get("/version", version_route)
        .post_async("/search", |_req, ctx| async move {
            let url = _req.url().unwrap();
            let kakao_api_key = ctx.secret("KAKAO_API_KEY").unwrap().to_string();
            let kv = ctx.kv("PLACE").unwrap();

            let query = url.query();
            if let Some(x) = query {
                if let Ok(result) = kv.get(x).json::<PlaceSearchResult>().await {
                    if let Some(x) = result {
                        return Response::from_json(&x);
                    }
                }
            } else {
                return Ok(ApiError::BadRequest("invalid request fields".to_string()).to_response());
            }

            let query = query.unwrap();

            let mut headers = Headers::new();
            headers.append("Authorization", &format!("KakaoAK {}", kakao_api_key))?;

            let mut req_init = RequestInit::new();
            req_init.with_method(Method::Post).with_headers(headers);

            let req_url = format!("{}?{}", KAKAO_MAP_API, query);
            let req = Request::new_with_init(&req_url, &req_init)?;

            match Fetch::Request(req).send().await {
                Ok(mut response) => {
                    match response.status_code() {
                        200 => {
                            let result = response.json::<PlaceSearchResult>().await?;
                            kv.put(&query, &result)?.expiration_ttl(60 * 60 * 24 * 30).execute().await?;

                            let res = Response::from_json(&result)?;
                            let mut res_headers = Headers::new();
                            res_headers.append("cache-control", &format!("public, max-age={}", 60 * 60 * 24))?;
                            res_headers.set("content-type", "application/json; charset=utf-8")?;

                            Ok(res.with_headers(res_headers))
                        },
                        _ => Ok(response)
                    }
                },
                Err(e) => Err(e)
            }
        })
        .run(req, env)
        .await
}
