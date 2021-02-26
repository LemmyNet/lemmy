use crate::{is_email_regex, ApubObject, Crud, ToSafeSettings};
mod safe_settings_type {
  use crate::ToSafeSettings;
  use lemmy_db_schema::{schema::user_::columns::*, source::user::User_};

  type Columns = (
    id,
    name,
    preferred_username,
    email,
    avatar,
    admin,
    banned,
    published,
    updated,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    lang,
    show_avatars,
    send_notifications_to_email,
    matrix_user_id,
    actor_id,
    bio,
    local,
    last_refreshed_at,
    banner,
    deleted,
  );

  impl ToSafeSettings for User_ {
    type SafeSettingsColumns = Columns;
    fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns {
      (
        id,
        name,
        preferred_username,
        email,
        avatar,
        admin,
        banned,
        published,
        updated,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        lang,
        show_avatars,
        send_notifications_to_email,
        matrix_user_id,
        actor_id,
        bio,
        local,
        last_refreshed_at,
        banner,
        deleted,
      )
    }
  }
}

pub trait UserSafeSettings_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<UserSafeSettings, Error>;
}

impl UserSafeSettings_ for UserSafeSettings {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_
      .select(User_::safe_settings_columns_tuple())
      .filter(deleted.eq(false))
      .find(user_id)
      .first::<Self>(conn)
  }
}
