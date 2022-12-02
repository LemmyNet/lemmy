use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeItemResponse, PurgePerson},
  utils::{get_local_user_view_from_jwt, is_admin, purge_image_posts_for_person},
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePerson, AdminPurgePersonForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for PurgePerson {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    // Read the person to get their images
    let person_id = data.person_id;
    let person = Person::read(context.pool(), person_id).await?;

    if let Some(banner) = person.banner {
      purge_image_from_pictrs(context.client(), context.settings(), &banner)
        .await
        .ok();
    }

    if let Some(avatar) = person.avatar {
      purge_image_from_pictrs(context.client(), context.settings(), &avatar)
        .await
        .ok();
    }

    purge_image_posts_for_person(
      person_id,
      context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;

    Person::delete(context.pool(), person_id).await?;

    // Mod tables
    let reason = data.reason.clone();
    let form = AdminPurgePersonForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    AdminPurgePerson::create(context.pool(), &form).await?;

    Ok(PurgeItemResponse { success: true })
  }
}
