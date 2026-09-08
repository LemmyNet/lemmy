use crate::{
  newtypes::InvitationId,
  source::local_user_invite::{
    LocalUserInvite,
    LocalUserInviteInsertForm,
    LocalUserInviteUpdateForm,
  },
};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::local_user_invite;
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

  pub async fn delete_by_token(pool: &mut DbPool<'_>, token: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_user_invite::table.filter(local_user_invite::token.eq(token)))
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorConversion for LocalUserInvite {
  type PaginatedType = LocalUserInvite;

  fn to_cursor(&self) -> CursorData {
    CursorData::new_plain(self.token.clone())
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    LocalUserInvite::read_by_token(pool, &cursor.plain()).await
  }
}

impl LocalUserInvite {
  pub fn is_expired(&self) -> bool {
    self.expires_at.map(|d| d < Utc::now()).unwrap_or(false)
  }
  pub fn get_invite_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let protocol_and_hostname = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!(
      "{}/signup?token={}",
      protocol_and_hostname, self.token
    ))?)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    newtypes::{InvitationId, LocalUserId},
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      local_user_invite::{LocalUserInvite, LocalUserInviteInsertForm, LocalUserInviteUpdateForm},
      person::{Person, PersonInsertForm},
    },
  };
  use chrono::{Duration, Utc};
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, traits::Crud};
  use lemmy_utils::{error::LemmyResult, settings::structs::Settings};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  fn make_invite(
    token: &str,
    max_uses: Option<i32>,
    uses_count: i32,
    expired: bool,
  ) -> LocalUserInvite {
    LocalUserInvite {
      id: InvitationId(1),
      token: token.to_string(),
      local_user_id: LocalUserId(1),
      max_uses,
      uses_count,
      expires_at: expired.then(|| Utc::now() - Duration::hours(1)),
      published_at: Utc::now(),
    }
  }

  #[test]
  fn test_is_expired() {
    let past = LocalUserInvite {
      expires_at: Some(Utc::now() - Duration::seconds(1)),
      ..make_invite("t", None, 0, false)
    };
    let future = LocalUserInvite {
      expires_at: Some(Utc::now() + Duration::hours(1)),
      ..make_invite("t", None, 0, false)
    };
    let no_expiry = make_invite("t", None, 0, false);
    assert!(past.is_expired());
    assert!(!future.is_expired());
    assert!(!no_expiry.is_expired());
  }

  #[test]
  fn test_get_invite_url() -> LemmyResult<()> {
    let mut settings = Settings::default();
    settings.hostname = "example.com".to_string();
    settings.tls_enabled = true;
    let invite = make_invite("abc123", None, 0, false);
    let url = invite.get_invite_url(&settings)?;
    assert_eq!(url.as_str(), "https://example.com/signup?token=abc123");
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_create_read_delete() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let new_person = PersonInsertForm::test_form(inserted_instance.id, "invite_tester");
    let inserted_person = Person::create(pool, &new_person).await?;
    let new_local_user = LocalUserInsertForm::test_form(inserted_person.id);
    let inserted_local_user = LocalUser::create(pool, &new_local_user, vec![]).await?;

    let token = "test_invite_token_abc";
    let form = LocalUserInviteInsertForm {
      token: token.to_string(),
      local_user_id: inserted_local_user.id,
      max_uses: Some(10),
      expires_at: None,
    };
    let inserted = LocalUserInvite::create(pool, &form).await?;

    assert_eq!(inserted.token, token);
    assert_eq!(inserted.local_user_id, inserted_local_user.id);
    assert_eq!(inserted.max_uses, Some(10));
    assert_eq!(inserted.uses_count, 0);
    assert!(inserted.expires_at.is_none());

    let read = LocalUserInvite::read_by_token(pool, token).await?;
    assert_eq!(read.id, inserted.id);
    assert_eq!(read.token, inserted.token);

    let deleted = LocalUserInvite::delete_by_token(pool, token).await?;
    assert_eq!(deleted.id, inserted.id);

    let read_after_delete = LocalUserInvite::read_by_token(pool, token).await;
    assert!(read_after_delete.is_err());

    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_update_uses_count() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let new_person = PersonInsertForm::test_form(inserted_instance.id, "invite_updater");
    let inserted_person = Person::create(pool, &new_person).await?;
    let new_local_user = LocalUserInsertForm::test_form(inserted_person.id);
    let inserted_local_user = LocalUser::create(pool, &new_local_user, vec![]).await?;

    let token = "test_update_token_xyz";
    let form = LocalUserInviteInsertForm {
      token: token.to_string(),
      local_user_id: inserted_local_user.id,
      max_uses: Some(3),
      expires_at: None,
    };
    let inserted = LocalUserInvite::create(pool, &form).await?;
    assert_eq!(inserted.uses_count, 0);

    let update_form = LocalUserInviteUpdateForm {
      uses_count: Some(2),
    };
    let updated = LocalUserInvite::update(pool, inserted.id, &update_form).await?;
    assert_eq!(updated.uses_count, 2);

    LocalUserInvite::delete_by_token(pool, token).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;
    Ok(())
  }
}
