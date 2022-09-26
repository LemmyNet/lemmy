use crate::{newtypes::LanguageId, source::language::Language};
use diesel::{result::Error, PgConnection, RunQueryDsl, *};

impl Language {
  pub fn read_all(conn: &mut PgConnection) -> Result<Vec<Language>, Error> {
    use crate::schema::language::dsl::*;
    language.load::<Self>(conn)
  }

  pub fn read_from_id(conn: &mut PgConnection, id_: LanguageId) -> Result<Language, Error> {
    use crate::schema::language::dsl::*;
    language.filter(id.eq(id_)).first::<Self>(conn)
  }

  pub fn read_id_from_code(conn: &mut PgConnection, code_: &str) -> Result<LanguageId, Error> {
    use crate::schema::language::dsl::*;
    Ok(language.filter(code.eq(code_)).first::<Self>(conn)?.id)
  }

  pub fn read_id_from_code_opt(
    conn: &mut PgConnection,
    code_: Option<&str>,
  ) -> Result<Option<LanguageId>, Error> {
    if let Some(code_) = code_ {
      Ok(Some(Language::read_id_from_code(conn, code_)?))
    } else {
      Ok(None)
    }
  }

  pub fn read_undetermined(conn: &mut PgConnection) -> Result<LanguageId, Error> {
    use crate::schema::language::dsl::*;
    Ok(language.filter(code.eq("und")).first::<Self>(conn)?.id)
  }
}

#[cfg(test)]
mod tests {
  use crate::{source::language::Language, utils::establish_unpooled_connection};
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_languages() {
    let conn = &mut establish_unpooled_connection();

    let all = Language::read_all(conn).unwrap();

    assert_eq!(184, all.len());
    assert_eq!("ak", all[5].code);
    assert_eq!("lv", all[99].code);
    assert_eq!("yi", all[179].code);
  }
}
