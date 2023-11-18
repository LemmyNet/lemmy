#[cfg(feature = "full")]
use crate::schema::secret;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = secret))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Secret {
  pub id: i32,
  pub jwt_secret: String,
}
