use crate::connection::DbPool;
use base64::{
  Engine,
  alphabet::Alphabet,
  engine::{GeneralPurpose, general_purpose::NO_PAD},
};
use cfg_if::cfg_if;
use i_love_jesus::{PaginatedQueryBuilder, SortDirection};
use itertools::Itertools;
#[cfg(feature = "full")]
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::sync::LazyLock;

/// Use base 64 engine with custom alphabet based on base64::engine::general_purpose::URL_SAFE
/// with randomized character order, to prevent clients from parsing or modifying cursor data.
const BASE64_ENGINE: LazyLock<GeneralPurpose> = LazyLock::new(|| {
  let alphabet =
    Alphabet::new("AphruVFwvCetlckdZ2g-foxXBGNbyHnD96qUj3KL_YsE7P1OQiaIR0z4T58mMWJS").unwrap();
  GeneralPurpose::new(&alphabet, NO_PAD)
});

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CursorData(String);

impl CursorData {
  pub fn new(id: i32) -> Self {
    Self(id.to_string())
  }
  pub fn id(self) -> i32 {
    self.0.parse().unwrap()
  }
  pub fn new_with_prefix(prefix: char, id: i32) -> Self {
    Self(format!("{prefix},{id}"))
  }
  pub fn id_and_prefix(self) -> (char, i32) {
    let (prefix, id) = self.0.split_once(',').unwrap();
    (prefix.chars().next().unwrap(), id.parse().unwrap())
  }

  pub fn new_multi<const N: usize>(data: [(char, i32); N]) -> Self {
    Self(
      data
        .into_iter()
        .map(|(prefix, id)| format!("{prefix}:{id}"))
        .join(","),
    )
  }
  pub fn multi<const N: usize>(self) -> [(char, i32); N] {
    self
      .0
      .split(",")
      .flat_map(|y| y.split_once(":"))
      .map(|(prefix, id)| (prefix.chars().next().unwrap(), id.parse::<i32>().unwrap()))
      .collect::<Vec<_>>()
      .try_into()
      .unwrap()
  }
}
pub trait PaginationCursorBuilderNew {
  type PaginatedType;

  fn to_cursor(&self) -> CursorData;

  fn from_cursor(
    data: CursorData,
    conn: &mut DbPool<'_>,
  ) -> impl Future<Output = LemmyResult<Self::PaginatedType>> + Send;

  /// Paginate a db query.
  fn paginate_new<Q: Send>(
    query: Q,
    cursor: Option<PaginationCursorNew>,
    sort_direction: SortDirection,
    pool: &mut DbPool<'_>,
  ) -> impl std::future::Future<Output = LemmyResult<PaginatedQueryBuilder<Self::PaginatedType, Q>>> + Send
  {
    async move {
      let (page_after, back) = if let Some(cursor) = cursor {
        let internal = cursor.to_internal()?;
        let object = Self::from_cursor(internal.data, pool).await?;
        (Some(object), Some(internal.back))
      } else {
        (None, None)
      };
      Ok(paginate(query, sort_direction, page_after, None, back))
    }
  }
}

/// To get the next or previous page, pass this string unchanged as `page_cursor` in a new request
/// to the same endpoint.
///
/// Do not attempt to parse or modify the cursor string. The format is internal and may change in
/// minor Lemmy versions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct PaginationCursorNew(String);

impl PaginationCursorNew {
  fn to_internal(self) -> LemmyResult<PaginationCursorNewInternal> {
    let decoded = BASE64_ENGINE.decode(self.0)?;
    Ok(serde_urlencoded::from_str(&String::from_utf8(decoded)?)?)
  }
  fn from_internal(other: PaginationCursorNewInternal) -> LemmyResult<Self> {
    let encoded = BASE64_ENGINE.encode(serde_urlencoded::to_string(other)?);
    Ok(Self(encoded))
  }
}

/// The actual data which is stored inside a cursor, not accessible outside this file.
/// Uses serde rename to keep the cursor string short.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct PaginationCursorNewInternal {
  #[serde(rename = "b")]
  back: bool,
  #[serde(rename = "d")]
  data: CursorData,
}

// There seems to be no way to set generic bounds like `T: ts_rs::TS` based on feature flags,
// so we need to define the struct once with and once without the bound.
cfg_if! {
  if #[cfg(feature = "ts-rs")] {

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[derive(ts_rs::TS)]
    #[ts(optional_fields, export)]
    pub struct PaginatedVec<T: ts_rs::TS> {
      pub data: Vec<T>,
      pub next_page: Option<PaginationCursorNew>,
      pub prev_page: Option<PaginationCursorNew>,
    }
    pub fn paginate_response<T: ts_rs::TS>(data: Vec<T>, limit: i64) -> LemmyResult<PaginatedVec<T>>
    where
      T: PaginationCursorBuilderNew + Serialize + for<'a> Deserialize<'a>,
    {
      todo!()
    }
  } else {

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct PaginatedVec<T> {
      pub data: Vec<T>,
      pub next_page: Option<PaginationCursorNew>,
      pub prev_page: Option<PaginationCursorNew>,
    }

    /// Add prev/next cursors to query result.
    pub fn paginate_response<T>(data: Vec<T>, limit: i64) -> LemmyResult<PaginatedVec<T>>
    where
      T: PaginationCursorBuilderNew + Serialize + for<'a> Deserialize<'a>,
    {
      let make_cursor = |item: Option<&T>, back: bool| -> LemmyResult<Option<PaginationCursorNew>> {
        if let Some(item) = item {
          let data = item.to_cursor();
          let cursor = PaginationCursorNewInternal { data, back };
          Ok(Some(PaginationCursorNew::from_internal(cursor)?))
        } else {
          Ok(None)
        }
      };
      let prev_page = make_cursor(data.first(), true)?;
      let mut next_page = make_cursor(data.last(), false)?;

      // If there are less than limit items we are on the last page, dont show next button.
      // Need to convert here because diesel takes i64 for limit while vec length is usize.
      let limit: usize = limit.try_into().unwrap_or_default();
      if data.len() < limit {
        next_page = None;
      }
      Ok(PaginatedVec {
        data,
        next_page,
        prev_page,
      })
    }
  }
}

// ------------------------------
// TODO: below are all old and need to be removed

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
  fn test_cursor() -> LemmyResult<()> {
    let data = CursorData::new(1);
    do_test_cursor(data)?;

    let data = CursorData::new_multi([('A', 1), ('B', 2)]);
    do_test_cursor(data)?;

    Ok(())
  }

  fn do_test_cursor(data: CursorData) -> LemmyResult<()> {
    let cursor = PaginationCursorNewInternal {
      back: true,
      data: data.clone(),
    };
    let encoded = PaginationCursorNew::from_internal(cursor.clone())?;
    let cursor2 = encoded.to_internal()?;
    assert_eq!(cursor, cursor2);
    assert_eq!(data, cursor2.data);
    Ok(())
  }
}
