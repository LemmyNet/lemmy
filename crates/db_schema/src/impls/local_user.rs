use crate::{
  newtypes::LocalUserId,
  schema::local_user::dsl::{
    accepted_application,
    email_verified,
    local_user,
    password_encrypted,
    validator_time,
  },
  source::{
    actor_language::{LocalUserLanguage, SiteLanguage},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, naive_now, DbPool},
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

mod safe_settings_type {
  use crate::{
    schema::local_user::columns::{
      accepted_application,
      default_listing_type,
      default_sort_type,
      email,
      email_verified,
      id,
      interface_language,
      person_id,
      send_notifications_to_email,
      show_avatars,
      show_bot_accounts,
      show_new_post_notifs,
      show_nsfw,
      show_read_posts,
      show_scores,
      theme,
      validator_time,
    },
    source::local_user::LocalUser,
    traits::ToSafeSettings,
  };

  type Columns = (
    id,
    person_id,
    email,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    interface_language,
    show_avatars,
    send_notifications_to_email,
    validator_time,
    show_bot_accounts,
    show_scores,
    show_read_posts,
    show_new_post_notifs,
    email_verified,
    accepted_application,
  );

  impl ToSafeSettings for LocalUser {
    type SafeSettingsColumns = Columns;

    /// Includes everything but the hashed password
    fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns {
      (
        id,
        person_id,
        email,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        interface_language,
        show_avatars,
        send_notifications_to_email,
        validator_time,
        show_bot_accounts,
        show_scores,
        show_read_posts,
        show_new_post_notifs,
        email_verified,
        accepted_application,
      )
    }
  }
}

impl LocalUser {
  pub async fn update_password(
    pool: &DbPool,
    local_user_id: LocalUserId,
    new_password: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(local_user.find(local_user_id))
      .set((
        password_encrypted.eq(password_hash),
        validator_time.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
      .await
  }

  pub async fn set_all_users_email_verified(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user)
      .set(email_verified.eq(true))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn set_all_users_registration_applications_accepted(
    pool: &DbPool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user)
      .set(accepted_application.eq(true))
      .get_results::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for LocalUser {
  type InsertForm = LocalUserInsertForm;
  type UpdateForm = LocalUserUpdateForm;
  type IdType = LocalUserId;
  async fn read(pool: &DbPool, local_user_id: LocalUserId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    local_user.find(local_user_id).first::<Self>(conn).await
  }
  async fn delete(pool: &DbPool, local_user_id: LocalUserId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_user.find(local_user_id))
      .execute(conn)
      .await
  }
  async fn create(pool: &DbPool, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut form_with_encrypted_password = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    form_with_encrypted_password.password_encrypted = password_hash;

    let local_user_ = insert_into(local_user)
      .values(form_with_encrypted_password)
      .get_result::<Self>(conn)
      .await
      .expect("couldnt create local user");

    let site_languages = SiteLanguage::read_local(pool).await;
    if let Ok(langs) = site_languages {
      // if site exists, init user with site languages
      LocalUserLanguage::update(pool, langs, local_user_.id).await?;
    } else {
      // otherwise, init with all languages (this only happens during tests and
      // for first admin user, which is created before site)
      LocalUserLanguage::update(pool, vec![], local_user_.id).await?;
    }

    Ok(local_user_)
  }
  async fn update(
    pool: &DbPool,
    local_user_id: LocalUserId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
