use crate::{newtypes::LocalUserId, source::registration_application::*, traits::Crud};
use diesel::{insert_into, result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

impl Crud for RegistrationApplication {
  type Form = RegistrationApplicationForm;
  type IdType = i32;
  fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
    use crate::schema::registration_application::dsl::*;
    insert_into(registration_application)
      .values(form)
      .get_result::<Self>(conn)
  }

  fn read(conn: &mut PgConnection, id_: Self::IdType) -> Result<Self, Error> {
    use crate::schema::registration_application::dsl::*;
    registration_application.find(id_).first::<Self>(conn)
  }

  fn update(conn: &mut PgConnection, id_: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
    use crate::schema::registration_application::dsl::*;
    diesel::update(registration_application.find(id_))
      .set(form)
      .get_result::<Self>(conn)
  }

  fn delete(conn: &mut PgConnection, id_: Self::IdType) -> Result<usize, Error> {
    use crate::schema::registration_application::dsl::*;
    diesel::delete(registration_application.find(id_)).execute(conn)
  }
}

impl RegistrationApplication {
  pub fn find_by_local_user_id(
    conn: &mut PgConnection,
    local_user_id_: LocalUserId,
  ) -> Result<Self, Error> {
    use crate::schema::registration_application::dsl::*;
    registration_application
      .filter(local_user_id.eq(local_user_id_))
      .first::<Self>(conn)
  }
}
