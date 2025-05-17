use super::utils::{adapt_request, delete_old_image, make_send};
use actix_web::{self, web::*, HttpRequest};
use lemmy_api_common::{
  context::LemmyContext,
  image::{CommunityIdQuery, UploadImageResponse},
  request::PictrsResponse,
  utils::{is_admin, is_mod_or_admin},
  LemmyErrorType,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    images::{LocalImage, LocalImageForm},
    person::{Person, PersonUpdateForm},
    site::{Site, SiteUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;
use reqwest::Body;
use std::time::Duration;
use UploadType::*;

pub enum UploadType {
  Avatar,
  Banner,
  Other,
}

pub async fn upload_image(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  if context.settings().pictrs()?.image_upload_disabled {
    return Err(LemmyErrorType::ImageUploadDisabled.into());
  }

  Ok(Json(
    do_upload_image(req, body, Other, &local_user_view, &context).await?,
  ))
}

pub async fn upload_user_avatar(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  let image = do_upload_image(req, body, Avatar, &local_user_view, &context).await?;
  delete_old_image(&local_user_view.person.avatar, &context).await?;

  let form = PersonUpdateForm {
    avatar: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(image))
}

pub async fn upload_user_banner(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  let image = do_upload_image(req, body, Banner, &local_user_view, &context).await?;
  delete_old_image(&local_user_view.person.banner, &context).await?;

  let form = PersonUpdateForm {
    banner: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(image))
}

pub async fn upload_community_icon(
  req: HttpRequest,
  query: Query<CommunityIdQuery>,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  let community: Community = Community::read(&mut context.pool(), query.id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view, community.id).await?;

  let image = do_upload_image(req, body, Avatar, &local_user_view, &context).await?;
  delete_old_image(&community.icon, &context).await?;

  let form = CommunityUpdateForm {
    icon: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Community::update(&mut context.pool(), community.id, &form).await?;

  Ok(Json(image))
}

pub async fn upload_community_banner(
  req: HttpRequest,
  query: Query<CommunityIdQuery>,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  let community: Community = Community::read(&mut context.pool(), query.id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view, community.id).await?;

  let image = do_upload_image(req, body, Banner, &local_user_view, &context).await?;
  delete_old_image(&community.banner, &context).await?;

  let form = CommunityUpdateForm {
    banner: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Community::update(&mut context.pool(), community.id, &form).await?;

  Ok(Json(image))
}

pub async fn upload_site_icon(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  is_admin(&local_user_view)?;
  let site = Site::read_local(&mut context.pool()).await?;

  let image = do_upload_image(req, body, Avatar, &local_user_view, &context).await?;
  delete_old_image(&site.icon, &context).await?;

  let form = SiteUpdateForm {
    icon: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(image))
}

pub async fn upload_site_banner(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<UploadImageResponse>> {
  is_admin(&local_user_view)?;
  let site = Site::read_local(&mut context.pool()).await?;

  let image = do_upload_image(req, body, Banner, &local_user_view, &context).await?;
  delete_old_image(&site.banner, &context).await?;

  let form = SiteUpdateForm {
    banner: Some(Some(image.image_url.clone().into())),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(image))
}

pub async fn do_upload_image(
  req: HttpRequest,
  body: Payload,
  upload_type: UploadType,
  local_user_view: &LocalUserView,
  context: &Data<LemmyContext>,
) -> LemmyResult<UploadImageResponse> {
  let pictrs = context.settings().pictrs()?;
  let max_upload_size = pictrs.max_upload_size.map(|m| m.to_string());
  let image_url = format!("{}image", pictrs.url);

  let mut client_req = adapt_request(&req, image_url, context);

  // Set pictrs parameters to downscale images and restrict file types.
  // https://git.asonix.dog/asonix/pict-rs/#api
  client_req = match upload_type {
    Avatar => {
      let max_size = pictrs.max_avatar_size.to_string();
      client_req.query(&[
        ("resize", max_size.as_ref()),
        ("allow_animation", "false"),
        ("allow_video", "false"),
      ])
    }
    Banner => {
      let max_size = pictrs.max_banner_size.to_string();
      client_req.query(&[
        ("resize", max_size.as_ref()),
        ("allow_animation", "false"),
        ("allow_video", "false"),
      ])
    }
    Other => {
      let mut query = vec![("allow_video", pictrs.allow_video_uploads.to_string())];
      if let Some(max_upload_size) = max_upload_size {
        query.push(("resize", max_upload_size));
      }
      client_req.query(&query)
    }
  };
  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string())
  };
  let res = client_req
    .timeout(Duration::from_secs(pictrs.upload_timeout))
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

  let url = image.image_url(&context.settings().get_protocol_and_hostname())?;
  Ok(UploadImageResponse {
    image_url: url,
    filename: image.file,
  })
}
