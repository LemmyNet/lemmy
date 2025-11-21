use crate::dburl::DbUrl;
use diesel::{
  Expression,
  IntoSql,
  dsl,
  helper_types::AsExprOf,
  pg::{Pg, data_types::PgInterval},
  query_builder::{Query, QueryFragment, QueryId},
  query_dsl::methods::LimitDsl,
  result::Error::{self as DieselError},
  sql_types::{self, Timestamptz},
};
use futures_util::future::BoxFuture;
use i_love_jesus::CursorKey;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::validation::clean_url,
};
use url::Url;

/// Necessary to be able to use cursors with the lower SQL function
pub struct LowerKey<K>(pub K);

impl<K, C> CursorKey<C> for LowerKey<K>
where
  K: CursorKey<C, SqlType = sql_types::Text>,
{
  type SqlType = sql_types::Text;
  type CursorValue = functions::lower<K::CursorValue>;
  type SqlValue = functions::lower<K::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    functions::lower(K::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::lower(K::get_sql_value())
  }
}

/// Necessary to be able to use cursors with the subpath SQL function
pub struct Subpath<K>(pub K);

impl<K, C> CursorKey<C> for Subpath<K>
where
  K: CursorKey<C, SqlType = diesel_ltree::sql_types::Ltree>,
{
  type SqlType = diesel_ltree::sql_types::Ltree;
  type CursorValue = diesel_ltree::subpath<K::CursorValue, i32, i32>;
  type SqlValue = diesel_ltree::subpath<K::SqlValue, i32, i32>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    diesel_ltree::subpath(K::get_cursor_value(cursor), 0, -1)
  }

  fn get_sql_value() -> Self::SqlValue {
    diesel_ltree::subpath(K::get_sql_value(), 0, -1)
  }
}

pub struct CoalesceKey<A, B>(pub A, pub B);

impl<A, B, C> CursorKey<C> for CoalesceKey<A, B>
where
  A: CursorKey<C, SqlType = sql_types::Nullable<B::SqlType>>,
  B: CursorKey<C, SqlType: Send>,
{
  type SqlType = B::SqlType;
  type CursorValue = functions::coalesce<B::SqlType, A::CursorValue, B::CursorValue>;
  type SqlValue = functions::coalesce<B::SqlType, A::SqlValue, B::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // TODO: for slight optimization, use unwrap_or_else here (this requires the CursorKey trait to
    // be changed to allow non-binded CursorValue)
    functions::coalesce(A::get_cursor_value(cursor), B::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::coalesce(A::get_sql_value(), B::get_sql_value())
  }
}

/// Includes an SQL comment before `T`, which can be used to label auto_explain output
#[derive(QueryId)]
pub struct Commented<T> {
  comment: String,
  inner: T,
}

impl<T> Commented<T> {
  pub fn new(inner: T) -> Self {
    Commented {
      comment: String::new(),
      inner,
    }
  }

  /// Adds `text` to the comment if `condition` is true
  fn text_if(mut self, text: &str, condition: bool) -> Self {
    if condition {
      if !self.comment.is_empty() {
        self.comment.push_str(", ");
      }
      self.comment.push_str(text);
    }
    self
  }

  /// Adds `text` to the comment
  pub fn text(self, text: &str) -> Self {
    self.text_if(text, true)
  }
}

impl<T: Query> Query for Commented<T> {
  type SqlType = T::SqlType;
}

impl<T: QueryFragment<Pg>> QueryFragment<Pg> for Commented<T> {
  fn walk_ast<'b>(
    &'b self,
    mut out: diesel::query_builder::AstPass<'_, 'b, Pg>,
  ) -> Result<(), DieselError> {
    for line in self.comment.lines() {
      out.push_sql("\n-- ");
      out.push_sql(line);
    }
    out.push_sql("\n");
    self.inner.walk_ast(out.reborrow())
  }
}

impl<T: LimitDsl> LimitDsl for Commented<T> {
  type Output = Commented<T::Output>;

  fn limit(self, limit: i64) -> Self::Output {
    Commented {
      comment: self.comment,
      inner: self.inner.limit(limit),
    }
  }
}

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q
    .replace('\\', "\\\\")
    .replace('%', "\\%")
    .replace('_', "\\_")
    .replace(' ', "%");
  format!("%{replaced}%")
}

/// Takes an API optional text input, and converts it to an optional diesel DB update.
pub fn diesel_string_update(opt: Option<&str>) -> Option<Option<String>> {
  match opt {
    // An empty string is an erase
    Some("") => Some(None),
    Some(str) => Some(Some(str.into())),
    None => None,
  }
}

/// Takes an API optional number, and converts it to an optional diesel DB update. Zero means erase.
pub fn diesel_opt_number_update(opt: Option<i32>) -> Option<Option<i32>> {
  match opt {
    // Zero is an erase
    Some(0) => Some(None),
    Some(num) => Some(Some(num)),
    None => None,
  }
}

/// Takes an API optional text input, and converts it to an optional diesel DB update (for non
/// nullable properties).
pub fn diesel_required_string_update(opt: Option<&str>) -> Option<String> {
  match opt {
    // An empty string is no change
    Some("") => None,
    Some(str) => Some(str.into()),
    None => None,
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB update.
/// Also cleans the url params.
pub fn diesel_url_update(opt: Option<&str>) -> LemmyResult<Option<Option<DbUrl>>> {
  match opt {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(Some(clean_url(&u).into())))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB update (for non
/// nullable properties). Also cleans the url params.
pub fn diesel_required_url_update(opt: Option<&str>) -> LemmyResult<Option<DbUrl>> {
  match opt {
    // An empty string is no change
    Some("") => Ok(None),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(clean_url(&u).into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB create.
/// Also cleans the url params.
pub fn diesel_url_create(opt: Option<&str>) -> LemmyResult<Option<DbUrl>> {
  match opt {
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(clean_url(&u).into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

pub mod functions {
  use diesel::{
    define_sql_function,
    sql_types::{Int4, Text, Timestamptz},
  };

  define_sql_function! {
    #[sql_name = "r.hot_rank"]
    fn hot_rank(score: Int4, time: Timestamptz) -> Float;
  }

  define_sql_function! {
    #[sql_name = "r.scaled_rank"]
    fn scaled_rank(score: Int4, time: Timestamptz, interactions_month: Int4) -> Float;
  }

  define_sql_function!(fn lower(x: Text) -> Text);

  define_sql_function!(fn random() -> Text);

  define_sql_function!(fn random_smallint() -> SmallInt);

  // really this function is variadic, this just adds the two-argument version
  define_sql_function!(fn coalesce<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: T) -> T);

  define_sql_function! {
    #[aggregate]
    fn json_agg<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(obj: T) -> Json
  }

  define_sql_function!(#[sql_name = "coalesce"] fn coalesce_2_nullable<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: diesel::sql_types::Nullable<T>) -> diesel::sql_types::Nullable<T>);

  define_sql_function!(#[sql_name = "coalesce"] fn coalesce_3_nullable<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: diesel::sql_types::Nullable<T>, z: diesel::sql_types::Nullable<T>) -> diesel::sql_types::Nullable<T>);
}

pub fn now() -> AsExprOf<diesel::dsl::now, diesel::sql_types::Timestamptz> {
  // https://github.com/diesel-rs/diesel/issues/1514
  diesel::dsl::now.into_sql::<Timestamptz>()
}

pub fn seconds_to_pg_interval(seconds: i32) -> PgInterval {
  PgInterval::from_microseconds(i64::from(seconds) * 1_000_000)
}

/// Output of `IntoSql::into_sql` for a type that implements `AsRecord`
pub type AsRecordOutput<T> = dsl::AsExprOf<T, sql_types::Record<<T as Expression>::SqlType>>;

pub type ResultFuture<'a, T> = BoxFuture<'a, Result<T, DieselError>>;

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_fuzzy_search() {
    let test = "This %is% _a_ fuzzy search";
    assert_eq!(
      fuzzy_search(test),
      "%This%\\%is\\%%\\_a\\_%fuzzy%search%".to_string()
    );
  }

  #[test]
  fn test_diesel_option_overwrite() {
    assert_eq!(diesel_string_update(None), None);
    assert_eq!(diesel_string_update(Some("")), Some(None));
    assert_eq!(
      diesel_string_update(Some("test")),
      Some(Some("test".to_string()))
    );
  }

  #[test]
  fn test_diesel_option_overwrite_to_url() -> LemmyResult<()> {
    assert!(matches!(diesel_url_update(None), Ok(None)));
    assert!(matches!(diesel_url_update(Some("")), Ok(Some(None))));
    assert!(diesel_url_update(Some("invalid_url")).is_err());
    let example_url = "https://example.com";
    assert!(matches!(
      diesel_url_update(Some(example_url)),
      Ok(Some(Some(url))) if url == Url::parse(example_url)?.into()
    ));
    Ok(())
  }
}
