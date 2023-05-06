use serde::Serialize;
use serde_json::json;
use vercel_runtime::{Body, Error, Response, StatusCode};

pub fn make_error_response<T: Into<String>>(
    status: StatusCode,
    message: T,
) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(
            json!({
                "error": status.to_string(),
                "code": status.as_u16(),
                "message": message.into(),
            })
            .to_string()
            .into(),
        )?)
}

pub fn make_error_response_detail<T: Into<String>, D: Serialize>(
    status: StatusCode,
    message: T,
    detail: D,
) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Content-Type", "application/json")
        .body(
            json!({
                "error": status.to_string(),
                "code": status.as_u16(),
                "message": message.into(),
                "detail": detail
            })
            .to_string()
            .into(),
        )?)
}
