use crate::ModlogView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  ModlogKindFilter,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, PostId},
  source::{
    local_user::LocalUser,
    modlog::{Modlog, modlog_keys as key},
  },
  utils::{
    limit_fetch,
    queries::filters::{
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      filter_suggested_communities,
    },
  },
};
use lemmy_db_schema_file::{
  PersonId,
  aliases,
  enums::ListingType,
  schema::{comment, community, community_actions, instance, modlog, person, post},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
};
use lemmy_utils::error::LemmyResult;

impl ModlogView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>) -> _ {
    // The query for the admin / mod person
    let moderator_join = person::table.on(modlog::mod_id.eq(person::id));

    // The modded / other person
    let target_person = aliases::person1.field(person::id).nullable();
    let target_person_join = aliases::person1.on(modlog::target_person_id.eq(target_person));

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community::id)
        .and(community_actions::person_id.nullable().eq(my_person_id)),
    );

    modlog::table
      .inner_join(moderator_join)
      .left_join(target_person_join)
      .left_join(comment::table.on(comment::id.nullable().eq(modlog::target_comment_id)))
      .left_join(post::table.on(post::id.nullable().eq(modlog::target_post_id)))
      .left_join(community::table.on(community::id.nullable().eq(modlog::target_community_id)))
      .left_join(instance::table.on(instance::id.nullable().eq(modlog::target_instance_id)))
      .left_join(community_actions_join)
  }
}

impl PaginationCursorConversion for ModlogView {
  type PaginatedType = Modlog;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.modlog.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let query = modlog::table
      .select(Self::PaginatedType::as_select())
      .filter(modlog::id.eq(cursor.id()?));
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
/// Querying / filtering the modlog.
pub struct ModlogQuery<'a> {
  pub type_: Option<ModlogKindFilter>,
  pub listing_type: Option<ListingType>,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub hide_modlog_names: Option<bool>,
  pub local_user: Option<&'a LocalUser>,
  pub mod_person_id: Option<PersonId>,
  pub target_person_id: Option<PersonId>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl ModlogQuery<'_> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<PagedResponse<ModlogView>> {
    let limit = limit_fetch(self.limit, None)?;

    let target_person = aliases::person1.field(person::id);
    let my_person_id = self.local_user.person_id();

    let mut query = ModlogView::joins(my_person_id)
      .select(ModlogView::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(mod_person_id) = self.mod_person_id {
      query = query.filter(person::id.eq(mod_person_id));
    };

    if let Some(target_person_id) = self.target_person_id {
      query = query.filter(target_person.eq(target_person_id));
    };

    if let Some(community_id) = self.community_id {
      query = query.filter(community::id.eq(community_id))
    }

    if let Some(post_id) = self.post_id {
      query = query.filter(post::id.eq(post_id))
    }

    if let Some(comment_id) = self.comment_id {
      query = query.filter(comment::id.eq(comment_id))
    }

    if let Some(type_) = self.type_ {
      query = match type_ {
        ModlogKindFilter::All => query,
        ModlogKindFilter::Other(kind) => query.filter(modlog::kind.eq(kind)),
      };
    }

    query = match self.listing_type.unwrap_or(ListingType::All) {
      ListingType::All => query,
      ListingType::Subscribed => query.filter(filter_is_subscribed()),
      ListingType::Local => query
        .filter(community::local.eq(true))
        .filter(filter_not_unlisted_or_is_subscribed()),
      ListingType::ModeratorView => {
        query.filter(community_actions::became_moderator_at.is_not_null())
      }
      ListingType::Suggested => query.filter(filter_suggested_communities()),
    };

    // Sorting by published
    let paginated_query =
      ModlogView::paginate(query, &self.page_cursor, SortDirection::Desc, pool, None)
        .await?
        .then_order_by(key::published_at)
        // Tie breaker
        .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query.load::<ModlogView>(conn).await?;

    let hide_modlog_names = self.hide_modlog_names.unwrap_or_default();

    // Map the query results to the enum
    let out = res
      .into_iter()
      .map(|u| u.hide_mod_name(hide_modlog_names))
      .collect();

    paginate_response(out, limit, self.page_cursor)
  }
}

impl ModlogView {
  /// Hides modlog names by setting the moderator to None.
  pub fn hide_mod_name(self, hide_modlog_names: bool) -> Self {
    if hide_modlog_names {
      Self {
        moderator: None,
        ..self
      }
    } else {
      self
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use lemmy_db_schema::source::{
    comment::{Comment, CommentInsertForm},
    community::{Community, CommunityInsertForm},
    instance::Instance,
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
  };
  use lemmy_db_schema_file::enums::ModlogKind;
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    community: Community,
    community_2: Community,
    post: Post,
    post_2: Post,
    comment: Comment,
    comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_rcv");
    let timmy = Person::create(pool, &timmy_form).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_rcv");
    let sara = Person::create(pool, &sara_form).await?;

    let jessica_form = PersonInsertForm::test_form(instance.id, "jessica_mrv");
    let jessica = Person::create(pool, &jessica_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community crv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let community_form_2 = CommunityInsertForm::new(
      instance.id,
      "test community crv 2".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community_2 = Community::create(pool, &community_form_2).await?;

    let post_form = PostInsertForm::new("A test post crv".into(), timmy.id, community.id);
    let post = Post::create(pool, &post_form).await?;

    let new_post_2 = PostInsertForm::new("A test post crv 2".into(), sara.id, community_2.id);
    let post_2 = Post::create(pool, &new_post_2).await?;

    // Timmy creates a comment
    let comment_form = CommentInsertForm::new(timmy.id, post.id, "A test comment rv".into());
    let comment = Comment::create(pool, &comment_form, None).await?;

    // jessica creates a comment
    let comment_form_2 =
      CommentInsertForm::new(jessica.id, post_2.id, "A test comment rv 2".into());
    let comment_2 = Comment::create(pool, &comment_form_2, None).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      community,
      community_2,
      post,
      post_2,
      comment,
      comment_2,
    })
  }
  use lemmy_db_schema::source::modlog::ModlogInsertForm;

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn admin_types() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form =
      ModlogInsertForm::admin_allow_instance(data.timmy.id, data.instance.id, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form =
      ModlogInsertForm::admin_block_instance(data.timmy.id, data.instance.id, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_purge_comment(
      data.timmy.id,
      &data.comment,
      data.community.id,
      "reason",
    );
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_purge_community(data.timmy.id, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_purge_person(data.timmy.id, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_purge_post(data.timmy.id, data.community.id, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_change_community_visibility(data.timmy.id, data.community.id);
    Modlog::create(pool, &[form]).await?;

    // A 2nd mod hide community, but to a different community, and with jessica
    let form =
      ModlogInsertForm::mod_change_community_visibility(data.jessica.id, data.community_2.id);
    Modlog::create(pool, &[form]).await?;

    let modlog = ModlogQuery::default().list(pool).await?.items;
    assert_eq!(8, modlog.len());

    let v = &modlog[0];
    assert_eq!(ModlogKind::ModChangeCommunityVisibility, v.modlog.kind);
    assert_eq!(
      Some(data.community_2.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.jessica.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[1];
    assert_eq!(ModlogKind::ModChangeCommunityVisibility, v.modlog.kind);
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[2];
    assert_eq!(ModlogKind::AdminPurgePost, v.modlog.kind);
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[3];
    assert_eq!(ModlogKind::AdminPurgePerson, v.modlog.kind);
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[4];
    assert_eq!(ModlogKind::AdminPurgeCommunity, v.modlog.kind);
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[5];
    assert_eq!(ModlogKind::AdminPurgeComment, v.modlog.kind);
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    // Make sure the report types are correct
    let v = &modlog[6]; // TODO: why index 2 again?
    assert_eq!(ModlogKind::AdminBlockInstance, v.modlog.kind);
    assert_eq!(
      Some(data.instance.id),
      v.target_instance.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[7];
    assert_eq!(ModlogKind::AdminAllowInstance, v.modlog.kind);
    assert_eq!(
      Some(data.instance.id),
      v.target_instance.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    // Filter by admin
    let modlog_admin_filter = ModlogQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    // Only one is jessica
    assert_eq!(7, modlog_admin_filter.len());

    // Filter by community
    let modlog_community_filter = ModlogQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // Should be 2, and not jessicas
    assert_eq!(3, modlog_community_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogQuery {
      type_: Some(ModlogKindFilter::Other(
        ModlogKind::ModChangeCommunityVisibility,
      )),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // 2 of these, one is jessicas
    assert_eq!(2, modlog_type_filter.len());

    let v = &modlog[0];
    assert_eq!(ModlogKind::ModChangeCommunityVisibility, v.modlog.kind);
    assert_eq!(
      Some(data.community_2.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.jessica.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[1];
    assert_eq!(ModlogKind::ModChangeCommunityVisibility, v.modlog.kind);
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mod_types() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = ModlogInsertForm::admin_add(&data.timmy, data.jessica.id, false);
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_add_to_community(
      data.timmy.id,
      data.community.id,
      data.jessica.id,
      false,
    );
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_ban(&data.timmy, data.jessica.id, true, None, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_ban_from_community(
      data.timmy.id,
      data.community.id,
      data.jessica.id,
      true,
      None,
      "reason",
    );
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_feature_post_community(data.timmy.id, &data.post, true);
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_feature_post_site(data.timmy.id, &data.post, true);
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_lock_post(data.timmy.id, &data.post, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_lock_comment(data.timmy.id, &data.comment, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_remove_comment(data.timmy.id, &data.comment, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::admin_remove_community(
      data.timmy.id,
      data.community.id,
      None,
      true,
      "reason",
    );
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_remove_post(data.timmy.id, &data.post, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form =
      ModlogInsertForm::mod_transfer_community(data.timmy.id, data.community.id, data.jessica.id);
    Modlog::create(pool, &[form]).await?;

    // A few extra ones to test different filters
    let form =
      ModlogInsertForm::mod_transfer_community(data.jessica.id, data.community_2.id, data.sara.id);
    Modlog::create(pool, &[form]).await?;

    let form = ModlogInsertForm::mod_remove_post(data.jessica.id, &data.post_2, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let form =
      ModlogInsertForm::mod_remove_comment(data.jessica.id, &data.comment_2, true, "reason");
    Modlog::create(pool, &[form]).await?;

    // The all view
    let modlog = ModlogQuery::default().list(pool).await?;
    assert_eq!(15, modlog.len());

    let v = &modlog[0];
    assert_eq!(ModlogKind::ModRemoveComment, v.modlog.kind);
    assert_eq!(
      Some(data.comment_2.id),
      v.target_comment.as_ref().map(|a| a.id)
    );
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.jessica.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[1];
    assert_eq!(ModlogKind::ModRemovePost, v.modlog.kind);
    assert_eq!(Some(data.post_2.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(Some(data.sara.id), v.target_person.as_ref().map(|a| a.id));
    assert_eq!(Some(data.jessica.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[2];
    assert_eq!(ModlogKind::ModTransferCommunity, v.modlog.kind);
    assert_eq!(
      Some(data.community_2.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.sara.id), v.target_person.as_ref().map(|a| a.id));
    assert_eq!(Some(data.jessica.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[3];
    assert_eq!(ModlogKind::ModTransferCommunity, v.modlog.kind);
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[4];
    assert_eq!(ModlogKind::ModRemovePost, v.modlog.kind);
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(Some(data.timmy.id), v.target_person.as_ref().map(|a| a.id));
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[5];
    assert_eq!(ModlogKind::AdminRemoveCommunity, v.modlog.kind);
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[6];
    assert_eq!(ModlogKind::ModRemoveComment, v.modlog.kind);
    assert_eq!(
      Some(data.comment.id),
      v.target_comment.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[7];
    assert_eq!(ModlogKind::ModLockComment, v.modlog.kind);
    assert_eq!(
      Some(data.comment.id),
      v.target_comment.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[8];
    assert_eq!(ModlogKind::ModLockPost, v.modlog.kind);
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[9];
    assert_eq!(ModlogKind::AdminFeaturePostSite, v.modlog.kind);
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[10];
    assert_eq!(ModlogKind::ModFeaturePostCommunity, v.modlog.kind);
    assert_eq!(Some(data.post.id), v.target_post.as_ref().map(|a| a.id));
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[11];
    assert_eq!(ModlogKind::ModBanFromCommunity, v.modlog.kind);
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[12];
    assert_eq!(ModlogKind::AdminBan, v.modlog.kind);
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[13];
    assert_eq!(ModlogKind::ModAddToCommunity, v.modlog.kind);
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(
      Some(data.community.id),
      v.target_community.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    let v = &modlog[14];
    assert_eq!(ModlogKind::AdminAdd, v.modlog.kind);
    assert_eq!(
      Some(data.jessica.id),
      v.target_person.as_ref().map(|a| a.id)
    );
    assert_eq!(Some(data.timmy.id), v.moderator.as_ref().map(|a| a.id));

    // Filter by moderator
    let modlog_mod_timmy_filter = ModlogQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(12, modlog_mod_timmy_filter.len());

    let modlog_mod_jessica_filter = ModlogQuery {
      mod_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_mod_jessica_filter.len());

    // Filter by target_person
    // Gets a little complicated because things aren't directly linked,
    // you have to go into the item to see who created it.

    let modlog_modded_timmy_filter = ModlogQuery {
      target_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, modlog_modded_timmy_filter.len());

    let modlog_modded_jessica_filter = ModlogQuery {
      target_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(6, modlog_modded_jessica_filter.len());

    let modlog_modded_sara_filter = ModlogQuery {
      target_person_id: Some(data.sara.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_modded_sara_filter.len());

    // Filter by community
    let modlog_community_filter = ModlogQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(6, modlog_community_filter.len());

    let modlog_community_2_filter = ModlogQuery {
      community_id: Some(data.community_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_community_2_filter.len());

    // Filter by post
    let modlog_post_filter = ModlogQuery {
      post_id: Some(data.post.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(5, modlog_post_filter.len());

    let modlog_post_2_filter = ModlogQuery {
      post_id: Some(data.post_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_post_2_filter.len());

    // Filter by comment
    let modlog_comment_filter = ModlogQuery {
      comment_id: Some(data.comment.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_comment_filter.len());

    let modlog_comment_2_filter = ModlogQuery {
      comment_id: Some(data.comment_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_comment_2_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogQuery {
      type_: Some(ModlogKindFilter::Other(ModlogKind::ModRemoveComment)),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_type_filter.len());

    // Assert that the types are correct
    assert_eq!(
      ModlogKind::ModRemoveComment,
      modlog_type_filter[0].modlog.kind,
    );
    assert_eq!(
      ModlogKind::ModRemoveComment,
      modlog_type_filter[1].modlog.kind,
    );

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn hide_modlog_names() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form =
      ModlogInsertForm::admin_allow_instance(data.timmy.id, data.instance.id, true, "reason");
    Modlog::create(pool, &[form]).await?;

    let modlog = ModlogQuery::default().list(pool).await?;
    assert_eq!(1, modlog.len());

    assert_eq!(ModlogKind::AdminAllowInstance, modlog[0].modlog.kind);
    assert_eq!(
      Some(data.timmy.id),
      modlog[0].moderator.as_ref().map(|a| a.id)
    );

    // Filter out the names
    let modlog_hide_names_filter = ModlogQuery {
      hide_modlog_names: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_hide_names_filter.len());

    assert_eq!(
      ModlogKind::AdminAllowInstance,
      modlog_hide_names_filter[0].modlog.kind
    );
    assert!(modlog_hide_names_filter[0].moderator.is_none());

    cleanup(data, pool).await?;

    Ok(())
  }
}
