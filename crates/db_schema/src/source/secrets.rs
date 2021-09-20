use crate::schema::secrets;

#[derive(Queryable, Identifiable)]
#[table_name = "secrets"]
pub struct Secrets {
  pub id: i32,
  pub jwt_secret: String,
}
