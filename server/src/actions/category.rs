extern crate diesel;
use schema::{category};
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {Crud};

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name="category"]
pub struct Category {
  pub id: i32,
  pub name: String
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize)]
#[table_name="category"]
pub struct CategoryForm {
  pub name: String,
}

impl Crud<CategoryForm> for Category {
  fn read(conn: &PgConnection, category_id: i32) -> Result<Self, Error> {
    use schema::category::dsl::*;
    category.find(category_id)
      .first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, category_id: i32) -> Result<usize, Error> {
    use schema::category::dsl::*;
    diesel::delete(category.find(category_id))
      .execute(conn)
  }

  fn create(conn: &PgConnection, new_category: &CategoryForm) -> Result<Self, Error> {
    use schema::category::dsl::*;
      insert_into(category)
        .values(new_category)
        .get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, category_id: i32, new_category: &CategoryForm) -> Result<Self, Error> {
    use schema::category::dsl::*;
    diesel::update(category.find(category_id))
      .set(new_category)
      .get_result::<Self>(conn)
  }
}

impl Category {
  pub fn list_all(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use schema::category::dsl::*;
    category.load::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  // use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();

    let categories = Category::list_all(&conn).unwrap();
    let expected_first_category = Category {
      id: 1,
      name: "Discussion".into()
    };

    assert_eq!(expected_first_category, categories[0]);
  }
}
