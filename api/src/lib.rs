use worker::*;

use crate::api_error::ApiError;
use crate::auth::build_admin_auth_req;
use crate::place::search_place;
use crate::routes::{health_route, version_route};
use crate::utils::wasm::set_panic_hook;

mod api_error;
mod api_result;
mod auth;
mod challenges;
mod jwt;
mod oauth;
mod place;
mod req;
mod routes;
mod users;
mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    let request_to_users = |_req: Request, ctx: RouteContext<()>| async move {
        let namespace = ctx.durable_object("USERS")?;
        let stub = namespace.id_from_name("USERS")?.get_stub()?;

        stub.fetch_with_request(_req).await
    };

    let request_to_challenges = |_req: Request, ctx: RouteContext<()>| async move {
        let namespace = ctx.durable_object("CHALLENGES")?;
        let stub = namespace.id_from_name("CHALLENGES")?.get_stub()?;

        stub.fetch_with_request(_req).await
    };

    let request_to_challenges_for_admin = |_req: Request, ctx: RouteContext<()>| async move {
        let users_namespace = ctx.durable_object("USERS")?;
        let users_stub = users_namespace.id_from_name("USERS")?.get_stub()?;

        let challenges_namespace = ctx.durable_object("CHALLENGES")?;
        let challenges_stub = challenges_namespace
            .id_from_name("CHALLENGES")?
            .get_stub()?;

        let admin_auth_req = build_admin_auth_req(&_req)?;

        match users_stub.fetch_with_request(admin_auth_req).await {
            Ok(res) => match res.status_code() {
                200 => challenges_stub.fetch_with_request(_req).await,
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
        .run(req, env)
        .await
}
