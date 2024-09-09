use crate::structs::PostCommunityPostTags;
use diesel::{
  deserialize::FromSql,
  pg::{Pg, PgValue},
  serialize::ToSql,
  sql_types::{self, Nullable},
};

impl FromSql<Nullable<sql_types::Json>, Pg> for PostCommunityPostTags {
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<sql_types::Json, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value::<PostCommunityPostTags>(value)?)
  }
  fn from_nullable_sql(
    bytes: Option<<Pg as diesel::backend::Backend>::RawValue<'_>>,
  ) -> diesel::deserialize::Result<Self> {
    match bytes {
      Some(bytes) => Self::from_sql(bytes),
      None => Ok(Self { tags: vec![] }),
    }
  }
}

impl ToSql<Nullable<sql_types::Json>, Pg> for PostCommunityPostTags {
  fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<sql_types::Json, Pg>>::to_sql(&value, &mut out.reborrow())
  }
}
