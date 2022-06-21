use crate::source::language::Language;
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl Language {
  pub fn read_all(conn: &PgConnection) -> Result<Vec<Language>, Error> {
    use crate::schema::language::dsl::*;
    language.load::<Self>(conn)
  }

  pub fn read_from_code(code_: &str, conn: &PgConnection) -> Result<Language, Error> {
    use crate::schema::language::dsl::*;
    language.find(code.eq(code_)).load::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{source::language::Language, utils::establish_unpooled_connection};
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_languages() {
    let conn = establish_unpooled_connection();

    let all = Language::read_all(&conn).unwrap();

    assert_eq!(123, all.len());
    assert_eq!("xy", all[5].code);
  }
}
