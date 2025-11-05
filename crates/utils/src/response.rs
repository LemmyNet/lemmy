use crate::error::{LemmyError, LemmyErrorType};
use actix_web::{
  HttpRequest,
  HttpResponse,
  dev::ServiceResponse,
  middleware::ErrorHandlerResponse,
};

pub fn jsonify_plain_text_errors<BODY>(
  res: ServiceResponse<BODY>,
) -> actix_web::Result<ErrorHandlerResponse<BODY>> {
  let maybe_error = res.response().error();
  let is_rate_limit_error = res.status() == 429;

  // This function is only expected to be called for errors, so if there is no error, return
  if maybe_error.is_none() && !is_rate_limit_error {
    return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
  }
  // We're assuming that any LemmyError is already in JSON format, so we don't need to do anything
  if let Some(maybe_error) = maybe_error
    && maybe_error.as_error::<LemmyError>().is_some()
  {
    return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
  }

  // convert other errors to json format
  let (req, res_parts) = res.into_parts();
  let lemmy_err_type = if let Some(error) = res_parts.error() {
    LemmyErrorType::Unknown(error.to_string())
  } else if is_rate_limit_error {
    LemmyErrorType::TooManyRequests
  } else {
    LemmyErrorType::Unknown("couldnt build json".into())
  };
  build_error_response(req, res_parts, lemmy_err_type)
}

fn build_error_response<BODY>(
  req: HttpRequest,
  res_parts: HttpResponse<BODY>,
  err: LemmyErrorType,
) -> actix_web::Result<ErrorHandlerResponse<BODY>> {
  let response = HttpResponse::build(res_parts.status()).json(err);

  let service_response = ServiceResponse::new(req, response);
  Ok(ErrorHandlerResponse::Response(
    service_response.map_into_right_body(),
  ))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::error::{LemmyError, LemmyErrorType};
  use actix_web::{
    App,
    Error,
    Handler,
    Responder,
    error::ErrorInternalServerError,
    http::StatusCode,
    middleware::ErrorHandlers,
    test,
    web,
  };
  use pretty_assertions::assert_eq;

  #[actix_web::test]
  async fn test_non_error_responses_are_not_modified() {
    async fn ok_service() -> actix_web::Result<String, Error> {
      Ok("Oll Korrect".to_string())
    }

    check_for_jsonification(ok_service, StatusCode::OK, "Oll Korrect").await;
  }

  #[actix_web::test]
  async fn test_lemmy_errors_are_not_modified() {
    async fn lemmy_error_service() -> actix_web::Result<String, LemmyError> {
      Err(LemmyError::from(LemmyErrorType::AlreadyExists))
    }

    check_for_jsonification(
      lemmy_error_service,
      StatusCode::BAD_REQUEST,
      "{\"error\":\"already_exists\"}",
    )
    .await;
  }

  #[actix_web::test]
  async fn test_generic_errors_are_jsonified_as_unknown_errors() {
    async fn generic_error_service() -> actix_web::Result<String, Error> {
      Err(ErrorInternalServerError("This is not a LemmyError"))
    }

    check_for_jsonification(
      generic_error_service,
      StatusCode::INTERNAL_SERVER_ERROR,
      "{\"error\":\"unknown\",\"message\":\"This is not a LemmyError\"}",
    )
    .await;
  }

  #[actix_web::test]
  async fn test_anyhow_errors_wrapped_in_lemmy_errors_are_jsonified_correctly() {
    async fn anyhow_error_service() -> actix_web::Result<String, LemmyError> {
      Err(LemmyError::from(anyhow::anyhow!("This is the inner error")))
    }

    check_for_jsonification(
      anyhow_error_service,
      StatusCode::BAD_REQUEST,
      "{\"error\":\"unknown\",\"message\":\"This is the inner error\"}",
    )
    .await;
  }

  #[actix_web::test]
  async fn test_rate_limit_error() {
    async fn lemmy_error_service() -> actix_web::Result<HttpResponse> {
      Ok(HttpResponse::TooManyRequests().finish())
    }

    check_for_jsonification(
      lemmy_error_service,
      StatusCode::TOO_MANY_REQUESTS,
      "{\"error\":\"too_many_requests\"}",
    )
    .await;
  }

  async fn check_for_jsonification(
    service: impl Handler<(), Output = impl Responder + 'static>,
    expected_status_code: StatusCode,
    expected_body: &str,
  ) {
    let app = test::init_service(
      App::new()
        .wrap(ErrorHandlers::new().default_handler(jsonify_plain_text_errors))
        .route("/", web::get().to(service)),
    )
    .await;
    let req = test::TestRequest::default().to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), expected_status_code);

    let body = test::read_body(res).await;
    assert_eq!(body, expected_body);
  }
}
