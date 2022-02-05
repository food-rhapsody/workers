use worker::*;

use shared::routes::{health_route, version_route};
use shared::utils::wasm::set_panic_hook;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    let request_to_users = |_req: Request, ctx: RouteContext<()>| async move {
        let namespace = ctx.durable_object("USERS")?;
        let stub = namespace.id_from_name("USERS")?.get_stub()?;

        stub.fetch_with_request(_req).await
    };

    router
        .get("/health", health_route)
        .get("/version", version_route)
        .post_async("/users", request_to_users)
        .get_async("/me", request_to_users)
        .post_async("/me/token", request_to_users)
        .run(req, env)
        .await
}
