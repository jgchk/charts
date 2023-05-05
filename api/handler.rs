use charts::{
    api::{make_error_response, make_error_response_detail},
    charts::{create_chart, Chart},
};
use serde_valid::Validate;
use vercel_runtime::{run, Body, Error, Request, RequestExt, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let chart_request = req.payload::<Chart>();
    let chart_request = match chart_request {
        Ok(Some(chart_request)) => chart_request,
        Ok(None) => {
            return make_error_response(StatusCode::BAD_REQUEST, "Request body is missing or empty")
        }
        Err(err) => {
            return make_error_response_detail(
                StatusCode::BAD_REQUEST,
                "Failed to parse request body",
                err.to_string(),
            )
        }
    };

    let is_valid = chart_request.validate();
    if let Err(errors) = is_valid {
        return make_error_response_detail(StatusCode::BAD_REQUEST, "Invalid request body", errors);
    };

    let chart = create_chart(chart_request);

    match chart {
        Ok(chart) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/png")
            .body(chart.into())?),
        Err(err) => make_error_response_detail(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error creating chart",
            err.to_string(),
        ),
    }
}