use crate::CommunityModeratorView;
use diesel::{dsl::exists, select, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  source::local_user::LocalUser,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{community, community_actions, person};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityModeratorView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    community_actions::table
      .filter(community_actions::became_moderator_at.is_not_null())
      .inner_join(community::table)
      .inner_join(person::table.on(person::id.eq(community_actions::person_id)))
  }

  pub async fn check_is_community_moderator(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins()
        .filter(community_actions::person_id.eq(person_id))
        .filter(community_actions::community_id.eq(community_id)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotAModerator.into())
  }

  pub async fn is_community_moderator_of_any(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins().filter(community_actions::person_id.eq(person_id)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotAModerator.into())
  }

  pub async fn for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .select(Self::as_select())
      .order_by(community_actions::became_moderator_at)
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_user: Option<&LocalUser>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins()
      .filter(community_actions::person_id.eq(person_id))
      .select(Self::as_select())
      .into_boxed();

    query = local_user.visible_communities_only(query);

    // only show deleted communities to creator
    if Some(person_id) != local_user.person_id() {
      query = query.filter(community::deleted.eq(false));
    }

    // Show removed communities to admins only
    if !local_user.is_admin() {
      query = query
        .filter(community::removed.eq(false))
        .filter(community::local_removed.eq(false));
    }

    query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .select(Self::as_select())
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_actions::community_id)
      .order_by((
        community_actions::community_id,
        community_actions::became_moderator_at,
      ))
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
