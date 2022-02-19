use worker::*;

use crate::api_error::ApiError;
use crate::auth::{build_admin_auth_req, build_auth_req};
use crate::place::search_place;
use crate::routes::{health_route, version_route};
use crate::users::UserInfoDto;
use crate::utils::wasm::set_panic_hook;
use crate::wasm_bindgen::JsValue;

mod api_error;
mod api_result;
mod auth;
mod challenges;
mod durable;
mod foodnotes;
mod jwt;
mod oauth;
mod place;
mod req;
mod res;
mod routes;
mod users;
mod utils;

fn get_users_stub(ctx: &RouteContext<()>) -> Result<Stub> {
    ctx.durable_object("USERS")?
        .id_from_name("USERS")?
        .get_stub()
}

fn get_challenges_stub(ctx: &RouteContext<()>) -> Result<Stub> {
    ctx.durable_object("CHALLENGES")?
        .id_from_name("CHALLENGES")?
        .get_stub()
}

fn get_foodnotes_stub(ctx: &RouteContext<()>) -> Result<Stub> {
    ctx.durable_object("FOODNOTES")?
        .id_from_name("FOODNOTES")?
        .get_stub()
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    let request_to_users = |_req: Request, ctx: RouteContext<()>| async move {
        get_users_stub(&ctx)?.fetch_with_request(_req).await
    };

    let request_to_challenges = |_req: Request, ctx: RouteContext<()>| async move {
        get_challenges_stub(&ctx)?.fetch_with_request(_req).await
    };

    let request_to_challenges_for_admin = |_req: Request, ctx: RouteContext<()>| async move {
        let users_stub = get_users_stub(&ctx)?;
        let challenges_stub = get_challenges_stub(&ctx)?;
        let admin_auth_req = build_admin_auth_req(&_req)?;

        match users_stub.fetch_with_request(admin_auth_req).await {
            Ok(res) => match res.status_code() {
                200 => challenges_stub.fetch_with_request(_req).await,
                _ => Ok(ApiError::Unauthorized.to_response()),
            },
            Err(e) => Err(e),
        }
    };

    let request_to_foodnotes = |mut _req: Request, ctx: RouteContext<()>| async move {
        let users_stub = get_users_stub(&ctx)?;
        let foodnotes_stub = get_foodnotes_stub(&ctx)?;

        let auth_req = build_auth_req(&_req)?;

        match users_stub.fetch_with_request(auth_req).await {
            Ok(mut res) => match res.status_code() {
                200 => {
                    let user_id = res.json::<UserInfoDto>().await?.id;
                    let mut req_headers = Headers::new();
                    req_headers.append("X-Foodrhapsody-User", &user_id)?;

                    let mut req_init = RequestInit::new();
                    req_init
                        .with_method(_req.method())
                        .with_headers(req_headers);

                    let body = _req.text().await?;
                    if !body.is_empty() {
                        req_init.with_body(Some(JsValue::from(body)));
                    }

                    let req = Request::new_with_init(_req.url()?.as_str(), &req_init)?;

                    foodnotes_stub.fetch_with_request(req).await
                }
                _ => Ok(ApiError::Unauthorized.to_response()),
            },
            Err(e) => Err(e),
        }
    };

    router
        .get("/health", health_route)
        .get("/version", version_route)
        .post_async("/users", request_to_users)
        .get_async("/me", request_to_users)
        .get_async("/me/admin", request_to_users)
        .post_async("/me/token", request_to_users)
        .post_async("/place/search", |_req, ctx| async move {
            match search_place(_req, ctx).await {
                Ok(res) => Ok(res),
                Err(e) => Ok(e.to_response()),
            }
        })
        .get_async("/challenges", request_to_challenges)
        .post_async("/challenges", request_to_challenges_for_admin)
        .put_async("/challenges", request_to_challenges_for_admin)
        .get_async("/foodnotes", request_to_foodnotes)
        .post_async("/foodnotes", request_to_foodnotes)
        .run(req, env)
        .await
}
