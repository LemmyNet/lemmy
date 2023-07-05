use crate::{
  newtypes::LocalUserId,
  schema::registration_application::dsl::{local_user_id, registration_application},
  source::registration_application::{
    RegistrationApplication,
    RegistrationApplicationInsertForm,
    RegistrationApplicationUpdateForm,
  },
  traits::Crud,
  utils::{DbPool, GetConn},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for RegistrationApplication {
  type InsertForm = RegistrationApplicationInsertForm;
  type UpdateForm = RegistrationApplicationUpdateForm;
  type IdType = i32;

  async fn create(mut pool: &mut impl GetConn, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(registration_application)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn read(mut pool: &mut impl GetConn, id_: Self::IdType) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    registration_application.find(id_).first::<Self>(conn).await
  }

  async fn update(
    mut pool: &mut impl GetConn,
    id_: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::update(registration_application.find(id_))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn delete(mut pool: &mut impl GetConn, id_: Self::IdType) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::delete(registration_application.find(id_))
      .execute(conn)
      .await
  }
}

impl RegistrationApplication {
  pub async fn find_by_local_user_id(
    mut pool: &mut impl GetConn,
    local_user_id_: LocalUserId,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    registration_application
      .filter(local_user_id.eq(local_user_id_))
      .first::<Self>(conn)
      .await
  }
}
