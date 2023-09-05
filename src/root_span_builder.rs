use actix_web::{http::StatusCode, ResponseError};
use tracing::Span;
use tracing_actix_web::RootSpanBuilder;

// Code in this module adapted from DefaultRootSpanBuilder
// https://github.com/LukeMathWalker/tracing-actix-web/blob/main/src/root_span_builder.rs
// and root_span!
// https://github.com/LukeMathWalker/tracing-actix-web/blob/main/src/root_span_macro.rs

pub struct QuieterRootSpanBuilder;

impl RootSpanBuilder for QuieterRootSpanBuilder {
    fn on_request_start(request: &actix_web::dev::ServiceRequest) -> Span {
        let request_id = tracing_actix_web::root_span_macro::private::get_request_id(request);

        tracing::info_span!(
            "HTTP request",
            http.method = %request.method(),
            http.scheme = request.connection_info().scheme(),
            http.host = %request.connection_info().host(),
            http.target = %request.uri().path(),
            http.status_code = tracing::field::Empty,
            otel.kind = "server",
            otel.status_code = tracing::field::Empty,
            trace_id = tracing::field::Empty,
            request_id = %request_id,
            exception.message = tracing::field::Empty,
            // Not proper OpenTelemetry, but their terminology is fairly exception-centric
            exception.details = tracing::field::Empty,
        )
    }

    fn on_request_end<B>(
        span: tracing::Span,
        outcome: &Result<actix_web::dev::ServiceResponse<B>, actix_web::Error>,
    ) {
        match &outcome {
            Ok(response) => {
                if let Some(error) = response.response().error() {
                    // use the status code already constructed for the outgoing HTTP response
                    handle_error(span, response.status(), error.as_response_error());
                } else {
                    let code: i32 = response.response().status().as_u16().into();
                    span.record("http.status_code", code);
                    span.record("otel.status_code", "OK");
                }
            }
            Err(error) => {
                let response_error = error.as_response_error();
                handle_error(span, response_error.status_code(), response_error);
            }
        };
    }
}

fn handle_error(span: Span, status_code: StatusCode, response_error: &dyn ResponseError) {
    let code: i32 = status_code.as_u16().into();

    span.record("http.status_code", code);

    if status_code.is_client_error() {
        span.record("otel.status_code", "OK");
    } else {
        span.record("otel.status_code", "ERROR");
    }

    // pre-formatting errors is a workaround for https://github.com/tokio-rs/tracing/issues/1565
    let display_error = format!("{response_error}");
    let debug_error = format!("{response_error:?}");

    tracing::info_span!(
      parent: None,
      "Error encountered while processing the incoming HTTP request"
    )
    .in_scope(|| {
        if status_code.is_client_error() {
            tracing::warn!("{}\n{}", display_error, debug_error);
        } else {
            tracing::error!("{}\n{}", display_error, debug_error);
        }
    });

    span.record("exception.message", &tracing::field::display(display_error));
    span.record("exception.details", &tracing::field::display(debug_error));
}
