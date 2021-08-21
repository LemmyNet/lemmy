use crate::PerformCrud;
use actix_web::web::Data;
use bcrypt::verify;
use lemmy_api_common::{get_local_user_view_from_jwt, person::*};
use lemmy_db_queries::source::{comment::Comment_, person::Person_, post::Post_};
use lemmy_db_schema::source::{comment::Comment, person::*, post::Post};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteAccount {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &DeleteAccount = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(ApiError::err("password_incorrect").into());
    }

    // Comments
    let person_id = local_user_view.person.id;
    let permadelete = Comment::permadelete_for_creator(&&context.pool.get().await?, person_id);
    if permadelete.is_err() {
      return Err(ApiError::err("couldnt_update_comment").into());
    }

    // Posts
    let permadelete = Post::permadelete_for_creator(&&context.pool.get().await?, person_id);
    if permadelete.is_err() {
      return Err(ApiError::err("couldnt_update_post").into());
    }

    Person::delete_account(&&context.pool.get().await?, person_id)?;

    Ok(LoginResponse {
      jwt: data.auth.to_owned(),
    })
  }
}
