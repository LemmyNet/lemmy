use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BannedPersonsResponse, GetBannedPersons},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for GetBannedPersons {
  type Response = BannedPersonsResponse;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &GetBannedPersons = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let banned = PersonView::banned(&mut context.pool()).await?;

    Ok(Self::Response { banned })
  }
}
