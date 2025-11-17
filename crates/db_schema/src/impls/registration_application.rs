use crate::{
  newtypes::{LocalUserId, RegistrationApplicationId},
  source::registration_application::{
    RegistrationApplication,
    RegistrationApplicationInsertForm,
    RegistrationApplicationUpdateForm,
  },
};
use diesel::{ExpressionMethods, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::registration_application;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for RegistrationApplication {
  type InsertForm = RegistrationApplicationInsertForm;
  type UpdateForm = RegistrationApplicationUpdateForm;
  type IdType = RegistrationApplicationId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(registration_application::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id_: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(registration_application::table.find(id_))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl RegistrationApplication {
  pub async fn find_by_local_user_id(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    registration_application::table
      .filter(registration_application::local_user_id.eq(local_user_id_))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Fetches the most recent updated application.
  pub async fn last_updated(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    registration_application::table
      .filter(registration_application::updated_at.is_not_null())
      .order_by(registration_application::updated_at.desc())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// The duration between the last application creation, and its approval / denial time.
  ///
  /// Useful for estimating when your application will be approved.
  pub fn updated_published_duration(&self) -> Option<i64> {
    self
      .updated_at
      .map(|updated| (updated - self.published_at).num_seconds())
  }

  /// A missing admin id, means the application is unread
  #[diesel::dsl::auto_type(no_type_alias)]
  pub fn is_unread() -> _ {
    registration_application::admin_id.is_null()
  }
}
