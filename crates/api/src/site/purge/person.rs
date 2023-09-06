use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  request::delete_image_from_pictrs,
  site::{PurgeItemResponse, PurgePerson},
  utils::{is_admin, local_user_view_from_jwt, sanitize_html_api_opt},
};
use lemmy_db_schema::{
  source::{
    image_upload::ImageUpload,
    moderator::{AdminPurgePerson, AdminPurgePersonForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn purge_person(
  data: Json<PurgePerson>,
  context: Data<LemmyContext>,
) -> Result<Json<PurgeItemResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the person to get their images
  let person_id = data.person_id;

  let local_user = LocalUserView::read_person(&mut context.pool(), person_id).await?;
  let pictrs_uploads =
    ImageUpload::get_all_by_local_user_id(&mut context.pool(), &local_user.local_user.id).await?;

  for upload in pictrs_uploads {
    delete_image_from_pictrs(&upload.pictrs_alias, &upload.pictrs_delete_token, &context)
      .await
      .ok();
  }

  Person::delete(&mut context.pool(), person_id).await?;

  // Mod tables
  let reason = sanitize_html_api_opt(&data.reason);
  let form = AdminPurgePersonForm {
    admin_person_id: local_user_view.person.id,
    reason,
  };

  AdminPurgePerson::create(&mut context.pool(), &form).await?;

  Ok(Json(PurgeItemResponse { success: true }))
}
