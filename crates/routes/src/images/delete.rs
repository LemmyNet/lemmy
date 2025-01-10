use super::utils::delete_old_image;
use actix_web::web::*;
use lemmy_api_common::{
  context::LemmyContext,
  image::{CommunityIdQuery, DeleteImageParams},
  utils::{is_admin, is_mod_or_admin},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    images::LocalImage,
    person::{Person, PersonUpdateForm},
    site::{Site, SiteUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn delete_site_icon(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let site = Site::read_local(&mut context.pool()).await?;
  is_admin(&local_user_view)?;

  delete_old_image(&site.icon, &context).await?;

  let form = SiteUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
pub async fn delete_site_banner(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let site = Site::read_local(&mut context.pool()).await?;
  is_admin(&local_user_view)?;

  delete_old_image(&site.banner, &context).await?;

  let form = SiteUpdateForm {
    banner: Some(None),
    ..Default::default()
  };
  Site::update(&mut context.pool(), site.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_community_icon(
  data: Json<CommunityIdQuery>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community = Community::read(&mut context.pool(), data.id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view.person, community.id).await?;

  delete_old_image(&community.icon, &context).await?;

  let form = CommunityUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Community::update(&mut context.pool(), community.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_community_banner(
  data: Json<CommunityIdQuery>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community = Community::read(&mut context.pool(), data.id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view.person, community.id).await?;

  delete_old_image(&community.icon, &context).await?;

  let form = CommunityUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Community::update(&mut context.pool(), community.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_user_avatar(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  delete_old_image(&local_user_view.person.avatar, &context).await?;

  let form = PersonUpdateForm {
    avatar: Some(None),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_user_banner(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  delete_old_image(&local_user_view.person.banner, &context).await?;

  let form = PersonUpdateForm {
    banner: Some(None),
    ..Default::default()
  };
  Person::update(&mut context.pool(), local_user_view.person.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

// TODO: get rid of delete tokens and allow deletion by admin or uploader
pub async fn delete_image(
  data: Json<DeleteImageParams>,
  context: Data<LemmyContext>,
  // require login
  _local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!(
    "{}image/delete/{}/{}",
    pictrs_config.url, &data.token, &data.filename
  );

  context
    .pictrs_client()
    .delete(url)
    .send()
    .await?
    .error_for_status()?;

  LocalImage::delete_by_alias(&mut context.pool(), &data.filename).await?;

  Ok(Json(SuccessResponse::default()))
}
