use serde_json::json;
use worker::{Request, Response, Result as WorkerResult, RouteContext};

pub fn health_route<D>(_: Request, _: RouteContext<D>) -> WorkerResult<Response> {
    Response::ok("OK")
}

pub fn version_route<D>(_: Request, ctx: RouteContext<D>) -> WorkerResult<Response> {
    let version = ctx.var("VERSION")?.to_string();

    Response::from_json(&json!({ "version": version }))
}
