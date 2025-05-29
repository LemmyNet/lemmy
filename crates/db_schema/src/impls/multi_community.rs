use crate::{
  diesel::{BoolExpressionMethods, OptionalExtension, PgExpressionMethods},
  newtypes::{CommunityId, DbUrl, MultiCommunityId, PersonId},
  source::{
    community::{Community, CommunityActions, CommunityFollowerForm},
    multi_community::{
      MultiCommunity,
      MultiCommunityApub,
      MultiCommunityFollow,
      MultiCommunityFollowForm,
      MultiCommunityInsertForm,
      MultiCommunityUpdateForm,
    },
  },
  traits::{Crud, Followable},
  utils::{functions::lower, get_conn, uplete, DbConn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{count, delete, exists, insert_into, sql},
  pg::sql_types::Array,
  select,
  sql_types::Text,
  update,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{
    community,
    community_actions,
    multi_community,
    multi_community_entry,
    multi_community_follow,
    person,
  },
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
    Self::update_local_follows(pool, id, new_community, false).await?;
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
    Self::update_local_follows(pool, id, old_community, true).await?;
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

  pub async fn update_local_follows(
    pool: &mut DbPool<'_>,
    multi_community_id: MultiCommunityId,
    community: &Community,
    is_removed_from_multi: bool,
  ) -> LemmyResult<()> {
    if !community.local {
      return Ok(());
    }

    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community_follow::table
      .inner_join(
        person::table.left_join(
          community_actions::table.on(
            person::id
              .eq(community_actions::person_id)
              .and(community_actions::community_id.eq(community.id)),
          ),
        ),
      )
      .filter(multi_community_follow::multi_community_id.eq(multi_community_id))
      .filter(person::local)
      .select(person::id)
      .into_boxed();
    if is_removed_from_multi {
      // Remove: only in case it was a multi-comm follow (not manually by user)
      query = query.filter(community_actions::is_multi_community_follow.is_not_null());
    } else {
      // Add: only commns which the user isnt following already
      query = query.filter(community_actions::followed.is_null());
    }
    let local_follows: Vec<PersonId> = query.get_results(conn).await?;

    for person_id in local_follows {
      if !is_removed_from_multi {
        let follow_state = if community.visibility == CommunityVisibility::Private {
          CommunityFollowerState::ApprovalRequired
        } else {
          CommunityFollowerState::Accepted
        };
        let form = CommunityFollowerForm {
          community_id: community.id,
          person_id,
          follow_state,
          follow_approver_id: None,
          followed: Utc::now(),
          is_multi_community_follow: Some(true),
        };
        CommunityActions::follow(&mut DbPool::Conn(conn), &form).await?;
      } else {
        Self::delete_follow_in_multi_comm(conn, multi_community_id, person_id, community.id)
          .await?;
      }
    }
    Ok(())
  }

  /// Unlike CommunityActions::unfollow this checks `is_multi_community_follow`
  async fn delete_follow_in_multi_comm(
    conn: &mut DbConn<'_>,
    multi_community_id: MultiCommunityId,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<()> {
    // True if the user also follows this community via other multi-comms, in this case dont delete
    let community_has_other_multi_follows = select(exists(
      multi_community::table
        .inner_join(multi_community_follow::table)
        .inner_join(
          multi_community_entry::table.inner_join(
            community_actions::table
              .on(multi_community_entry::community_id.eq(community_actions::community_id)),
          ),
        )
        .filter(multi_community::id.ne(multi_community_id))
        .filter(multi_community_follow::person_id.eq(person_id))
        .filter(multi_community_entry::community_id.eq(community_id))
        .filter(community_actions::is_multi_community_follow.eq(true)),
    ))
    .get_result::<bool>(conn)
    .await?;

    if !community_has_other_multi_follows {
      // delete the community follow
      uplete::new(community_actions::table.find((person_id, community_id)))
        .set_null(community_actions::followed)
        .set_null(community_actions::follow_state)
        .set_null(community_actions::follow_approver_id)
        .set_null(community_actions::is_multi_community_follow)
        .get_result::<uplete::Count>(conn)
        .await?;
    }
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
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl MultiCommunityApub {
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
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    if new_communities.len() >= usize::try_from(MULTI_COMMUNITY_ENTRY_LIMIT)? {
      return Err(LemmyErrorType::MultiCommunityEntryLimitReached.into());
    }

    delete(multi_community_entry::table)
      .filter(multi_community_entry::multi_community_id.eq(id))
      .filter(multi_community_entry::community_id.ne_all(new_communities))
      .execute(conn)
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
    insert_into(multi_community_entry::table)
      .values(forms)
      .on_conflict_do_nothing()
      .execute(conn)
      .await?;
    Ok(())
  }

  // TODO: not needed?
  pub async fn read_local(
    pool: &mut DbPool<'_>,
    multi_name: &str,
  ) -> LemmyResult<MultiCommunityApub> {
    let conn = &mut get_conn(pool).await?;
    let (multi, entries) = multi_community::table
      .left_join(person::table)
      .left_join(multi_community_entry::table.inner_join(community::table))
      .group_by(multi_community::id)
      .filter(
        community::removed
          .or(community::deleted)
          .is_distinct_from(true),
      )
      .filter(person::local)
      .filter(multi_community::name.eq(multi_name))
      .select((
        multi_community::all_columns,
        // Get vec of community.ap_id. If no row exists for multi_community_entry this returns
        // [null] so we need to filter that with array_remove.
        sql::<Array<Text>>("array_remove(array_agg(community.ap_id), null)"),
      ))
      .first(conn)
      .await?;
    Ok(MultiCommunityApub { multi, entries })
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

    let form = MultiCommunityInsertForm::new(person.id, instance.id, "multi".to_string());
    let multi = MultiCommunity::create(pool, &form).await?;
    assert_eq!(form.creator_id, multi.creator_id);
    assert_eq!(form.name, multi.name);
    assert_eq!(form.ap_id.as_ref(), Some(&multi.ap_id));

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

    let multi_read_apub_empty = MultiCommunityApub::read_local(pool, &data.multi.name).await?;
    assert!(multi_read_apub_empty.entries.is_empty());

    let multi_entries = vec![data.community.id];
    MultiCommunityApub::update_entries(pool, data.multi.id, &multi_entries).await?;

    let multi_read_apub = MultiCommunityApub::read_local(pool, &data.multi.name).await?;
    assert_eq!(multi_read_apub.multi.creator_id, data.multi.creator_id);
    assert_eq!(vec![data.community.ap_id], multi_read_apub.entries);

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_community_follow() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    let form = MultiCommunityFollowForm {
      multi_community_id: data.multi.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;

    // add community to multi-comm
    MultiCommunity::create_entry(pool, data.multi.id, &data.community).await?;

    // user should be following the community now
    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await?;
    assert_eq!(actions.is_multi_community_follow, Some(true));

    // remove community from multi-comm
    MultiCommunity::delete_entry(pool, data.multi.id, &data.community).await?;

    // follow is also removed
    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await;
    assert!(actions.is_err());

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

    let form = MultiCommunityInsertForm::new(person2.id, person2.instance_id, "multi2".to_string());
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

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_overlapping_multi_follows() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    // create another multi
    let form = MultiCommunityInsertForm::new(
      data.person.id,
      data.person.instance_id,
      "multi2".to_string(),
    );
    let multi2 = MultiCommunity::create(pool, &form).await?;

    // follow both of them
    let form = MultiCommunityFollowForm {
      multi_community_id: data.multi.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;
    let form = MultiCommunityFollowForm {
      multi_community_id: multi2.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;

    // add the same community to both multis
    MultiCommunity::create_entry(pool, data.multi.id, &data.community).await?;
    MultiCommunity::create_entry(pool, multi2.id, &data.community).await?;

    // user should be following community
    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await?;
    assert_eq!(actions.is_multi_community_follow, Some(true));

    // delete entry from one multi, user should still follow community
    MultiCommunity::delete_entry(pool, data.multi.id, &data.community).await?;
    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await?;
    assert_eq!(actions.is_multi_community_follow, Some(true));

    // delete entry from one multi, user should not follow community anymore
    MultiCommunity::delete_entry(pool, multi2.id, &data.community).await?;
    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await;
    assert!(actions.is_err());

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_with_manual_follow() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = setup(pool).await?;

    let form = CommunityFollowerForm::new(
      data.community.id,
      data.person.id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &form).await?;

    let form = MultiCommunityFollowForm {
      multi_community_id: data.multi.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;
    let form = MultiCommunityFollowForm {
      multi_community_id: data.multi.id,
      person_id: data.person.id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &form).await?;

    MultiCommunity::create_entry(pool, data.multi.id, &data.community).await?;

    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await?;
    assert!(actions.followed.is_some());
    assert_eq!(actions.is_multi_community_follow, None);

    MultiCommunity::delete_entry(pool, data.multi.id, &data.community).await?;

    let actions = CommunityActions::read(pool, data.community.id, data.person.id).await?;
    assert!(actions.followed.is_some());
    assert_eq!(actions.is_multi_community_follow, None);

    Instance::delete(pool, data.instance.id).await?;
    Ok(())
  }
}
