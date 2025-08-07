use diesel::{
  deserialize::{FromStaticSqlRow, Queryable},
  pg::Pg,
};

pub struct ChangeNullTo<const N: i8, T>(Option<T>);

/* this won't compile

impl<const N: i8, T: From<i8>> Into<T> for ChangeNullTo<N, T> {
    fn into(self) -> T {
        self.0.unwrap_or(T::from(N))
    }
}*/

impl<const N: i8> Into<i32> for ChangeNullTo<N, i32> {
  fn into(self) -> i32 {
    self.0.unwrap_or(i32::from(N))
  }
}

impl<const N: i8> Into<i16> for ChangeNullTo<N, i16> {
  fn into(self) -> i16 {
    self.0.unwrap_or(i16::from(N))
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
