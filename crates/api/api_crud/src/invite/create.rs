use actix_web::web::{Data, Json};
use base64::{Engine, engine::general_purpose};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::local_user_invite::{LocalUserInvite, LocalUserInviteInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_local_user_invite::{
  api::{CreateInvitation, CreateInvitationResponse},
  impls::LocalUserInviteQuery,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use uuid::Uuid;

pub async fn create_invitation(
  Json(data): Json<CreateInvitation>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CreateInvitationResponse>> {
  let pool = &mut context.pool();

  let local_user_id = local_user_view.local_user.id;

  let active_invite_count = LocalUserInviteQuery {
    local_user_id,
    ..Default::default()
  }
  .count(pool)
  .await?;

  if let Some(max) = context.settings().max_invites_per_user_allowed
    && is_admin(&local_user_view).is_err()
    && active_invite_count >= i64::from(max)
  {
    return Err(LemmyErrorType::TooManyInvites.into());
  }

  let token = generate_invite_token();

  let insert = LocalUserInviteInsertForm {
    token,
    local_user_id,
    max_uses: data.max_uses,
    expires_at: data.expires_at,
  };

  let invite = LocalUserInvite::create(pool, &insert).await?;

  Ok(Json(CreateInvitationResponse { invite }))
}

fn generate_invite_token() -> String {
  let id = Uuid::new_v4();
  // Convert to base64 for a more compact URL-friendly string
  general_purpose::URL_SAFE_NO_PAD.encode(id.as_bytes())
}
