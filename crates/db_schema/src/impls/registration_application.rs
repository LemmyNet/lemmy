use crate::{
  newtypes::{LocalUserId, RegistrationApplicationId},
  source::registration_application::{
    RegistrationApplication,
    RegistrationApplicationInsertForm,
    RegistrationApplicationUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::registration_application;

impl Crud for RegistrationApplication {
  type InsertForm = RegistrationApplicationInsertForm;
  type UpdateForm = RegistrationApplicationUpdateForm;
  type IdType = RegistrationApplicationId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(registration_application::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id_: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(registration_application::table.find(id_))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl RegistrationApplication {
  pub async fn find_by_local_user_id(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    registration_application::table
      .filter(registration_application::local_user_id.eq(local_user_id_))
      .first(conn)
      .await
  }

  /// A missing admin id, means the application is unread
  #[diesel::dsl::auto_type(no_type_alias)]
  pub fn is_unread() -> _ {
    registration_application::admin_id.is_null()
  }
}
