use charts::{
    api::{make_error_response, make_error_response_detail},
    charts::{create_chart, Chart},
};
use http::Method;
use serde_valid::Validate;
use vercel_runtime::{run, Body, Error, Request, RequestExt, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    println!("Received request");

    if req.method() == Method::OPTIONS {
        println!("Handling OPTIONS request");
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .body(Body::Empty)?);
    }

    if req.method() != Method::POST {
        println!("Invalid method: {:?}", req.method());
        return make_error_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "Only POST requests are allowed",
        );
    }

    let chart_request = req.payload::<Chart>();
    let chart_request = match chart_request {
        Ok(Some(chart_request)) => chart_request,
        Ok(None) => {
            println!("Missing or empty request body");
            return make_error_response(
                StatusCode::BAD_REQUEST,
                "Request body is missing or empty",
            );
        }
        Err(err) => {
            println!("Failed to parse request body: {:?}", err);
            return make_error_response_detail(
                StatusCode::BAD_REQUEST,
                "Failed to parse request body",
                err.to_string(),
            );
        }
    };

    println!("Parsed request body");

    let is_valid = chart_request.validate();
    if let Err(errors) = is_valid {
        println!("Invalid request body: {:?}", errors);
        return make_error_response_detail(StatusCode::BAD_REQUEST, "Invalid request body", errors);
    };

    let chart = match create_chart(chart_request).await {
        Ok(chart) => chart,
        Err(err) => {
            println!("Error creating chart: {:?}", err);
            return make_error_response_detail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating chart",
                err.to_string(),
            );
        }
    };

    println!("Chart created!");

    let response = match Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Content-Type", "image/png")
        .body(chart.into())
    {
        Ok(response) => response,
        Err(err) => {
            println!("Error creating response: {:?}", err);
            return make_error_response_detail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating response",
                err.to_string(),
            );
        }
    };

    println!("Sending response");

    Ok(response)
}
