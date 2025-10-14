use crate::PendingFollowerView;
use diesel::{
  dsl::{count, exists, sql},
  pg::sql_types::Array,
  select,
  sql_types::Integer,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  aliases,
  newtypes::{CommunityId, InstanceId, PaginationCursor, PersonId},
  source::{
    community::{community_actions_keys as key, Community, CommunityActions},
    person::Person,
  },
  traits::PaginationCursorBuilder,
  utils::{get_conn, limit_fetch, paginate, queries::selects::person1_select, DbPool},
};
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{community, community_actions, person},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::collections::HashMap;

diesel::alias!(community_actions as follower_community_actions: FollowerCommunityActions,
person as person_instance_check: PersonInstanceCheck,
community_actions as community_actions_instance_check: CommunityActionInstanceCheck);

impl PendingFollowerView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let follower_community_actions_join = follower_community_actions
      .on(community::id.eq(follower_community_actions.field(community_actions::community_id)));
    let follower_id = aliases::person1.field(person::id);
    let follower_join = aliases::person1.on(
      follower_community_actions
        .field(community_actions::person_id)
        .eq(follower_id)
        .and(
          follower_community_actions
            .field(community_actions::followed_at)
            .is_not_null(),
        )
        .and(community::id.eq(follower_community_actions.field(community_actions::community_id))),
    );
    let person_join = person::table.on(community_actions::person_id.eq(person::id));

    community_actions::table
      .inner_join(community::table)
      .inner_join(person_join)
      .inner_join(follower_community_actions_join)
      .inner_join(follower_join)
  }

  pub async fn list_approval_required(
    pool: &mut DbPool<'_>,
    mod_id: PersonId,
    all_communities: bool,
    unread_only: bool,
    cursor_data: Option<CommunityActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<PendingFollowerView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let mut query = Self::joins()
      .filter(community_actions::became_moderator_at.is_not_null())
      .filter(community::visibility.eq(CommunityVisibility::Private))
      .select((
        person1_select(),
        community::all_columns,
        follower_community_actions
          .field(community_actions::follow_state)
          .nullable(),
      ))
      .limit(limit)
      .into_boxed();

    // if param is false, only return items for communities where user is a mod
    if !all_communities {
      query = query.filter(person::id.eq(mod_id));
    }

    if unread_only {
      query = query.filter(
        follower_community_actions
          .field(community_actions::follow_state)
          .eq(CommunityFollowerState::ApprovalRequired),
      );
    }

    // Sorting by published
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::followed_at);

    let mut res: Vec<_> = paginated_query
      .load::<(Person, Community, Option<CommunityFollowerState>)>(conn)
      .await?
      .into_iter()
      .map(|(person, community, follow_state)| PendingFollowerView {
        person,
        community,
        is_new_instance: true,
        follow_state,
      })
      .collect();

    // For all returned communities, get the list of approved follower instances
    // TODO: This should be merged into the main query above as a subquery
    let community_ids: Vec<_> = res.iter().map(|r| r.community.id).collect();
    let approved_follower_instances: HashMap<_, _> = community_actions::table
      .inner_join(person::table.on(community_actions::person_id.eq(person::id)))
      .filter(community_actions::community_id.eq_any(community_ids))
      .filter(community_actions::follow_state.eq(CommunityFollowerState::Accepted))
      .group_by(community_actions::community_id)
      .select((
        community_actions::community_id,
        sql::<Array<Integer>>("array_agg(distinct person.instance_id) instance_ids"),
      ))
      .load::<(CommunityId, Vec<InstanceId>)>(conn)
      .await?
      .into_iter()
      .collect();

    // Check if there is already an approved follower from the same instance. If not, frontends
    // should show a warning because a malicious admin could leak private community data.
    for r in &mut res {
      let instance_ids = approved_follower_instances.get(&r.community.id);
      if let Some(instance_ids) = instance_ids {
        if instance_ids.contains(&r.person.instance_id) {
          r.is_new_instance = false;
        }
      }
    }
    Ok(res)
  }

  pub async fn count_approval_required(
    pool: &mut DbPool<'_>,
    mod_id: PersonId,
  ) -> LemmyResult<i64> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::became_moderator_at.is_not_null())
      .filter(community::visibility.eq(CommunityVisibility::Private))
      .filter(person::id.eq(mod_id))
      .filter(
        follower_community_actions
          .field(community_actions::follow_state)
          .eq(CommunityFollowerState::ApprovalRequired),
      )
      .select(count(community_actions::community_id))
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub async fn check_private_community_action(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    community: &Community,
  ) -> LemmyResult<()> {
    if community.visibility != CommunityVisibility::Private {
      return Ok(());
    }
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins()
        .filter(community_actions::community_id.eq(community.id))
        .filter(community_actions::person_id.eq(from_person_id))
        .filter(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
  }
  pub async fn check_has_followers_from_instance(
    community_id: CommunityId,
    instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins()
        .filter(community::visibility.eq(CommunityVisibility::Private))
        .filter(community_actions::community_id.eq(community_id))
        .filter(aliases::person1.field(person::instance_id).eq(instance_id))
        .filter(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
  }
}

impl PaginationCursorBuilder for PendingFollowerView {
  type CursorData = CommunityActions;

  fn to_cursor(&self) -> PaginationCursor {
    // This needs a person and community
    let prefixes_and_ids = [('P', self.person.id.0), ('C', self.community.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }
  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let [(_, person_id), (_, community_id)] = cursor.prefixes_and_ids()?;
    CommunityActions::read(pool, CommunityId(community_id), PersonId(person_id)).await
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use crate::PendingFollowerView;
  use lemmy_db_schema::{
    assert_length,
    source::{
      community::{
        CommunityActions,
        CommunityFollowerForm,
        CommunityInsertForm,
        CommunityModeratorForm,
      },
      instance::Instance,
      person::PersonInsertForm,
    },
    traits::{Crud, Followable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_db_schema_file::enums::CommunityVisibility;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_has_followers_from_instance() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // insert local community
    let local_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let community_form = CommunityInsertForm {
      visibility: Some(CommunityVisibility::Private),
      ..CommunityInsertForm::new(
        local_instance.id,
        "test_community_3".to_string(),
        "nada".to_owned(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;

    // insert remote user
    let remote_instance = Instance::read_or_create(pool, "other_domain.tld".to_string()).await?;
    let person_form =
      PersonInsertForm::new("name".to_string(), "pubkey".to_string(), remote_instance.id);
    let person = Person::create(pool, &person_form).await?;

    // community has no follower from remote instance, returns error
    let has_followers = PendingFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_err());

    // insert unapproved follower
    let mut follower_form = CommunityFollowerForm::new(
      community.id,
      person.id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &follower_form).await?;

    // still returns error
    let has_followers = PendingFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_err());

    // mark follower as accepted
    follower_form.follow_state = CommunityFollowerState::Accepted;
    CommunityActions::follow(pool, &follower_form).await?;

    // now returns ok
    let has_followers = PendingFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_ok());

    Instance::delete(pool, local_instance.id).await?;
    Instance::delete(pool, remote_instance.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_pending_followers() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // insert local community
    let local_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let community_form = CommunityInsertForm {
      visibility: Some(CommunityVisibility::Private),
      ..CommunityInsertForm::new(
        local_instance.id,
        "test_community_3".to_string(),
        "nada".to_owned(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;

    // insert local mod
    let mod_form =
      PersonInsertForm::new("name".to_string(), "pubkey".to_string(), local_instance.id);
    let mod_ = Person::create(pool, &mod_form).await?;

    let moderator_form = CommunityModeratorForm::new(community.id, mod_.id);
    CommunityActions::join(pool, &moderator_form).await?;

    // insert remote user
    let remote_instance = Instance::read_or_create(pool, "other_domain.tld".to_string()).await?;
    let person_form =
      PersonInsertForm::new("name".to_string(), "pubkey".to_string(), remote_instance.id);
    let person = Person::create(pool, &person_form).await?;

    // check that counts are initially 0
    let count = PendingFollowerView::count_approval_required(pool, mod_.id).await?;
    assert_eq!(0, count);
    let list =
      PendingFollowerView::list_approval_required(pool, mod_.id, false, true, None, None, None)
        .await?;
    assert_length!(0, list);

    // user is not allowed to post
    let posting_allowed =
      PendingFollowerView::check_private_community_action(pool, person.id, &community).await;
    assert!(posting_allowed.is_err());

    // send follow request
    let follower_form = CommunityFollowerForm::new(
      community.id,
      person.id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &follower_form).await?;

    // now there should be a pending follow
    let count = PendingFollowerView::count_approval_required(pool, mod_.id).await?;
    assert_eq!(1, count);
    let list =
      PendingFollowerView::list_approval_required(pool, mod_.id, false, true, None, None, None)
        .await?;
    assert_length!(1, list);
    assert_eq!(person.id, list[0].person.id);
    assert_eq!(community.id, list[0].community.id);
    assert_eq!(
      Some(CommunityFollowerState::ApprovalRequired),
      list[0].follow_state
    );
    assert!(list[0].is_new_instance);

    // approve the follow
    CommunityActions::follow_accepted(pool, community.id, person.id).await?;

    // now the user can post
    let posting_allowed =
      PendingFollowerView::check_private_community_action(pool, person.id, &community).await;
    assert!(posting_allowed.is_ok());

    // check counts again
    let count = PendingFollowerView::count_approval_required(pool, mod_.id).await?;
    assert_eq!(0, count);
    let list =
      PendingFollowerView::list_approval_required(pool, mod_.id, false, true, None, None, None)
        .await?;
    assert_length!(0, list);
    let list_all =
      PendingFollowerView::list_approval_required(pool, mod_.id, false, false, None, None, None)
        .await?;
    assert_length!(1, list_all);
    assert_eq!(person.id, list_all[0].person.id);
    assert_eq!(community.id, list_all[0].community.id);
    assert_eq!(
      Some(CommunityFollowerState::Accepted),
      list_all[0].follow_state
    );
    assert!(!list_all[0].is_new_instance);

    Instance::delete(pool, local_instance.id).await?;
    Instance::delete(pool, remote_instance.id).await?;
    Ok(())
  }
}
