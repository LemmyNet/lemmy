use super::utils::delete_old_image;
use actix_web::web::{self, *};
use lemmy_api_utils::{
  context::LemmyContext,
  request::{delete_image_alias, purge_image_from_pictrs},
  utils::{is_admin, is_mod_or_admin},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    community::{Community, CommunityUpdateForm},
    images::LocalImage,
    person::{Person, PersonUpdateForm},
    site::{Site, SiteUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
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
  community_id: Path<CommunityId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community_id = community_id.into_inner();
  let community = Community::read(&mut context.pool(), community_id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view, community.id).await?;

  delete_old_image(&community.icon, &context).await?;

  let form = CommunityUpdateForm {
    icon: Some(None),
    ..Default::default()
  };
  Community::update(&mut context.pool(), community.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn delete_community_banner(
  community_id: Path<CommunityId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let community_id = community_id.into_inner();
  let community = Community::read(&mut context.pool(), community_id).await?;
  is_mod_or_admin(&mut context.pool(), &local_user_view, community.id).await?;

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

/// Deletes an image for a specific user.
pub async fn delete_image(
  filename: web::Path<String>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  LocalImage::validate_by_alias_and_user(&mut context.pool(), &filename, local_user_view.person.id)
    .await?;

  delete_image_alias(&filename, &context).await?;

  Ok(Json(SuccessResponse::default()))
}

/// Deletes any image, only for admins.
pub async fn delete_image_admin(
  filename: web::Path<String>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  // Use purge, since it should remove any other aliases.
  purge_image_from_pictrs(&filename, &context).await?;

  Ok(Json(SuccessResponse::default()))
}
