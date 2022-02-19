use serde::{Deserialize, Serialize};
use worker::{Fetch, Headers, Method, Request, RequestInit, Response, RouteContext};

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::res::response_with_cache;

const KAKAO_MAP_API: &str = "https://dapi.kakao.com/v2/local/search/keyword.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceDocument {
    pub id: String,
    pub place_name: String,
    pub category_name: String,
    pub category_group_code: String,
    pub category_group_name: String,
    pub phone: String,
    pub address_name: String,
    pub road_address_name: String,
    pub x: String,
    pub y: String,
    pub place_url: String,
    pub distance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSearchMetadata {
    pub total_count: i32,
    pub pageable_count: i32,
    pub is_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSearchResult {
    pub meta: PlaceSearchMetadata,
    pub documents: Vec<PlaceDocument>,
}

pub async fn search_place(req: Request, ctx: RouteContext<()>) -> ApiResult<Response> {
    let url = req.url()?;
    let query = url.query();
    let kakao_api_key = ctx.secret("KAKAO_API_KEY").unwrap().to_string();
    let cache = ctx.kv("PLACE").unwrap();

    if let Some(x) = query {
        if let Ok(result) = cache.get(x).json::<PlaceSearchResult>().await {
            if let Some(x) = result {
                return Ok(Response::from_json(&x)?);
            }
        }
    } else {
        return Err(ApiError::BadRequest("invalid request fields".to_string()));
    }

    let query = query.unwrap();

    match search_place_with_kakao_api(&kakao_api_key, query).await {
        Ok(mut res) => {
            match res.status_code() {
                200 => {
                    let result = res.json::<PlaceSearchResult>().await?;
                    cache.put(&query, &result)?.expiration_ttl(60 * 60 * 24 * 30).execute().await?;

                    Ok(response_with_cache(&result, 60 * 60 * 24)?)
                },
                _ => Ok(res)
            }
        },
        Err(e) => Err(e),
    }
}

async fn search_place_with_kakao_api(kakao_api_key: &str, query: &str) -> ApiResult<Response> {
    let mut headers = Headers::new();
    headers.append("Authorization", &format!("KakaoAK {}", kakao_api_key)).unwrap();

    let mut req_init = RequestInit::new();
    req_init.with_method(Method::Post).with_headers(headers);

    let req_url = format!("{}?{}", KAKAO_MAP_API, query);
    let req = Request::new_with_init(&req_url, &req_init)?;

    let res = Fetch::Request(req).send().await?;

    Ok(res)
}
