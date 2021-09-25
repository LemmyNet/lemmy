use crate::schema::secret;

#[derive(Queryable, Identifiable, Clone)]
#[table_name = "secret"]
pub struct Secret {
  pub id: i32,
  pub jwt_secret: String,
}
