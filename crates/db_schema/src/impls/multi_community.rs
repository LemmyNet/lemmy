use crate::{
  diesel::{BoolExpressionMethods, PgExpressionMethods},
  newtypes::{CommunityId, MultiCommunityId, PersonId},
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
  utils::{get_conn, DbConn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{delete, insert_into, sql},
  pg::sql_types::Array,
  sql_types::Text,
  update,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{community, multi_community, multi_community_entry, multi_community_follow, person},
};
use lemmy_utils::error::LemmyResult;

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
  pub async fn upsert(conn: &mut DbConn<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .on_conflict(multi_community::ap_id)
        .do_update()
        .set(form)
        .get_result::<Self>(conn)
        .await?,
    )
  }

  pub async fn create_entry(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_community: &Community,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
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

  /// Should be called in a transaction together with update() or upsert()
  pub async fn update_entries(
    conn: &mut DbConn<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> LemmyResult<()> {
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

  pub async fn list(
    pool: &mut DbPool<'_>,
    owner_id: Option<PersonId>,
    followed_by: Option<PersonId>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table
      .inner_join(multi_community_follow::table)
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
    multi_community_id: MultiCommunityId,
    person_id: PersonId,
  ) -> LemmyResult<MultiCommunityFollow> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      delete(
        multi_community_follow::table
          .filter(multi_community_follow::multi_community_id.eq(multi_community_id))
          .filter(multi_community_follow::person_id.eq(person_id)),
      )
      .get_result(conn)
      .await?,
    )
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
    let local_follows: Vec<PersonId> = multi_community_follow::table
      .inner_join(person::table)
      .filter(multi_community_follow::multi_community_id.eq(multi_community_id))
      .filter(person::local)
      .select(person::id)
      .get_results(conn)
      .await?;

    for person_id in local_follows {
      if is_removed_from_multi {
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
        CommunityActions::follow(pool, &form).await?;
      } else {
        CommunityActions::unfollow(pool, person_id, community.id).await?;
      }
    }
    Ok(())
  }
}

impl MultiCommunityApub {
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

  #[tokio::test]
  #[serial]
  async fn test_multi_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let form = PersonInsertForm::test_form(instance.id, "bobby");
    let bobby = Person::create(pool, &form).await?;

    let form = CommunityInsertForm::new(
      instance.id,
      "TIL".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &form).await?;

    let form =
      MultiCommunityInsertForm::new(bobby.id, "multi".to_string(), community.ap_id.clone());
    let multi_create = MultiCommunity::create(pool, &form).await?;
    assert_eq!(form.creator_id, multi_create.creator_id);
    assert_eq!(form.name, multi_create.name);
    assert_eq!(form.ap_id, multi_create.ap_id);

    let multi_read_apub_empty = MultiCommunityApub::read_local(pool, &multi_create.name).await?;
    assert!(multi_read_apub_empty.entries.is_empty());

    let multi_entries = vec![community.id];
    let conn = &mut get_conn(pool).await?;
    MultiCommunity::update_entries(conn, multi_create.id, &multi_entries).await?;

    let multi_read_apub = MultiCommunityApub::read_local(pool, &multi_create.name).await?;
    assert_eq!(multi_read_apub.multi.creator_id, multi_create.creator_id);
    assert_eq!(vec![community.ap_id], multi_read_apub.entries);

    //let list = MultiCommunity::list(pool, None).await?;
    //assert_eq!(1, list.len());
    // TODO: test follow methods, test list(followed_by)
    todo!();

    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
