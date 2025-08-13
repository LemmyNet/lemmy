//! Wrappers that can be used with the `deserialize_as` attribute when using the `Queryable` derive
//! macro.
use diesel::{
  deserialize::{FromStaticSqlRow, Queryable},
  pg::Pg,
};

pub struct ChangeNullTo<const N: i8, T>(T);

impl<const N: i8, T, U> Queryable<U, Pg> for ChangeNullTo<N, T>
where
  Option<T>: FromStaticSqlRow<U, Pg>,
  T: From<i8>,
{
  type Row = Option<T>;
  fn build(value: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(ChangeNullTo(value.unwrap_or(N.into())))
  }
}

// Generic impl won't compile.
impl<const N: i8> From<ChangeNullTo<N, Self>> for i16 {
  fn from(value: ChangeNullTo<N, Self>) -> Self {
    value.0
  }
}

impl<const N: i8> From<ChangeNullTo<N, Self>> for i32 {
  fn from(value: ChangeNullTo<N, Self>) -> Self {
    value.0
  }
}
