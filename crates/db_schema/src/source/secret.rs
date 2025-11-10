#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::secret;
use lemmy_diesel_utils::sensitive::SensitiveString;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = secret))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Secret {
  pub id: i32,
  pub jwt_secret: SensitiveString,
}
