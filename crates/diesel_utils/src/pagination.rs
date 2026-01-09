use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
  ops::{Deref, DerefMut},
  sync::LazyLock,
};
#[cfg(feature = "full")]
use {
  crate::connection::DbPool,
  base64::{
    Engine,
    alphabet::Alphabet,
    engine::{GeneralPurpose, general_purpose::NO_PAD},
  },
  i_love_jesus::{PaginatedQueryBuilder, SortDirection},
  itertools::Itertools,
  lemmy_utils::error::LemmyErrorType,
  lemmy_utils::error::LemmyResult,
};

/// Use base 64 engine with custom alphabet based on base64::engine::general_purpose::URL_SAFE
/// with randomized character order, to prevent clients from parsing or modifying cursor data.
#[cfg(feature = "full")]
#[expect(clippy::expect_used)]
static BASE64_ENGINE: LazyLock<GeneralPurpose> = LazyLock::new(|| {
  let alphabet = Alphabet::new("AphruVFwvCetlckdZ2g-foxXBGNbyHnD96qUj3KL_YsE7P1OQiaIR0z4T58mMWJS")
    .expect("create base64 alphabet");
  GeneralPurpose::new(&alphabet, NO_PAD)
});

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CursorData(String);

#[cfg(feature = "full")]
impl CursorData {
  pub fn new_id(id: i32) -> Self {
    Self(id.to_string())
  }
  pub fn id(self) -> LemmyResult<i32> {
    Ok(self.0.parse()?)
  }

  pub fn new_with_prefix(prefix: char, id: i32) -> Self {
    Self(format!("{prefix},{id}"))
  }
  pub fn id_and_prefix(self) -> LemmyResult<(char, i32)> {
    let (prefix, id) = self
      .0
      .split_once(',')
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;
    let prefix = prefix
      .chars()
      .next()
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;
    Ok((prefix, id.parse()?))
  }

  pub fn new_plain(data: String) -> Self {
    Self(data)
  }
  pub fn plain(self) -> String {
    self.0
  }

  pub fn new_multi<const N: usize>(data: [i32; N]) -> Self {
    Self(data.into_iter().join(","))
  }
  pub fn multi<const N: usize>(self) -> LemmyResult<[i32; N]> {
    Ok(
      self
        .0
        .split(",")
        .flat_map(|id| id.parse::<i32>().ok())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_e| LemmyErrorType::CouldntParsePaginationToken)?,
    )
  }
}

#[cfg(feature = "full")]
pub trait PaginationCursorConversion {
  type PaginatedType: Send;

  fn to_cursor(&self) -> CursorData;

  fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> impl Future<Output = LemmyResult<Self::PaginatedType>> + Send;

  /// Paginate a db query.
  fn paginate<Q: Send>(
    query: Q,
    cursor: &Option<PaginationCursor>,
    sort_direction: SortDirection,
    pool: &mut DbPool<'_>,
    // this is only used by PostView for optimization
    page_before_or_equal: Option<Self::PaginatedType>,
  ) -> impl std::future::Future<Output = LemmyResult<PaginatedQueryBuilder<Self::PaginatedType, Q>>> + Send
  {
    async move {
      let (page_after, page_back, recovery) = if let Some(cursor) = cursor {
        let internal = cursor.clone().into_internal()?;
        let object = Self::from_cursor(internal.data, pool).await?;
        (Some(object), Some(internal.back), internal.recovery)
      } else {
        (None, None, false)
      };
      let mut query = PaginatedQueryBuilder::new(query, sort_direction);

      if page_back.unwrap_or_default() {
        if recovery {
          query = query.before_or_equal(page_after);
        } else {
          query = query.before(page_after);
        }
      } else if recovery {
        query = query.after_or_equal(page_after);
      } else {
        query = query.after(page_after);
      }

      if page_back.unwrap_or_default() {
        query = query
          .after_or_equal(page_before_or_equal)
          .limit_and_offset_from_end();
      } else {
        query = query.before_or_equal(page_before_or_equal);
      }

      Ok(query)
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
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PaginationCursor(String);

#[cfg(feature = "full")]
impl PaginationCursor {
  fn into_internal(self) -> LemmyResult<PaginationCursorInternal> {
    let decoded = BASE64_ENGINE.decode(self.0)?;
    Ok(serde_urlencoded::from_str(&String::from_utf8(decoded)?)?)
  }
  fn from_internal(other: PaginationCursorInternal) -> LemmyResult<Self> {
    let encoded = BASE64_ENGINE.encode(serde_urlencoded::to_string(other)?);
    Ok(Self(encoded))
  }

  // only used for PostView optimization
  pub fn is_back(self) -> LemmyResult<bool> {
    Ok(self.into_internal()?.back)
  }
}

/// The actual data which is stored inside a cursor, not accessible outside this file.
/// Uses serde rename to keep the cursor string short.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct PaginationCursorInternal {
  #[serde(rename = "b")]
  back: bool,
  #[serde(rename = "d")]
  data: CursorData,
  #[serde(rename = "r")]
  /// Allows to recover from empty pages without skipping an item by including the pointed to item
  /// in responses.
  recovery: bool,
}

/// This response contains only a single page of items. To get the next page, take the
/// cursor string from `next_page` and pass it to the same API endpoint via `page_cursor`
/// parameter. For going to the previous page, use `prev_page` instead.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PagedResponse<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T> {
  pub items: Vec<T>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[cfg(feature = "full")]
impl<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T> Deref
  for PagedResponse<T>
{
  type Target = Vec<T>;
  fn deref(&self) -> &Vec<T> {
    &self.items
  }
}

#[cfg(feature = "full")]
impl<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T> DerefMut
  for PagedResponse<T>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.items
  }
}

#[cfg(feature = "full")]
impl<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T> IntoIterator
  for PagedResponse<T>
{
  type Item = T;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  // Required method
  fn into_iter(self) -> Self::IntoIter {
    self.items.into_iter()
  }
}

/// Add prev/next cursors to query result.
#[cfg(feature = "full")]
// https://github.com/rust-lang/rust/issues/115590
#[allow(clippy::multiple_bound_locations)]
pub fn paginate_response<#[cfg(feature = "ts-rs")] T: ts_rs::TS, #[cfg(not(feature = "ts-rs"))] T>(
  data: Vec<T>,
  limit: i64,
  request_cursor: Option<PaginationCursor>,
) -> LemmyResult<PagedResponse<T>>
where
  T: PaginationCursorConversion + Serialize + for<'a> Deserialize<'a>,
{
  let make_cursor = |item: Option<&T>, back: bool| -> LemmyResult<Option<PaginationCursor>> {
    if let Some(item) = item {
      let data = item.to_cursor();
      let cursor = PaginationCursorInternal {
        data,
        back,
        recovery: false,
      };
      Ok(Some(PaginationCursor::from_internal(cursor)?))
    } else {
      Ok(None)
    }
  };
  let mut prev_page = make_cursor(data.first(), true)?;
  let mut next_page = make_cursor(data.last(), false)?;

  if let Ok(ref request_cursor) = request_cursor
    .map(PaginationCursor::into_internal)
    .transpose()
  {
    // Need to convert here because diesel takes i64 for limit while vec length is usize.
    let limit: usize = limit.try_into().unwrap_or_default();
    // Hide next and back buttons when possible.
    let back = request_cursor.as_ref().map(|r| r.back);
    match (data.len() < limit, back) {
      (false, None) => {
        prev_page = None; // no page before first
      }
      (true, None) => {
        prev_page = None; // no page before first
        next_page = None;
      }
      (true, Some(true)) => {
        prev_page = None;
      }
      (true, Some(false)) => {
        next_page = None;
      }
      (false, Some(_)) => {}
    };

    // When a page_cursor points to the very last or first item, the response for that cursor
    // contains no items and therefore ordinarily no cursors. Simply changing the direction of the
    // request_cursor would allow users to escape these empty pages, but would skip the item that
    // the cursor points to. Marking the cursor as recovery cursor allows to include this item, and
    // as long as the list remains unchanged, to recover at the start or end of the list. The
    // easiest way to reproduce this is to press next on the first page, then back twice.
    if data.is_empty()
      && let Some(PaginationCursorInternal {
        back,
        data,
        recovery: false,
      }) = request_cursor
    {
      if *back {
        next_page = Some(PaginationCursor::from_internal(PaginationCursorInternal {
          back: false,
          data: data.clone(),
          recovery: true,
        })?);
      } else {
        prev_page = Some(PaginationCursor::from_internal(PaginationCursorInternal {
          back: true,
          data: data.clone(),
          recovery: true,
        })?);
      }
    }
  }
  Ok(PagedResponse {
    items: data,
    next_page,
    prev_page,
  })
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_cursor() -> LemmyResult<()> {
    let data = CursorData::new_id(1);
    do_test_cursor(data)?;

    let data = CursorData::new_multi([1, 2]);
    do_test_cursor(data)?;

    Ok(())
  }

  fn do_test_cursor(data: CursorData) -> LemmyResult<()> {
    let cursor = PaginationCursorInternal {
      back: true,
      data: data.clone(),
      recovery: false,
    };
    let encoded = PaginationCursor::from_internal(cursor.clone())?;
    let cursor2 = encoded.into_internal()?;
    assert_eq!(cursor, cursor2);
    assert_eq!(data, cursor2.data);
    Ok(())
  }

  #[test]
  fn test_internal_format() -> LemmyResult<()> {
    assert_eq!(
      serde_urlencoded::to_string(PaginationCursorInternal {
        back: true,
        data: CursorData::new_plain("test".into()),
        recovery: false,
      })?,
      "b=true&d=test&r=false"
    );
    Ok(())
  }
}
