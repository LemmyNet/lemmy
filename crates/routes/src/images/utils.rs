use actix_web::{
  http::{
    header::{HeaderName, ACCEPT_ENCODING, HOST},
    Method,
    StatusCode,
  },
  web::{Data, Payload},
  HttpRequest,
};
use futures::stream::{Stream, StreamExt};
use http::HeaderValue;
use lemmy_api_common::{
  context::LemmyContext,
  request::{client_builder, delete_image_from_pictrs, PictrsFile, PictrsResponse},
  LemmyErrorType,
};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::images::{LocalImage, LocalImageForm},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, settings::SETTINGS, REQWEST_TIMEOUT};
use reqwest::Body;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_tracing::TracingMiddleware;
use serde::Deserialize;
use std::{sync::LazyLock, time::Duration};
use url::Url;

// Pictrs cannot use proxy
pub(super) static PICTRS_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
  ClientBuilder::new(
    client_builder(&SETTINGS)
      .no_proxy()
      .build()
      .expect("build pictrs client"),
  )
  .with(TracingMiddleware::default())
  .build()
});

#[derive(Deserialize, Clone)]
pub struct PictrsGetParams {
  format: Option<String>,
  thumbnail: Option<i32>,
}

pub(super) trait ProcessUrl {
  /// If thumbnail or format is given, this uses the pictrs process endpoint.
  /// Otherwise, it uses the normal pictrs url (IE image/original).
  fn process_url(&self, image_url: &str, pictrs_url: &Url) -> String;
}

impl ProcessUrl for PictrsGetParams {
  fn process_url(&self, src: &str, pictrs_url: &Url) -> String {
    if self.format.is_none() && self.thumbnail.is_none() {
      format!("{}image/original/{}", pictrs_url, src)
    } else {
      // Take file type from name, or jpg if nothing is given
      let format = self
        .clone()
        .format
        .unwrap_or_else(|| src.split('.').last().unwrap_or("jpg").to_string());

      let mut url = format!("{}image/process.{}?src={}", pictrs_url, format, src);

      if let Some(size) = self.thumbnail {
        url = format!("{url}&thumbnail={size}",);
      }
      url
    }
  }
}

pub(super) fn adapt_request(request: &HttpRequest, url: String) -> RequestBuilder {
  // remove accept-encoding header so that pictrs doesn't compress the response
  const INVALID_HEADERS: &[HeaderName] = &[ACCEPT_ENCODING, HOST];

  let client_request = PICTRS_CLIENT
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
pub(super) fn convert_method(method: &Method) -> http::Method {
  http::Method::from_bytes(method.as_str().as_bytes()).expect("method can be converted")
}

pub(super) fn convert_header<'a>(
  name: &'a http::HeaderName,
  value: &'a HeaderValue,
) -> (&'a str, &'a [u8]) {
  (name.as_str(), value.as_bytes())
}

pub(super) enum UploadType {
  Avatar,
  Other,
}

pub(super) async fn do_upload_image(
  req: HttpRequest,
  body: Payload,
  upload_type: UploadType,
  local_user_view: &LocalUserView,
  context: &Data<LemmyContext>,
) -> LemmyResult<PictrsFile> {
  let pictrs_config = context.settings().pictrs_config()?;
  let image_url = format!("{}image", pictrs_config.url);

  let mut client_req = adapt_request(&req, image_url);

  client_req = match upload_type {
    UploadType::Avatar => {
      let max_size = context
        .settings()
        .pictrs_config()?
        .max_thumbnail_size
        .to_string();
      client_req.query(&[
        ("max_width", max_size.as_ref()),
        ("max_height", max_size.as_ref()),
        ("allow_animation", "false"),
        ("allow_video", "false"),
      ])
    }
    _ => client_req,
  };
  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string())
  };
  let res = client_req
    .timeout(Duration::from_secs(pictrs_config.upload_timeout))
    .body(Body::wrap_stream(make_send(body)))
    .send()
    .await?
    .error_for_status()?;

  let mut images = res.json::<PictrsResponse>().await?;
  for image in &images.files {
    // Pictrs allows uploading multiple images in a single request. Lemmy doesnt need this,
    // but still a user may upload multiple and so we need to store all links in db for
    // to allow deletion via web ui.
    let form = LocalImageForm {
      local_user_id: Some(local_user_view.local_user.id),
      pictrs_alias: image.file.to_string(),
      pictrs_delete_token: image.delete_token.to_string(),
    };

    let protocol_and_hostname = context.settings().get_protocol_and_hostname();
    let thumbnail_url = image.thumbnail_url(&protocol_and_hostname)?;

    // Also store the details for the image
    let details_form = image.details.build_image_details_form(&thumbnail_url);
    LocalImage::create(&mut context.pool(), &form, &details_form).await?;
  }
  let image = images
    .files
    .pop()
    .ok_or(LemmyErrorType::InvalidImageUpload)?;

  Ok(image)
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
