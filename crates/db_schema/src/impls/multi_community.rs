use crate::{
  diesel::{BoolExpressionMethods, OptionalExtension, PgExpressionMethods},
  newtypes::{CommunityId, DbUrl, MultiCommunityId, PersonId},
  source::{
    community::Community,
    multi_community::{
      MultiCommunity,
      MultiCommunityFollow,
      MultiCommunityFollowForm,
      MultiCommunityInsertForm,
      MultiCommunityUpdateForm,
    },
  },
  traits::Crud,
  utils::{functions::lower, get_conn, DbPool},
};
use diesel::{
  dsl::{count, delete, exists, insert_into},
  select,
  update,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  community,
  multi_community,
  multi_community_entry,
  multi_community_follow,
  person,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

const MULTI_COMMUNITY_ENTRY_LIMIT: i8 = 50;

impl Crud for MultiCommunity {
  type InsertForm = MultiCommunityInsertForm;
  type UpdateForm = MultiCommunityUpdateForm;
  type IdType = MultiCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .get_result(conn)
        .await?,
    )
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    form: &MultiCommunityUpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      update(multi_community::table.find(id))
        .set(form)
        .get_result(conn)
        .await?,
    )
  }
}

impl MultiCommunity {
  pub async fn read_from_name(pool: &mut DbPool<'_>, multi_name: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      multi_community::table
        .filter(multi_community::local.eq(true))
        .filter(multi_community::deleted.eq(false))
        .filter(lower(multi_community::name).eq(multi_name.to_lowercase()))
        .first(conn)
        .await?,
    )
  }

  pub async fn read_from_ap_id(pool: &mut DbPool<'_>, ap_id: &DbUrl) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    multi_community::table
      .filter(multi_community::ap_id.eq(ap_id))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn create_entry(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_community: &Community,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let count: i64 = multi_community::table
      .left_join(multi_community_entry::table)
      .filter(multi_community::id.eq(id))
      .select(count(multi_community_entry::community_id.nullable()))
      .first(conn)
      .await?;
    if count >= MULTI_COMMUNITY_ENTRY_LIMIT.into() {
      return Err(LemmyErrorType::MultiCommunityEntryLimitReached.into());
    }

    insert_into(multi_community_entry::table)
      .values((
        multi_community_entry::multi_community_id.eq(id),
        multi_community_entry::community_id.eq(new_community.id),
      ))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn delete_entry(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    old_community: &Community,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(
      multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.eq(id))
        .filter(multi_community_entry::community_id.eq(old_community.id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }

  pub async fn list(
    pool: &mut DbPool<'_>,
    owner_id: Option<PersonId>,
    followed_by: Option<PersonId>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table
      .left_join(multi_community_follow::table)
      .select(multi_community::all_columns)
      .into_boxed();
    if let Some(owner_id) = owner_id {
      query = query.filter(multi_community::creator_id.eq(owner_id));
    }
    if let Some(followed_by) = followed_by {
      query = query.filter(multi_community_follow::person_id.eq(followed_by));
    }
    Ok(query.get_results(conn).await?)
  }

  pub async fn follow(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityFollowForm,
  ) -> LemmyResult<MultiCommunityFollow> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(multi_community_follow::table)
        .values(form)
        .on_conflict((
          multi_community_follow::multi_community_id,
          multi_community_follow::person_id,
        ))
        .do_update()
        .set(form)
        .get_result(conn)
        .await?,
    )
  }

  pub async fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    multi_community_id: MultiCommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(
      multi_community_follow::table
        .filter(multi_community_follow::multi_community_id.eq(multi_community_id))
        .filter(multi_community_follow::person_id.eq(person_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }

  pub async fn follower_inboxes(
    pool: &mut DbPool<'_>,
    multi_community_id: MultiCommunityId,
  ) -> LemmyResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;
    multi_community_follow::table
      .inner_join(person::table)
      .filter(multi_community_follow::multi_community_id.eq(multi_community_id))
      .select(person::inbox_url)
      .distinct()
      .load(conn)
      .await
      .optional()?
      .ok_or(LemmyErrorType::NotFound.into())
  }

  pub async fn upsert(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityInsertForm,
  ) -> LemmyResult<MultiCommunity> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .on_conflict(multi_community::ap_id)
        .do_update()
        .set(form)
        .get_result(conn)
        .await?,
    )
  }

  /// Should be called in a transaction together with update() or upsert()
  pub async fn update_entries(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> LemmyResult<(Vec<Community>, Vec<Community>, bool)> {
    let conn = &mut get_conn(pool).await?;
    if new_communities.len() >= usize::try_from(MULTI_COMMUNITY_ENTRY_LIMIT)? {
      return Err(LemmyErrorType::MultiCommunityEntryLimitReached.into());
    }

    let removed: Vec<CommunityId> = delete(
      multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.eq(id))
        .filter(multi_community_entry::community_id.ne_all(new_communities)),
    )
    .returning(multi_community_entry::community_id)
    .get_results::<CommunityId>(conn)
    .await?;
    let removed: Vec<Community> = community::table
      .filter(community::id.eq_any(removed))
      .get_results(conn)
      .await?;

    let forms = new_communities
      .iter()
      .map(|k| {
        (
          multi_community_entry::multi_community_id.eq(id),
          multi_community_entry::community_id.eq(k),
        )
      })
      .collect::<Vec<_>>();
    let added: Vec<_> = insert_into(multi_community_entry::table)
      .values(forms)
      .on_conflict_do_nothing()
      .returning(multi_community_entry::community_id)
      .get_results::<CommunityId>(conn)
      .await?;
    let added: Vec<Community> = community::table
      .filter(community::id.eq_any(added))
      .get_results(conn)
      .await?;

    // get all local users who follow the multi-comm
    let has_local_followers: bool = select(exists(
      multi_community_follow::table
        .inner_join(person::table)
        .inner_join(multi_community::table)
        .filter(person::local),
    ))
    .get_result(conn)
    .await?;

    Ok((added, removed, has_local_followers))
  }

  pub async fn read_entry_ap_ids(
    pool: &mut DbPool<'_>,
    multi_name: &str,
  ) -> LemmyResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;
    let entries = multi_community::table
      .inner_join(multi_community_entry::table.inner_join(community::table))
      .left_join(person::table)
      .filter(
        community::removed
          .or(community::deleted)
          .is_distinct_from(true),
      )
      .filter(person::local)
      .filter(multi_community::name.eq(multi_name))
      .select(community::ap_id)
      .get_results(conn)
      .await?;
    Ok(entries)
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use crate::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      multi_community::{MultiCommunity, MultiCommunityInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_db_schema_file::enums::CommunityFollowerState;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use std::collections::HashSet;

  struct Data {
    multi: MultiCommunity,
    instance: Instance,
    person: Person,
    community: Community,
  }

  async fn setup(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let form = PersonInsertForm::test_form(instance.id, "bobby");
    let person = Person::create(pool, &form).await?;

    let form = CommunityInsertForm::new(
      instance.id,
      "TIL".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &form).await?;

    let form =
      MultiCommunityInsertForm::new(person.id, instance.id, "multi".to_string(), String::new());
    let multi = MultiCommunity::create(pool, &form).await?;
    assert_eq!(form.creator_id, multi.creator_id);
    assert_eq!(form.name, multi.name);

    Ok(Data {
      multi,
      instance,
      person,
      community,
    })
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_community_apub() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    let multi_read_apub_empty = MultiCommunity::read_entry_ap_ids(pool, &data.multi.name).await?;
    assert!(multi_read_apub_empty.is_empty());

    let multi_entries = vec![data.community.id];
    MultiCommunity::update_entries(pool, data.multi.id, &multi_entries).await?;

    let multi_read_apub = MultiCommunity::read_entry_ap_ids(pool, &data.multi.name).await?;
    assert_eq!(vec![data.community.ap_id], multi_read_apub);

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_community_list() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    let form = PersonInsertForm::test_form(data.instance.id, "tom");
    let person2 = Person::create(pool, &form).await?;

    let form = MultiCommunityInsertForm::new(
      person2.id,
      person2.instance_id,
      "multi2".to_string(),
      String::new(),
    );
    let multi2 = MultiCommunity::create(pool, &form).await?;

    // list all multis
    let list_all = MultiCommunity::list(pool, None, None)
      .await?
      .iter()
      .map(|m| m.id)
      .collect::<HashSet<_>>();
    assert_eq!(list_all, HashSet::from([data.multi.id, multi2.id]));

    // list multis by owner
    let list_owner = MultiCommunity::list(pool, Some(data.person.id), None).await?;
    assert_eq!(list_owner.len(), 1);
    assert_eq!(list_owner[0].id, data.multi.id);

    // list multis followed by user
    let form = MultiCommunityFollowForm {
      multi_community_id: multi2.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;
    let list_followed = MultiCommunity::list(pool, None, Some(data.person.id)).await?;
    assert_eq!(list_followed.len(), 1);
    assert_eq!(list_followed[0].id, multi2.id);

    MultiCommunity::unfollow(pool, data.person.id, multi2.id).await?;
    let list_followed = MultiCommunity::list(pool, None, Some(data.person.id)).await?;
    assert_eq!(list_followed.len(), 0);

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }
}
