use crate::{
  newtypes::LocalUserId,
  schema::local_user::dsl::*,
  source::{
    actor_language::{LocalUserLanguage, SiteLanguage},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, *};

mod safe_settings_type {
  use crate::{
    schema::local_user::columns::*,
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
  pub fn update_password(
    conn: &mut PgConnection,
    local_user_id: LocalUserId,
    new_password: &str,
  ) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(local_user.find(local_user_id))
      .set((
        password_encrypted.eq(password_hash),
        validator_time.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  pub fn set_all_users_email_verified(conn: &mut PgConnection) -> Result<Vec<Self>, Error> {
    diesel::update(local_user)
      .set(email_verified.eq(true))
      .get_results::<Self>(conn)
  }

  pub fn set_all_users_registration_applications_accepted(
    conn: &mut PgConnection,
  ) -> Result<Vec<Self>, Error> {
    diesel::update(local_user)
      .set(accepted_application.eq(true))
      .get_results::<Self>(conn)
  }
}

impl Crud for LocalUser {
  type InsertForm = LocalUserInsertForm;
  type UpdateForm = LocalUserUpdateForm;
  type IdType = LocalUserId;
  fn read(conn: &mut PgConnection, local_user_id: LocalUserId) -> Result<Self, Error> {
    local_user.find(local_user_id).first::<Self>(conn)
  }
  fn delete(conn: &mut PgConnection, local_user_id: LocalUserId) -> Result<usize, Error> {
    diesel::delete(local_user.find(local_user_id)).execute(conn)
  }
  fn create(conn: &mut PgConnection, form: &Self::InsertForm) -> Result<Self, Error> {
    let mut form_with_encrypted_password = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    form_with_encrypted_password.password_encrypted = password_hash;

    let local_user_ = insert_into(local_user)
      .values(form_with_encrypted_password)
      .get_result::<Self>(conn)?;

    let site_languages = SiteLanguage::read_local(conn);
    if let Ok(langs) = site_languages {
      // if site exists, init user with site languages
      LocalUserLanguage::update(conn, langs, local_user_.id)?;
    } else {
      // otherwise, init with all languages (this only happens during tests and
      // for first admin user, which is created before site)
      LocalUserLanguage::update(conn, vec![], local_user_.id)?;
    }

    Ok(local_user_)
  }
  fn update(
    conn: &mut PgConnection,
    local_user_id: LocalUserId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(local_user.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}
