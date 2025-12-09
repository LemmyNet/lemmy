use crate::newtypes::CommunityId;
use diesel_uplete::UpleteCount;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::{connection::DbPool, dburl::DbUrl};
use lemmy_utils::{error::LemmyResult, settings::structs::Settings};
use std::future::Future;
use url::Url;

pub trait Followable: Sized {
  type Form;
  type IdType;
  fn follow(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    item_id: Self::IdType,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;
}

pub trait Likeable: Sized {
  type Form;
  type IdType;
  fn like(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;

  fn remove_all_likes(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;

  fn remove_likes_in_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;
}

pub trait Bannable: Sized {
  type Form;
  fn ban(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn unban(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;
}

pub trait Saveable: Sized {
  type Form;
  fn save(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn unsave(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;
}

pub trait Blockable: Sized {
  type Form;
  type ObjectIdType;
  type ObjectType;
  fn block(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn unblock(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<UpleteCount>> + Send;
  fn read_block(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    for_item_id: Self::ObjectIdType,
  ) -> impl Future<Output = LemmyResult<()>> + Send;

  fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    // Note: cant use lemmyresult because of try_pool
  ) -> impl Future<Output = LemmyResult<Vec<Self::ObjectType>>> + Send;
}

pub trait Reportable: Sized {
  type Form;
  type IdType;
  type ObjectIdType;
  fn report(
    pool: &mut DbPool<'_>,
    form: &Self::Form,
  ) -> impl Future<Output = LemmyResult<Self>> + Send;
  fn update_resolved(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    resolver_id: PersonId,
    is_resolved: bool,
  ) -> impl Future<Output = LemmyResult<usize>> + Send;
  fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send;
  fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    comment_id_: Self::ObjectIdType,
    by_resolver_id: PersonId,
  ) -> impl Future<Output = LemmyResult<usize>> + Send;
}

pub trait ApubActor: Sized {
  fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> impl Future<Output = LemmyResult<Option<Self>>> + Send;
  /// - actor_name is the name of the community or user to read.
  /// - domain if None only local actors are searched, if Some only actors from that domain
  /// - include_deleted, if true, will return communities or users that were deleted/removed
  fn read_from_name(
    pool: &mut DbPool<'_>,
    actor_name: &str,
    domain: Option<&str>,
    include_deleted: bool,
  ) -> impl Future<Output = LemmyResult<Option<Self>>> + Send;

  fn generate_local_actor_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl>;
  fn actor_url(&self, settings: &Settings) -> LemmyResult<Url>;
}

pub trait InternalToCombinedView {
  type CombinedView;

  /// Maps the combined DB row to an enum
  fn map_to_enum(self) -> Option<Self::CombinedView>;
}
