use crate::schema::category;
use serde::Serialize;

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "category"]
pub struct Category {
  pub id: i32,
  pub name: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "category"]
pub struct CategoryForm {
  pub name: String,
}
