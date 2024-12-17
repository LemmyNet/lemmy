use super::utils::{adapt_request, delete_old_image, make_send};
use actix_web::{self, web::*, HttpRequest};
use lemmy_api_common::{
  context::LemmyContext,
  image::UploadImageResponse,
  request::{PictrsFile, PictrsResponse},
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    images::{LocalImage, LocalImageForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;
use reqwest::Body;
use std::time::Duration;
use url::Url;

pub async fn upload_image(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  if context.settings().pictrs()?.image_upload_disabled {
    return Err(LemmyErrorType::ImageUploadDisabled.into());
  }

  let image = do_upload_image(req, body, UploadType::Other, &local_user_view, &context).await?;

  let image_url = image.image_url(&context.settings().get_protocol_and_hostname())?;
  Ok(Json(UploadImageResponse {
    image_url,
    filename: image.file,
    delete_token: image.delete_token,
  }))
}

pub async fn upload_avatar(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let image = do_upload_image(req, body, UploadType::Avatar, &local_user_view, &context).await?;

  delete_old_image(&local_user_view.person.avatar, &context).await?;

  let avatar = format!(
    "{}/api/v4/image/{}",
    context.settings().get_protocol_and_hostname(),
    image.file
  );
  let avatar = Some(Some(Url::parse(&avatar)?.into()));
  let person_form = PersonUpdateForm {
    avatar,
    ..Default::default()
  };

  Person::update(&mut context.pool(), local_user_view.person.id, &person_form).await?;

  Ok(Json(SuccessResponse::default()))
}
pub enum UploadType {
  Avatar,
  Other,
}

pub async fn do_upload_image(
  req: HttpRequest,
  body: Payload,
  upload_type: UploadType,
  local_user_view: &LocalUserView,
  context: &Data<LemmyContext>,
) -> LemmyResult<PictrsFile> {
  let pictrs_config = context.settings().pictrs()?;
  let image_url = format!("{}image", pictrs_config.url);

  let mut client_req = adapt_request(&req, image_url);

  client_req = match upload_type {
    UploadType::Avatar => {
      let max_size = context.settings().pictrs()?.max_avatar_size.to_string();
      client_req.query(&[
        ("resize", max_size.as_ref()),
        ("allow_animation", "false"),
        ("allow_video", "false"),
      ])
    }
    // TODO: same as above but using `max_banner_size`
    // UploadType::Banner => {}
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
    let thumbnail_url = image.image_url(&protocol_and_hostname)?;

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
