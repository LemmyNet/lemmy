use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeItemResponse, PurgePerson},
  utils::{is_admin, local_user_view_from_jwt, purge_image_posts_for_person, sanitize_html_opt},
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePerson, AdminPurgePersonForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for PurgePerson {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Only let admin purge an item
    is_admin(&local_user_view)?;

    // Read the person to get their images
    let person_id = data.person_id;
    let person = Person::read(&mut context.pool(), person_id).await?;

    if let Some(banner) = person.banner {
      purge_image_from_pictrs(&banner, context).await.ok();
    }

    if let Some(avatar) = person.avatar {
      purge_image_from_pictrs(&avatar, context).await.ok();
    }

    purge_image_posts_for_person(person_id, context).await?;

    Person::delete(&mut context.pool(), person_id).await?;

    // Mod tables
    let reason = sanitize_html_opt(&data.reason);
    let form = AdminPurgePersonForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    AdminPurgePerson::create(&mut context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
