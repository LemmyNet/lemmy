//! Wrappers that can be used with the `deserialize_as` attribute when using the `Queryable` derive
//! macro.
use diesel::{
  deserialize::{FromStaticSqlRow, Queryable},
  pg::Pg,
};

pub struct NullableBoolToIntScore(Option<i16>);

pub struct BoolToIntScore(i16);

pub struct ChangeNullTo<const N: i8, T>(T);

impl<ST> Queryable<ST, Pg> for NullableBoolToIntScore
where
  Option<bool>: FromStaticSqlRow<ST, Pg>,
{
  type Row = Option<bool>;
  fn build(is_positive: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(NullableBoolToIntScore(is_positive.map(|x| {
      if x {
        1
      } else {
        -1
      }
    })))
  }
}

impl<ST> Queryable<ST, Pg> for BoolToIntScore
where
  bool: FromStaticSqlRow<ST, Pg>,
{
  type Row = bool;
  fn build(is_positive: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(BoolToIntScore(if is_positive { 1 } else { -1 }))
  }
}

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

impl From<NullableBoolToIntScore> for Option<i16> {
  fn from(value: NullableBoolToIntScore) -> Self {
    value.0
  }
}

impl From<BoolToIntScore> for i16 {
  fn from(value: BoolToIntScore) -> Self {
    value.0
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
