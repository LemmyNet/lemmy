use crate::connection::DbPool;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use i_love_jesus::{PaginatedQueryBuilder, SortDirection};
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// TODO: at the moment we only store the item id in the cursor, and later we read it again from
//       the db. instead we could store the whole item in the cursor to avoid this db read (or
//       store only the fields which are used for sorting).

pub trait PaginationCursorBuilderNew {
  type CursorData;

  fn to_cursor(&self) -> (Option<char>, i32);

  fn from_cursor(
    prefix: Option<char>,
    id: i32,
    conn: &mut DbPool<'_>,
  ) -> impl Future<Output = LemmyResult<Self::CursorData>> + Send;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct PaginationCursorNew(String);

impl PaginationCursorNew {
  fn to_internal(self) -> PaginationCursorNewInternal {
    let encoded = BASE64_URL_SAFE_NO_PAD.decode(self.0).unwrap();
    serde_urlencoded::from_str(&String::from_utf8(encoded).unwrap()).unwrap()
  }
  fn from_internal(other: PaginationCursorNewInternal) -> Self {
    Self(BASE64_URL_SAFE_NO_PAD.encode(serde_urlencoded::to_string(other).unwrap()))
  }
}

pub async fn paginate_new<Q, T: PaginationCursorBuilderNew>(
  query: Q,
  cursor: Option<PaginationCursorNew>,
  sort_direction: SortDirection,
  pool: &mut DbPool<'_>,
) -> PaginatedQueryBuilder<T::CursorData, Q> {
  let (page_after, back) = if let Some(cursor) = cursor {
    let internal = cursor.to_internal();
    let object = T::from_cursor(internal.prefix, internal.id, pool)
      .await
      .unwrap();
    (Some(object), Some(internal.back))
  } else {
    (None, None)
  };
  paginate(query, sort_direction, page_after, None, back)
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct PaginationCursorNewInternal {
  #[serde(rename = "b")]
  back: bool,
  #[serde(rename = "p")]
  pub prefix: Option<char>,
  #[serde(rename = "i")]
  id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PaginatedVec<T>
where
  T: PaginationCursorBuilderNew,
{
  pub data: Vec<T>,
  pub next_page: Option<PaginationCursorNew>,
  pub prev_page: Option<PaginationCursorNew>,
}

pub fn paginate_response<T>(data: Vec<T>, limit: i64) -> PaginatedVec<T>
where
  T: PaginationCursorBuilderNew + Serialize + for<'a> Deserialize<'a>,
{
  let prev_page = data
    .first()
    .map(|d| d.to_cursor())
    .map(|(prefix, id)| PaginationCursorNewInternal {
      id,
      prefix,
      back: true,
    })
    .map(PaginationCursorNew::from_internal);

  let mut next_page = data
    .last()
    .map(|d| d.to_cursor())
    .map(|(prefix, id)| PaginationCursorNewInternal {
      id,
      prefix,
      back: false,
    })
    .map(PaginationCursorNew::from_internal);

  // If there are less than limit items we are on the last page, dont show next button.
  // Need to convert here because diesel takes i64 for limit while vec length is usize.
  let limit: usize = limit.try_into().unwrap_or_default();
  if data.len() < limit {
    next_page = None;
  }
  PaginatedVec {
    data,
    next_page,
    prev_page,
  }
}

// ------------------------------
// TODO: below are all old

pub trait PaginationCursorBuilder {
  type CursorData;

  /// Builds a pagination cursor for the given query result.
  fn to_cursor(&self) -> PaginationCursor;

  /// Reads a database row from a given pagination cursor.
  fn from_cursor(
    cursor: &PaginationCursor,
    conn: &mut DbPool<'_>,
  ) -> impl Future<Output = LemmyResult<Self::CursorData>> + Send;
}

/// A pagination cursor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PaginationCursor(pub String);

#[cfg(feature = "full")]
impl PaginationCursor {
  /// Used for tables that have a single primary key.
  /// IE the post table cursor looks like `P123`
  pub fn new_single(prefix: char, id: i32) -> Self {
    Self::new(&[(prefix, id)])
  }

  /// Some tables (like community_actions for example) have compound primary keys.
  /// This creates a cursor that can use both, like `C123-P123`
  pub fn new(prefixes_and_ids: &[(char, i32)]) -> Self {
    Self(
      prefixes_and_ids
        .iter()
        .map(|(prefix, id)|
          // hex encoding to prevent ossification
          format!("{prefix}{id:x}"))
        .collect::<Vec<String>>()
        .join("-"),
    )
  }

  pub fn prefixes_and_ids<const N: usize>(&self) -> LemmyResult<[(char, i32); N]> {
    use lemmy_utils::error::LemmyErrorType;

    let default_prefix = 'Z';
    let default_id = 0;
    self
      .0
      .split("-")
      .map(|i| {
        let opt = i.split_at_checked(1);
        if let Some((prefix_str, id_str)) = opt {
          let prefix = prefix_str.chars().next().unwrap_or(default_prefix);
          let id = i32::from_str_radix(id_str, 16).unwrap_or(default_id);
          (prefix, id)
        } else {
          (default_prefix, default_id)
        }
      })
      // TODO: use `Iterator::next_chunk` when it becomes available
      .collect::<Vec<_>>()
      .try_into()
      .map_err(|_vec| LemmyErrorType::CouldntParsePaginationToken.into())
  }
}

pub fn paginate<Q, C>(
  query: Q,
  sort_direction: SortDirection,
  page_after: Option<C>,
  page_before_or_equal: Option<C>,
  page_back: Option<bool>,
) -> PaginatedQueryBuilder<C, Q> {
  let mut query = PaginatedQueryBuilder::new(query, sort_direction);

  if page_back.unwrap_or_default() {
    query = query
      .before(page_after)
      .after_or_equal(page_before_or_equal)
      .limit_and_offset_from_end();
  } else {
    query = query
      .after(page_after)
      .before_or_equal(page_before_or_equal);
  }

  query
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_cursor() {
    let data = PaginationCursorNewInternal {
      back: false,
      prefix: None,
      id: 123,
    };
    let encoded = PaginationCursorNew::from_internal(data.clone());
    assert_eq!("Yj1mYWxzZSZpPTEyMw", &encoded.0);
    let data2 = encoded.to_internal();
    assert_eq!(data, data2);
  }
}
