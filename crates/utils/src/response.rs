use crate::error::{LemmyError, LemmyErrorType};
use actix_web::{dev::ServiceResponse, middleware::ErrorHandlerResponse, HttpResponse};

pub fn jsonify_plain_text_errors<BODY>(
  res: ServiceResponse<BODY>,
) -> actix_web::Result<ErrorHandlerResponse<BODY>> {
  let maybe_error = res.response().error();

  // This function is only expected to be called for errors, so if there is no error, return
  if maybe_error.is_none() {
    return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
  }
  // We're assuming that any LemmyError is already in JSON format, so we don't need to do anything
  if maybe_error
    .expect("http responses with 400-599 statuses should have an error object")
    .as_error::<LemmyError>()
    .is_some()
  {
    return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
  }

  let (req, res) = res.into_parts();
  let error = res
    .error()
    .expect("expected an error object in the response");
  let response = HttpResponse::build(res.status()).json(LemmyErrorType::Unknown(error.to_string()));

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
    error::ErrorInternalServerError,
    middleware::ErrorHandlers,
    test,
    web,
    App,
    Error,
    Handler,
    Responder,
  };
  use http::StatusCode;
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
      Err(LemmyError::from(LemmyErrorType::EmailAlreadyExists))
    }

    check_for_jsonification(
      lemmy_error_service,
      StatusCode::BAD_REQUEST,
      "{\"error\":\"email_already_exists\"}",
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
