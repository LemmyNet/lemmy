use crate::{
  newtypes::{CommunityId, DbUrl, PaginationCursor, PersonId},
  utils::{get_conn, uplete, DbPool},
};
use diesel::{
  associations::HasTable,
  dsl,
  query_builder::{DeleteStatement, IntoUpdateTarget},
  query_dsl::methods::{FindDsl, LimitDsl},
  Table,
};
use diesel_async::{
  methods::{ExecuteDsl, LoadQuery},
  AsyncPgConnection,
  RunQueryDsl,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use std::future::Future;
use url::Url;

/// Returned by `diesel::delete`
pub type Delete<T> = DeleteStatement<<T as HasTable>::Table, <T as IntoUpdateTarget>::WhereClause>;

/// Returned by `Self::table().find(id)`
pub type Find<T> = dsl::Find<<T as HasTable>::Table, <T as Crud>::IdType>;

pub type PrimaryKey<T> = <<T as HasTable>::Table as Table>::PrimaryKey;

// Trying to create default implementations for `create` and `update` results in a lifetime mess and
// weird compile errors. https://github.com/rust-lang/rust/issues/102211
pub trait Crud: HasTable + Sized
where
  Self::Table: FindDsl<Self::IdType>,
  Find<Self>: LimitDsl + IntoUpdateTarget + Send,
  Delete<Find<Self>>: ExecuteDsl<AsyncPgConnection> + Send + 'static,

  // Used by `RunQueryDsl::first`
  dsl::Limit<Find<Self>>: LoadQuery<'static, AsyncPgConnection, Self> + Send + 'static,
{
  type InsertForm;
  type UpdateForm;
  type IdType: Send;

  fn create(
    pool: &mut DbPool<'_>,
    form: &Self::InsertForm,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;

  fn read(pool: &mut DbPool<'_>, id: Self::IdType) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Send,
  {
    async {
      let query: Find<Self> = Self::table().find(id);
      let conn = &mut *get_conn(pool).await?;
      query
        .first(conn)
        .await
        .with_lemmy_type(LemmyErrorType::NotFound)
    }
  }

  /// when you want to null out a column, you have to send Some(None)), since sending None means you
  /// just don't want to update that column.
  fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;

  fn delete(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<usize>> + Send {
    async {
      let query: Delete<Find<Self>> = diesel::delete(Self::table().find(id));
      let conn = &mut *get_conn(pool).await?;
      query
        .execute(conn)
        .await
        .with_lemmy_type(LemmyErrorType::Deleted)
    }
  }
}

pub trait Followable {
  type Form;
  type IdType;
  fn follow(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Joinable {
  type Form;
  fn join(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn leave(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Likeable {
  type Form;
  type IdType;
  fn like(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Bannable {
  type Form;
  fn ban(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn unban(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Saveable {
  type Form;
  fn save(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn unsave(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Readable {
  type Form;
  fn mark_as_read(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
  fn mark_many_as_read(
    pool: &mut DbPool<'_>,
    forms: &[Self::Form],
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
  fn mark_as_unread(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait ReadComments {
  type Form;
  type IdType;
  fn update_read_comments(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn remove_read_comments(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Hideable {
  type Form;
  fn hide(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn unhide(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
}

pub trait Blockable {
  type Form;
  type ObjectIdType;
  type ObjectType;
  fn block(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn unblock(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<uplete::Count>> + Send
  where
    Self: Sized;
  fn read_block(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_item_id: Self::ObjectIdType,
  ) -> impl Future<Output = LemmyResult<()>> + Send
  where
    Self: Sized;

  fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    // Note: cant use lemmyresult because of try_pool
  ) -> impl Future<Output = LemmyResult<Vec<Self::ObjectType>>> + Send
  where
    Self: Sized;
}

pub trait Reportable {
  type Form;
  type IdType;
  type ObjectIdType;
  fn report(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send
  where
    Self: Sized;
  fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
  fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
  fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    comment_id_: Self::ObjectIdType,
    by_resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
  fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send
  where
    Self: Sized;
}

pub trait ApubActor {
  fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> impl Future<Output = LemmyResult<Option<Self>>> + Send
  where
    Self: Sized;
  /// - actor_name is the name of the community or user to read.
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  fn read_from_name(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    include_deleted: bool,
  ) -> impl Future<Output = LemmyResult<Option<Self>>> + Send
  where
    Self: Sized;
  fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    protocol_domain: &str,
  ) -> impl Future<Output = LemmyResult<Option<Self>>> + Send
  where
    Self: Sized;

  fn generate_local_actor_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl>;
  fn actor_url(&self, settings: &Settings) -> LemmyResult<Url>;
}

pub trait InternalToCombinedView {
  type CombinedView;

  /// Maps the combined DB row to an enum
  fn map_to_enum(self) -> Option<Self::CombinedView>;
}

pub trait PaginationCursorBuilder {
  type CursorData;

  /// Builds a pagination cursor for the given query result.
  fn to_cursor(&self) -> PaginationCursor;

  /// Reads a database row from a given pagination cursor.
  fn from_cursor(
    cursor: &PaginationCursor,
    conn: &mut DbPool<'_>,
  ) -> impl Future<Output = LemmyResult<Self::CursorData>> + Send
  where
    Self: Sized;
}
