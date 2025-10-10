use crate::{
  diesel::{BoolExpressionMethods, OptionalExtension, PgExpressionMethods, SelectableHelper},
  newtypes::{CommunityId, DbUrl, MultiCommunityId, PersonId},
  source::{
    community::Community,
    multi_community::{
      MultiCommunity,
      MultiCommunityEntry,
      MultiCommunityEntryForm,
      MultiCommunityFollow,
      MultiCommunityFollowForm,
      MultiCommunityInsertForm,
      MultiCommunityUpdateForm,
    },
  },
  traits::{ApubActor, Crud},
  utils::{format_actor_url, functions::lower, get_conn, DbPool},
};
use diesel::{
  dsl::{delete, exists, insert_into, not},
  select,
  update,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  community,
  instance,
  multi_community,
  multi_community_entry,
  multi_community_follow,
  person,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

const MULTI_COMMUNITY_ENTRY_LIMIT: i8 = 50;

impl Crud for MultiCommunity {
  type InsertForm = MultiCommunityInsertForm;
  type UpdateForm = MultiCommunityUpdateForm;
  type IdType = MultiCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(multi_community::table)
      .values(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(multi_community::table.find(id))
      .set(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl MultiCommunity {
  pub async fn upsert(pool: &mut DbPool<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(multi_community::table)
      .values(form)
      .on_conflict(multi_community::ap_id)
      .do_update()
      .set(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn follow(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityFollowForm,
  ) -> LemmyResult<MultiCommunityFollow> {
    let conn = &mut get_conn(pool).await?;

    insert_into(multi_community_follow::table)
      .values(form)
      .on_conflict((
        multi_community_follow::multi_community_id,
        multi_community_follow::person_id,
      ))
      .do_update()
      .set(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      .get_results(conn)
      .await
      .optional()?
      .ok_or(LemmyErrorType::NotFound.into())
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
      .filter(not(community::local))
      .get_results(conn)
      .await?;

    let forms = new_communities
      .iter()
      .map(|community_id| MultiCommunityEntryForm {
        multi_community_id: id,
        community_id: *community_id,
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
      .filter(not(community::local))
      .get_results(conn)
      .await?;

    // check if any local user follows the multi-comm
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

  pub async fn read_community_ap_ids(
    pool: &mut DbPool<'_>,
    multi_name: &str,
  ) -> LemmyResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;

    multi_community::table
      .inner_join(multi_community_entry::table.inner_join(community::table))
      .filter(
        community::removed
          .or(community::deleted)
          .is_distinct_from(true),
      )
      .filter(multi_community::name.eq(multi_name))
      .select(community::ap_id)
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl ApubActor for MultiCommunity {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    multi_community::table
      .filter(lower(multi_community::ap_id).eq(object_id.to_lowercase()))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    name: &str,
    domain: Option<&str>,
    include_deleted: bool,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut q = multi_community::table
      .inner_join(instance::table)
      .filter(lower(multi_community::name).eq(name.to_lowercase()))
      .select(MultiCommunity::as_select())
      .into_boxed();
    if !include_deleted {
      q = q.filter(multi_community::deleted.eq(false))
    }
    if let Some(domain) = domain {
      q = q.filter(lower(instance::domain).eq(domain.to_lowercase()))
    } else {
      q = q.filter(multi_community::local.eq(true))
    }
    q.first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  fn actor_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let domain = self
      .ap_id
      .inner()
      .domain()
      .ok_or(LemmyErrorType::NotFound)?;

    format_actor_url(&self.name, domain, 'm', settings)
  }

  fn generate_local_actor_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/m/{name}"))?.into())
  }
}

impl MultiCommunityEntry {
  pub async fn create(pool: &mut DbPool<'_>, form: &MultiCommunityEntryForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(multi_community_entry::table)
      .values(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn delete(pool: &mut DbPool<'_>, form: &MultiCommunityEntryForm) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    delete(
      multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.eq(form.multi_community_id))
        .filter(multi_community_entry::community_id.eq(form.community_id)),
    )
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::Deleted)
  }

  /// Make sure you aren't trying to insert more communities than the entry limit allows.
  pub async fn check_entry_limit(
    pool: &mut DbPool<'_>,
    multi_community_id: MultiCommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    let count: i64 = multi_community_entry::table
      .filter(multi_community_entry::multi_community_id.eq(multi_community_id))
      .count()
      .get_result(conn)
      .await?;

    if count >= MULTI_COMMUNITY_ENTRY_LIMIT.into() {
      Err(LemmyErrorType::MultiCommunityEntryLimitReached.into())
    } else {
      Ok(())
    }
  }

  pub async fn community_used_in_multiple(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityEntryForm,
  ) -> LemmyResult<bool> {
    let conn = &mut get_conn(pool).await?;

    select(exists(
      multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.ne(form.multi_community_id))
        .filter(multi_community_entry::community_id.eq(form.community_id)),
    ))
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
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
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    multi: MultiCommunity,
    instance: Instance,
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
      community,
    })
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_community_apub() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    let multi_read_apub_empty =
      MultiCommunity::read_community_ap_ids(pool, &data.multi.name).await?;
    assert!(multi_read_apub_empty.is_empty());

    let multi_entries = vec![data.community.id];
    MultiCommunity::update_entries(pool, data.multi.id, &multi_entries).await?;

    let multi_read_apub = MultiCommunity::read_community_ap_ids(pool, &data.multi.name).await?;
    assert_eq!(vec![data.community.ap_id], multi_read_apub);

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }
}
