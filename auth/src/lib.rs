use worker::*;

use shared::routes::{health_route, version_route};
use shared::utils::wasm::set_panic_hook;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    router
        .get("/health", health_route)
        .get("/version", version_route)
        .run(req, env)
        .await
}
