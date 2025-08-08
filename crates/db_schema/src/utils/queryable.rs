use diesel::{
  deserialize::{FromStaticSqlRow, Queryable},
  pg::Pg,
};

pub struct LikeScore(Option<bool>);

impl From<LikeScore> for Option<i16> {
  fn from(val: LikeScore) -> Self {
    val.0.map(|is_positive| if is_positive { 1 } else { -1 })
  }
}

impl<ST> Queryable<ST, Pg> for LikeScore
where
  Option<bool>: FromStaticSqlRow<ST, Pg>,
{
  type Row = Option<bool>;
  fn build(value: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(LikeScore(value))
  }
}

pub struct NonNullLikeScore(bool);

impl From<NonNullLikeScore> for i16 {
  fn from(val: NonNullLikeScore) -> Self {
    if val.0 {
      1
    } else {
      -1
    }
  }
}

impl<ST> Queryable<ST, Pg> for NonNullLikeScore
where
  bool: FromStaticSqlRow<ST, Pg>,
{
  type Row = bool;
  fn build(value: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(NonNullLikeScore(value))
  }
}

pub struct ChangeNullTo<const N: i8, T>(Option<T>);

/* this won't compile

impl<const N: i8, T: From<i8>> Into<T> for ChangeNullTo<N, T> {
    fn into(self) -> T {
        self.0.unwrap_or(T::from(N))
    }
}*/

impl<const N: i8> From<ChangeNullTo<N, i32>> for i32 {
  fn from(val: ChangeNullTo<N, i32>) -> Self {
    val.0.unwrap_or(i32::from(N))
  }
}

impl<const N: i8> From<ChangeNullTo<N, i16>> for i16 {
  fn from(val: ChangeNullTo<N, i16>) -> Self {
    val.0.unwrap_or(i16::from(N))
  }
}

impl<const N: i8, T, U> Queryable<U, Pg> for ChangeNullTo<N, T>
where
  Option<T>: FromStaticSqlRow<U, Pg>,
{
  type Row = Option<T>;
  fn build(value: Self::Row) -> diesel::deserialize::Result<Self> {
    Ok(ChangeNullTo(value))
  }
}
