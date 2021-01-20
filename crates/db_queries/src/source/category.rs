use crate::Crud;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{schema::category::dsl::*, source::category::*};

impl Crud<CategoryForm> for Category {
  fn read(conn: &PgConnection, category_id: i32) -> Result<Self, Error> {
    category.find(category_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, new_category: &CategoryForm) -> Result<Self, Error> {
    insert_into(category)
      .values(new_category)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    category_id: i32,
    new_category: &CategoryForm,
  ) -> Result<Self, Error> {
    diesel::update(category.find(category_id))
      .set(new_category)
      .get_result::<Self>(conn)
  }
}

pub trait Category_ {
  fn list_all(conn: &PgConnection) -> Result<Vec<Category>, Error>;
}

impl Category_ for Category {
  fn list_all(conn: &PgConnection) -> Result<Vec<Category>, Error> {
    category.load::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, source::category::Category_};
  use lemmy_db_schema::source::category::Category;

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let categories = Category::list_all(&conn).unwrap();
    let expected_first_category = Category {
      id: 1,
      name: "Discussion".into(),
    };

    assert_eq!(expected_first_category, categories[0]);
  }
}
