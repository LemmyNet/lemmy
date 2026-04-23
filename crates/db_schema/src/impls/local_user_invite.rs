use crate::{
  newtypes::{InvitationId, LocalUserId},
  source::local_user_invite::{
    LocalUserInvite,
    LocalUserInviteInsertForm,
    LocalUserInviteUpdateForm,
  },
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{enums::LocalUserInviteStatus, schema::local_user_invite};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{CursorData, PaginationCursorConversion},
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

impl LocalUserInvite {
  pub async fn read(pool: &mut DbPool<'_>, id: InvitationId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    local_user_invite::table
      .find(id)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &LocalUserInviteInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_user_invite::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    id: InvitationId,
    form: &LocalUserInviteUpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user_invite::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn read_by_token(pool: &mut DbPool<'_>, token: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    local_user_invite::table
      .filter(local_user_invite::token.eq(token))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_by_token_and_user(
    pool: &mut DbPool<'_>,
    local_user_id: &LocalUserId,
    token: &str,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    local_user_invite::table
      .filter(local_user_invite::local_user_id.eq(local_user_id))
      .filter(local_user_invite::token.eq(token))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorConversion for LocalUserInvite {
  type PaginatedType = LocalUserInvite;

  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    LocalUserInvite::read(pool, InvitationId(cursor.id()?)).await
  }
}

impl LocalUserInvite {
  pub fn is_exhausted(&self) -> bool {
    self.max_uses.map(|m| self.uses_count >= m).unwrap_or(false)
  }
  pub fn is_expired(&self) -> bool {
    self.expires_at.map(|d| d < Utc::now()).unwrap_or(false)
  }
  pub fn is_active(&self) -> bool {
    self.status == LocalUserInviteStatus::Active && !self.is_exhausted() && !self.is_expired()
  }
  pub fn get_invite_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let protocol_and_hostname = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!(
      "{}/signup?token={}",
      protocol_and_hostname, self.token
    ))?)
  }
}
