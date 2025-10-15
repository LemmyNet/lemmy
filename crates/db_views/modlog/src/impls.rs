use crate::{
  AdminAddView,
  AdminAllowInstanceView,
  AdminBanView,
  AdminBlockInstanceView,
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  AdminRemoveCommunityView,
  ModAddToCommunityView,
  ModBanFromCommunityView,
  ModChangeCommunityVisibilityView,
  ModFeaturePostView,
  ModLockCommentView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemovePostView,
  ModTransferCommunityView,
  ModlogData,
  ModlogView,
  ModlogViewInternal,
};
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
  aliases,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, PaginationCursor, PersonId, PostId},
  source::{
    local_user::LocalUser,
    modlog::{modlog_keys as key, Modlog},
  },
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::filters::{
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      filter_suggested_communities,
    },
    DbPool,
  },
};
use lemmy_db_schema_file::{
  enums::{ListingType, ModlogKind},
  schema::{comment, community, community_actions, instance, modlog, person, post},
};
use lemmy_utils::error::LemmyResult;

impl ModlogViewInternal {
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
      .left_join(moderator_join)
      .left_join(target_person_join)
      .left_join(comment::table)
      .left_join(post::table)
      .left_join(community::table)
      .left_join(instance::table)
      .left_join(community_actions_join)
  }
}

impl PaginationCursorBuilder for ModlogView {
  type CursorData = Modlog;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor(self.modlog.id.0.to_string())
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let id: i32 = cursor.0.parse()?;
    let query = modlog::table
      .select(Self::CursorData::as_select())
      .filter(modlog::id.eq(id));
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
/// Querying / filtering the modlog.
pub struct ModlogQuery<'a> {
  pub type_: Option<ModlogKind>,
  pub listing_type: Option<ListingType>,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub hide_modlog_names: Option<bool>,
  pub local_user: Option<&'a LocalUser>,
  pub mod_person_id: Option<PersonId>,
  pub target_person_id: Option<PersonId>,
  pub cursor_data: Option<Modlog>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl ModlogQuery<'_> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<ModlogView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let target_person = aliases::person1.field(person::id);
    let my_person_id = self.local_user.person_id();

    let mut query = ModlogViewInternal::joins(my_person_id)
      .select(ModlogViewInternal::as_select())
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
      query = query.filter(modlog::kind.eq(type_))
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
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::published_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query.load::<ModlogViewInternal>(conn).await?;

    let hide_modlog_names = self.hide_modlog_names.unwrap_or_default();

    // Map the query results to the enum
    let out = res
      .into_iter()
      .map(|u| u.hide_mod_name(hide_modlog_names))
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl ModlogViewInternal {
  /// Hides modlog names by setting the moderator to None.
  fn hide_mod_name(self, hide_modlog_names: bool) -> Self {
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

impl InternalToCombinedView for ModlogViewInternal {
  type CombinedView = ModlogView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    let data = match self.modlog.kind {
      ModlogKind::AdminAdd => ModlogData::AdminAdd(AdminAddView {
        moderator: self.moderator?,
        target_person: self.target_person?,
      }),
      ModlogKind::AdminBan => ModlogData::AdminBan(AdminBanView {
        moderator: self.moderator?,
        target_person: self.target_person?,
      }),
      ModlogKind::AdminAllowInstance => ModlogData::AdminAllowInstance(AdminAllowInstanceView {
        admin: self.moderator?,
        instance: self.target_instance?,
      }),
      ModlogKind::AdminBlockInstance => ModlogData::AdminBlockInstance(AdminBlockInstanceView {
        admin: self.moderator?,
        instance: self.target_instance?,
      }),
      ModlogKind::AdminPurgeComment => ModlogData::AdminPurgeComment(AdminPurgeCommentView {
        admin: self.moderator?,
      }),
      ModlogKind::AdminPurgeCommunity => ModlogData::AdminPurgeCommunity(AdminPurgeCommunityView {
        admin: self.moderator?,
      }),
      ModlogKind::AdminPurgePerson => ModlogData::AdminPurgePerson(AdminPurgePersonView {
        admin: self.moderator?,
      }),
      ModlogKind::AdminPurgePost => ModlogData::AdminPurgePost(AdminPurgePostView {
        admin: self.moderator?,
      }),
      ModlogKind::ModAddToCommunity => ModlogData::ModAddToCommunity(ModAddToCommunityView {
        moderator: self.moderator?,
        target_person: self.target_person?,
        community: self.target_community?,
      }),
      ModlogKind::ModBanFromCommunity => ModlogData::ModBanFromCommunity(ModBanFromCommunityView {
        moderator: self.moderator?,
        target_person: self.target_person?,
        community: self.target_community?,
      }),
      ModlogKind::ModFeaturePost => ModlogData::ModFeaturePost(ModFeaturePostView {
        moderator: self.moderator?,
        post: self.target_post?,
        community: self.target_community?,
      }),
      ModlogKind::ModChangeCommunityVisibility => {
        ModlogData::ModChangeCommunityVisibility(ModChangeCommunityVisibilityView {
          moderator: self.moderator?,
          community: self.target_community?,
        })
      }
      ModlogKind::ModLockPost => ModlogData::ModLockPost(ModLockPostView {
        moderator: self.moderator?,
        post: self.target_post?,
        community: self.target_community?,
      }),
      ModlogKind::ModRemoveComment => ModlogData::ModRemoveComment(ModRemoveCommentView {
        moderator: self.moderator?,
        comment: self.target_comment?,
        post: self.target_post?,
        community: self.target_community?,
      }),
      ModlogKind::AdminRemoveCommunity => {
        ModlogData::AdminRemoveCommunity(AdminRemoveCommunityView {
          moderator: self.moderator?,
          community: self.target_community?,
        })
      }
      ModlogKind::ModRemovePost => ModlogData::ModRemovePost(ModRemovePostView {
        moderator: self.moderator?,
        post: self.target_post?,
        community: self.target_community?,
      }),
      ModlogKind::ModTransferCommunity => {
        ModlogData::ModTransferCommunity(ModTransferCommunityView {
          moderator: self.moderator?,
          target_person: self.target_person?,
          community: self.target_community?,
        })
      }
      ModlogKind::ModLockComment => ModlogData::ModLockComment(ModLockCommentView {
        moderator: self.moderator?,
        comment: self.target_comment?,
        post: self.target_post?,
        community: self.target_community?,
      }),
    };

    Some(ModlogView {
      modlog: self.modlog,
      data,
    })
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use crate::AdminAllowInstance;
  use lemmy_db_schema::{
    newtypes::PersonId,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      mod_log::{
        admin::{
          AdminAdd,
          AdminAddForm,
          AdminAllowInstanceForm,
          AdminBan,
          AdminBanForm,
          AdminBlockInstance,
          AdminBlockInstanceForm,
          AdminRemoveCommunity,
          AdminRemoveCommunityForm,
        },
        moderator::{
          ModAddToCommunity,
          ModAddToCommunityForm,
          ModBanFromCommunity,
          ModBanFromCommunityForm,
          ModChangeCommunityVisibility,
          ModChangeCommunityVisibilityForm,
          ModFeaturePost,
          ModFeaturePostForm,
          ModLockComment,
          ModLockCommentForm,
          ModLockPost,
          ModLockPostForm,
          ModRemoveComment,
          ModRemoveCommentForm,
          ModRemovePost,
          ModRemovePostForm,
          ModTransferCommunity,
          ModTransferCommunityForm,
        },
      },
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
  };
  use lemmy_db_schema_file::enums::CommunityVisibility;
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
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

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

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: "reason".to_string(),
    };
    AdminAllowInstance::create(pool, &form).await?;

    let form = AdminBlockInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      blocked: true,
      reason: "reason".to_string(),
    };
    AdminBlockInstance::create(pool, &form).await?;

    let form = AdminPurgeCommentForm {
      admin_person_id: data.timmy.id,
      post_id: data.post.id,
      reason: "reason".to_string(),
    };
    AdminPurgeComment::create(pool, &form).await?;

    let form = AdminPurgeCommunityForm {
      admin_person_id: data.timmy.id,
      reason: "reason".to_string(),
    };
    AdminPurgeCommunity::create(pool, &form).await?;

    let form = AdminPurgePersonForm {
      admin_person_id: data.timmy.id,
      reason: "reason".to_string(),
    };
    AdminPurgePerson::create(pool, &form).await?;

    let form = AdminPurgePostForm {
      admin_person_id: data.timmy.id,
      community_id: data.community.id,
      reason: "reason".to_string(),
    };
    AdminPurgePost::create(pool, &form).await?;

    let form = ModChangeCommunityVisibilityForm {
      mod_person_id: data.timmy.id,
      community_id: data.community.id,
      visibility: CommunityVisibility::Unlisted,
    };
    ModChangeCommunityVisibility::create(pool, &form).await?;

    // A 2nd mod hide community, but to a different community, and with jessica
    let form = ModChangeCommunityVisibilityForm {
      mod_person_id: data.jessica.id,
      community_id: data.community_2.id,
      visibility: CommunityVisibility::Unlisted,
    };
    ModChangeCommunityVisibility::create(pool, &form).await?;

    let modlog = ModlogQuery::default().list(pool).await?;
    assert_eq!(4, modlog.len());

    if let ModlogView::ModChangeCommunityVisibility(v) = &modlog[0] {
      assert_eq!(
        data.community_2.id,
        v.mod_change_community_visibility.community_id
      );
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModChangeCommunityVisibility(v) = &modlog[1] {
      assert_eq!(
        data.community.id,
        v.mod_change_community_visibility.community_id
      );
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    /*
      Temporarily disabled to speed up compilation
      https://github.com/LemmyNet/lemmy/issues/6012
    if let ModlogView::AdminPurgePost(v) = &modlog[2] {
      assert_eq!(data.community.id, v.admin_purge_post.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminPurgePerson(v) = &modlog[3] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminPurgeCommunity(v) = &modlog[4] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminPurgeComment(v) = &modlog[5] {
      assert_eq!(data.post.id, v.admin_purge_comment.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }
    */

    // Make sure the report types are correct
    if let ModlogView::AdminBlockInstance(v) = &modlog[2] {
      assert_eq!(data.instance.id, v.admin_block_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminAllowInstance(v) = &modlog[3] {
      assert_eq!(data.instance.id, v.admin_allow_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter by admin
    let modlog_admin_filter = ModlogQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    // Only one is jessica
    assert_eq!(3, modlog_admin_filter.len());

    // Filter by community
    let modlog_community_filter = ModlogQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // Should be 2, and not jessicas
    assert_eq!(1, modlog_community_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogQuery {
      type_: Some(ModlogKind::ModChangeCommunityVisibility),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // 2 of these, one is jessicas
    assert_eq!(2, modlog_type_filter.len());

    if let ModlogView::ModChangeCommunityVisibility(v) = &modlog_type_filter[0] {
      assert_eq!(
        data.community_2.id,
        v.mod_change_community_visibility.community_id
      );
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModChangeCommunityVisibility(v) = &modlog_type_filter[1] {
      assert_eq!(
        data.community.id,
        v.mod_change_community_visibility.community_id
      );
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mod_types() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAddForm {
      mod_person_id: data.timmy.id,
      target_person_id: data.jessica.id,
      removed: Some(false),
    };
    AdminAdd::create(pool, &form).await?;

    let form = ModAddToCommunityForm {
      mod_person_id: data.timmy.id,
      target_person_id: data.jessica.id,
      community_id: data.community.id,
      removed: Some(false),
    };
    ModAddToCommunity::create(pool, &form).await?;

    let form = AdminBanForm {
      mod_person_id: data.timmy.id,
      target_person_id: data.jessica.id,
      banned: Some(true),
      reason: "reason".to_string(),
      expires_at: None,
      instance_id: data.instance.id,
    };
    AdminBan::create(pool, &form).await?;

    let form = ModBanFromCommunityForm {
      mod_person_id: data.timmy.id,
      target_person_id: data.jessica.id,
      community_id: data.community.id,
      banned: Some(true),
      reason: "reason".to_string(),
      expires_at: None,
    };
    ModBanFromCommunity::create(pool, &form).await?;

    let form = ModFeaturePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      featured: Some(true),
      is_featured_community: None,
    };
    ModFeaturePost::create(pool, &form).await?;

    let form = ModLockPostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      locked: Some(true),
      reason: "reason".to_string(),
    };
    ModLockPost::create(pool, &form).await?;

    let form = ModLockCommentForm {
      mod_person_id: data.timmy.id,
      comment_id: data.comment.id,
      locked: Some(true),
      reason: "reason".to_string(),
    };
    ModLockComment::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.timmy.id,
      comment_id: data.comment.id,
      removed: Some(true),
      reason: "reason".to_string(),
    };
    ModRemoveComment::create(pool, &form).await?;

    let form = AdminRemoveCommunityForm {
      mod_person_id: data.timmy.id,
      community_id: data.community.id,
      removed: Some(true),
      reason: "reason".to_string(),
    };
    AdminRemoveCommunity::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      removed: Some(true),
      reason: "reason".to_string(),
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModTransferCommunityForm {
      mod_person_id: data.timmy.id,
      target_person_id: data.jessica.id,
      community_id: data.community.id,
    };
    ModTransferCommunity::create(pool, &form).await?;

    // A few extra ones to test different filters
    let form = ModTransferCommunityForm {
      mod_person_id: data.jessica.id,
      target_person_id: data.sara.id,
      community_id: data.community_2.id,
    };
    ModTransferCommunity::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.jessica.id,
      post_id: data.post_2.id,
      removed: Some(true),
      reason: "reason".to_string(),
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.jessica.id,
      comment_id: data.comment_2.id,
      removed: Some(true),
      reason: "reason".to_string(),
    };
    ModRemoveComment::create(pool, &form).await?;

    // The all view
    let modlog = ModlogQuery::default().list(pool).await?;
    assert_eq!(14, modlog.len());

    if let ModlogView::ModRemoveComment(v) = &modlog[0] {
      assert_eq!(data.comment_2.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment_2.id, v.comment.id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModRemovePost(v) = &modlog[1] {
      assert_eq!(data.post_2.id, v.mod_remove_post.post_id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.sara.id, v.post.creator_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModTransferCommunity(v) = &modlog[2] {
      assert_eq!(data.community_2.id, v.mod_transfer_community.community_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModTransferCommunity(v) = &modlog[3] {
      assert_eq!(data.community.id, v.mod_transfer_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModRemovePost(v) = &modlog[4] {
      assert_eq!(data.post.id, v.mod_remove_post.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminRemoveCommunity(v) = &modlog[5] {
      assert_eq!(data.community.id, v.admin_remove_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModRemoveComment(v) = &modlog[6] {
      assert_eq!(data.comment.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModLockComment(v) = &modlog[7] {
      assert_eq!(data.comment.id, v.mod_lock_comment.comment_id);
      assert!(v.mod_lock_comment.locked);
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.timmy.id, v.comment.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModLockPost(v) = &modlog[8] {
      assert_eq!(data.post.id, v.mod_lock_post.post_id);
      assert!(v.mod_lock_post.locked);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModFeaturePost(v) = &modlog[9] {
      assert_eq!(data.post.id, v.mod_feature_post.post_id);
      assert!(v.mod_feature_post.featured);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModBanFromCommunity(v) = &modlog[10] {
      assert_eq!(data.community.id, v.mod_ban_from_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminBan(v) = &modlog[11] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::ModAddToCommunity(v) = &modlog[12] {
      assert_eq!(data.community.id, v.mod_add_to_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogView::AdminAdd(v) = &modlog[13] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.target_person.id);
    } else {
      panic!("wrong type");
    }

    // Filter by moderator
    let modlog_mod_timmy_filter = ModlogQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(11, modlog_mod_timmy_filter.len());

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
    assert_eq!(5, modlog_modded_timmy_filter.len());

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
    assert_eq!(9, modlog_community_filter.len());

    let modlog_community_2_filter = ModlogQuery {
      community_id: Some(data.community_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_community_2_filter.len());

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
      type_: Some(ModlogKind::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_type_filter.len());

    // Assert that the types are correct
    assert!(matches!(
      modlog_type_filter[0],
      ModlogView::ModRemoveComment(_)
    ));
    assert!(matches!(
      modlog_type_filter[1],
      ModlogView::ModRemoveComment(_)
    ));

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn hide_modlog_names() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: "reason".to_string(),
    };
    AdminAllowInstance::create(pool, &form).await?;

    let modlog = ModlogQuery::default().list(pool).await?;
    assert_eq!(1, modlog.len());

    if let ModlogView::AdminAllowInstance(v) = &modlog[0] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter out the names
    let modlog_hide_names_filter = ModlogQuery {
      hide_modlog_names: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_hide_names_filter.len());

    if let ModlogView::AdminAllowInstance(v) = &modlog_hide_names_filter[0] {
      assert!(v.admin.is_none())
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
