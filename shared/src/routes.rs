use worker::*;
use serde_json::json;

pub fn health_route<D>(_: Request, _: RouteContext<D>) -> Result<Response> {
    Response::ok("OK")
}

pub fn version_route<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let version = ctx.var("VERSION")?.to_string();

    Response::from_json(&json!({ "version": version }))
}
