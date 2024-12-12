use actix_web::http::{Method, StatusCode};
use futures::stream::{Stream, StreamExt};
use http::HeaderValue;

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

pub(super) struct SendStream<T> {
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
pub(super) fn convert_status(status: http::StatusCode) -> StatusCode {
  StatusCode::from_u16(status.as_u16()).expect("status can be converted")
}

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
