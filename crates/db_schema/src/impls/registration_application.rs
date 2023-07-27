use crate::{
  newtypes::LocalUserId,
  schema::registration_application::dsl::{local_user_id, registration_application},
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

#[async_trait]
impl Crud for RegistrationApplication {
  type InsertForm<'a> = &'a RegistrationApplicationInsertForm;
  type UpdateForm<'a> = &'a RegistrationApplicationUpdateForm;
  type IdType = i32;

  async fn create<'a>(pool: &mut DbPool<'_>, form: Self::InsertForm<'a>) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(registration_application)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update<'a>(
    pool: &mut DbPool<'_>,
    id_: Self::IdType,
    form: Self::UpdateForm<'a>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(registration_application.find(id_))
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
    registration_application
      .filter(local_user_id.eq(local_user_id_))
      .first::<Self>(conn)
      .await
  }
}
