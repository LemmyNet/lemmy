use actix_web::{
  http::{
    header::{HeaderName, ACCEPT_ENCODING, HOST},
    Method,
  },
  web::Data,
  HttpRequest,
};
use futures::stream::{Stream, StreamExt};
use http::HeaderValue;
use lemmy_api_common::{context::LemmyContext, request::delete_image_from_pictrs};
use lemmy_db_schema::{newtypes::DbUrl, source::images::LocalImage};
use lemmy_utils::{error::LemmyResult, REQWEST_TIMEOUT};
use reqwest_middleware::RequestBuilder;

pub(super) fn adapt_request(
  request: &HttpRequest,
  url: String,
  context: &LemmyContext,
) -> RequestBuilder {
  // remove accept-encoding header so that pictrs doesn't compress the response
  const INVALID_HEADERS: &[HeaderName] = &[ACCEPT_ENCODING, HOST];

  let client_request = context
    .pictrs_client()
    .request(convert_method(request.method()), url)
    .timeout(REQWEST_TIMEOUT);

  request
    .headers()
    .iter()
    .fold(client_request, |client_req, (key, value)| {
      if INVALID_HEADERS.contains(key) {
        client_req
      } else {
        // TODO: remove as_str and as_bytes conversions after actix-web upgrades to http 1.0
        client_req.header(key.as_str(), value.as_bytes())
      }
    })
}

pub(super) fn make_send<S>(mut stream: S) -> impl Stream<Item = S::Item> + Send + Unpin + 'static
where
  S: Stream + Unpin + 'static,
  S::Item: Send,
{
  // NOTE: the 8 here is arbitrary
  let (tx, rx) = tokio::sync::mpsc::channel(8);

  // NOTE: spawning stream into a new task can potentially hit this bug:
  // - https://github.com/actix/actix-web/issues/1679
  //
  // Since 4.0.0-beta.2 this issue is incredibly less frequent. I have not personally reproduced it.
  // That said, it is still technically possible to encounter.
  actix_web::rt::spawn(async move {
    while let Some(res) = stream.next().await {
      if tx.send(res).await.is_err() {
        break;
      }
    }
  });

  SendStream { rx }
}

struct SendStream<T> {
  rx: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Stream for SendStream<T>
where
  T: Send,
{
  type Item = T;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    std::pin::Pin::new(&mut self.rx).poll_recv(cx)
  }
}

// TODO: remove these conversions after actix-web upgrades to http 1.0
#[allow(clippy::expect_used)]
pub(super) fn convert_method(method: &Method) -> http::Method {
  http::Method::from_bytes(method.as_str().as_bytes()).expect("method can be converted")
}

pub(super) fn convert_header<'a>(
  name: &'a http::HeaderName,
  value: &'a HeaderValue,
) -> (&'a str, &'a [u8]) {
  (name.as_str(), value.as_bytes())
}

/// When adding a new avatar, banner or similar image, delete the old one.
pub(super) async fn delete_old_image(
  old_image: &Option<DbUrl>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  if let Some(old_image) = old_image {
    let image = LocalImage::delete_by_url(&mut context.pool(), old_image)
      .await
      .ok();
    if let Some(image) = image {
      delete_image_from_pictrs(&image.pictrs_alias, &image.pictrs_delete_token, context).await?;
    }
  }
  Ok(())
}
