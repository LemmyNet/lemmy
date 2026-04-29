use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::local_user_invite::{LocalUserInvite, LocalUserInviteInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_local_user_invite::{
  api::{CreateInvitation, CreateInvitationResponse},
  impls::LocalUserInviteQuery,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use rand::{RngExt, distr::Alphanumeric};

pub async fn create_invitation(
  Json(data): Json<CreateInvitation>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CreateInvitationResponse>> {
  let pool = &mut context.pool();

  let local_user_id = local_user_view.local_user.id;
  let local_site = SiteView::read_local(pool).await?.local_site;

  let active_invite_count = LocalUserInviteQuery {
    local_user_id,
    ..Default::default()
  }
  .count(pool)
  .await?;

  if is_admin(&local_user_view).is_err()
    && active_invite_count >= i64::from(local_site.max_invites_per_user_allowed)
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
  rand::rng()
    .sample_iter(Alphanumeric)
    .take(12)
    .map(char::from)
    .collect()
}
