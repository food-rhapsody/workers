use serde_json::json;
use worker::*;
use shared::utils::set_panic_hook;

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    let router = Router::new();
    set_panic_hook();

    router
        .get("/health", |_, _| Response::ok("OK"))
        .get("/version", |_, ctx| {
            let version = ctx.var("VERSION")?.to_string();
            Response::from_json(&json!({ "version": version }))
        })
        .run(req, env)
        .await
}
