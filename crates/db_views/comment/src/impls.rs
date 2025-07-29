use crate::{CommentSlimView, CommentView};
use diesel::{
  dsl::exists,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{nlevel, Ltree, LtreeExtensions};
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, InstanceId, PaginationCursor, PersonId, PostId},
  source::{
    comment::{comment_keys as key, Comment},
    local_user::LocalUser,
    site::Site,
  },
  traits::{Crud, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    now,
    paginate,
    queries::{
      creator_community_actions_join,
      creator_community_instance_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      filter_blocked,
      my_comment_actions_join,
      my_community_actions_join,
      my_instance_communities_actions_join,
      my_instance_persons_actions_join_1,
      my_local_user_admin_join,
      my_person_actions_join,
      suggested_communities,
    },
    seconds_to_pg_interval,
    DbPool,
    Subpath,
  },
};
use lemmy_db_schema_file::{
  enums::{
    CommentSortType::{self, *},
    CommunityFollowerState,
    CommunityVisibility,
    ListingType,
  },
  schema::{comment, community, community_actions, local_user_language, person, post},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl PaginationCursorBuilder for CommentView {
  type CursorData = Comment;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('C', self.comment.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let [(_, id)] = cursor.prefixes_and_ids()?;
    Comment::read(pool, CommentId(id)).await
  }
}

impl CommentView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let community_join = community::table.on(post::community_id.eq(community::id));

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_instance_communities_actions_join: my_instance_communities_actions_join =
      my_instance_communities_actions_join(my_person_id);
    let my_instance_persons_actions_join_1: my_instance_persons_actions_join_1 =
      my_instance_persons_actions_join_1(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    comment::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_community_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_community_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_instance_communities_actions_join)
      .left_join(my_instance_persons_actions_join_1)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    my_local_user: Option<&'_ LocalUser>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins(my_local_user.person_id(), local_instance_id)
      .filter(comment::id.eq(comment_id))
      .select(Self::as_select())
      .into_boxed();

    query = my_local_user.visible_communities_only(query);

    // Check permissions to view private community content.
    // Specifically, if the community is private then only accepted followers may view its
    // content, otherwise it is filtered out. Admins can view private community content
    // without restriction.
    if !my_local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
    }

    query
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub fn map_to_slim(self) -> CommentSlimView {
    CommentSlimView {
      comment: self.comment,
      creator: self.creator,
      comment_actions: self.comment_actions,
      person_actions: self.person_actions,
      creator_is_admin: self.creator_is_admin,
      can_mod: self.can_mod,
      creator_banned: self.creator_banned,
      creator_banned_from_community: self.creator_banned_from_community,
      creator_is_moderator: self.creator_is_moderator,
    }
  }
}

#[derive(Default)]
pub struct CommentQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommentSortType>,
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub post_id: Option<PostId>,
  pub parent_path: Option<Ltree>,
  pub local_user: Option<&'a LocalUser>,
  pub max_depth: Option<i32>,
  pub cursor_data: Option<Comment>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl CommentQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> LemmyResult<Vec<CommentView>> {
    let conn = &mut get_conn(pool).await?;
    let o = self;

    // The left join below will return None in this case
    let my_person_id = o.local_user.person_id();
    let local_user_id = o.local_user.local_user_id();

    let mut query = CommentView::joins(my_person_id, site.instance_id)
      .select(CommentView::as_select())
      .into_boxed();

    if let Some(post_id) = o.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = o.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };

    if let Some(community_id) = o.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    let is_subscribed = community_actions::followed_at.is_not_null();

    // For posts, we only show hidden if its subscribed, but for comments,
    // we ignore hidden.
    query = match o.listing_type.unwrap_or_default() {
      ListingType::Subscribed => query.filter(is_subscribed),
      ListingType::Local => query.filter(community::local.eq(true)),
      ListingType::All => query,
      ListingType::ModeratorView => {
        query.filter(community_actions::became_moderator_at.is_not_null())
      }
      ListingType::Suggested => query.filter(suggested_communities()),
    };

    if !o.local_user.show_bot_accounts() {
      query = query.filter(person::bot_account.eq(false));
    };

    if o.local_user.is_some() && o.listing_type.unwrap_or_default() != ListingType::ModeratorView {
      // Filter out the rows with missing languages
      query = query.filter(exists(
        local_user_language::table.filter(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(
              local_user_language::local_user_id
                .nullable()
                .eq(local_user_id),
            ),
        ),
      ));

      query = query.filter(filter_blocked());
    };

    if !o.local_user.show_nsfw(site) {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    query = o.local_user.visible_communities_only(query);
    query = query.filter(
      comment::federation_pending
        .eq(false)
        .or(comment::creator_id.nullable().eq(my_person_id)),
    );

    if !o.local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
    }

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query =
        query.filter(comment::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // A Max depth given means its a tree fetch
    let limit = if let Some(max_depth) = o.max_depth {
      let depth_limit = if let Some(parent_path) = o.parent_path.as_ref() {
        let count: i32 = parent_path.0.split('.').count().try_into()?;
        count + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // TODO limit question. Limiting does not work for comment threads ATM, only max_depth
      // For now, don't do any limiting for tree fetches
      // https://stackoverflow.com/questions/72983614/postgres-ltree-how-to-limit-the-max-number-of-children-at-any-given-level

      // Don't use the regular error-checking one, many more comments must ofter be fetched.
      // This does not work for comment trees, and the limit should be manually set to a high number
      //
      // If a max depth is given, then you know its a tree fetch, and limits should be ignored
      // TODO a kludge to prevent attacks. Limit comments to 300 for now.
      // (i64::MAX, 0)
      300
    } else {
      limit_fetch(o.limit)?
    };
    query = query.limit(limit);

    // Only sort by ascending for Old
    let sort = o.sort.unwrap_or(Hot);
    let sort_direction = asc_if(sort == Old);

    let mut pq = paginate(query, sort_direction, o.cursor_data, None, o.page_back);

    // Order by a subpath for max depth queries
    // Only order if filtering by a post id, or parent_path. DOS potential otherwise and max_depth
    // + !post_id isn't used anyways (afaik)
    if o.max_depth.is_some() && (o.post_id.is_some() || o.parent_path.is_some()) {
      // Always order by the parent path first
      pq = pq.then_order_by(Subpath(key::path));
    }

    // Distinguished comments should go first when viewing post
    // Don't do for new / old sorts
    if sort != New && sort != Old && (o.post_id.is_some() || o.parent_path.is_some()) {
      pq = pq.then_order_by(key::distinguished);
    }

    pq = match sort {
      Hot => pq.then_order_by(key::hot_rank).then_order_by(key::score),
      Controversial => pq.then_order_by(key::controversy_rank),
      Old | New => pq.then_order_by(key::published_at),
      Top => pq.then_order_by(key::score),
    };

    let res = pq.load::<CommentView>(conn).await?;

    Ok(res)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::{
    impls::{CommentQuery, DbPool},
    CommentView,
  };
  use lemmy_db_schema::{
    assert_length,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::CommentId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm, CommentUpdateForm},
      community::{
        Community,
        CommunityActions,
        CommunityFollowerForm,
        CommunityInsertForm,
        CommunityModeratorForm,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm},
      post::{Post, PostInsertForm, PostUpdateForm},
      site::{Site, SiteInsertForm},
    },
    traits::{Bannable, Blockable, Crud, Followable, Likeable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  // TODO rename these
  struct Data {
    instance: Instance,
    comment_0: Comment,
    comment_1: Comment,
    comment_2: Comment,
    _comment_5: Comment,
    post: Post,
    timmy_local_user_view: LocalUserView,
    sara_person: Person,
    community: Community,
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_person_form = PersonInsertForm::test_form(inserted_instance.id, "timmy");
    let inserted_timmy_person = Person::create(pool, &timmy_person_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form_admin(inserted_timmy_person.id);

    let inserted_timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;

    let sara_person_form = PersonInsertForm::test_form(inserted_instance.id, "sara");
    let sara_person = Person::create(pool, &sara_person_form).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test community 5".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post 2".into(),
      inserted_timmy_person.id,
      community.id,
    );
    let post = Post::create(pool, &new_post).await?;
    let english_id = Language::read_id_from_code(pool, "en").await?;

    // Create a comment tree with this hierarchy
    //       0
    //     \     \
    //    1      2
    //    \
    //  3  4
    //     \
    //     5
    let comment_form_0 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(inserted_timmy_person.id, post.id, "Comment 0".into())
    };

    let comment_0 = Comment::create(pool, &comment_form_0, None).await?;

    let comment_form_1 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(sara_person.id, post.id, "Comment 1".into())
    };
    let comment_1 = Comment::create(pool, &comment_form_1, Some(&comment_0.path)).await?;

    let finnish_id = Language::read_id_from_code(pool, "fi").await?;
    let comment_form_2 = CommentInsertForm {
      language_id: Some(finnish_id),
      ..CommentInsertForm::new(inserted_timmy_person.id, post.id, "Comment 2".into())
    };

    let comment_2 = Comment::create(pool, &comment_form_2, Some(&comment_0.path)).await?;

    let comment_form_3 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(inserted_timmy_person.id, post.id, "Comment 3".into())
    };
    let _inserted_comment_3 = Comment::create(pool, &comment_form_3, Some(&comment_1.path)).await?;

    let polish_id = Language::read_id_from_code(pool, "pl").await?;
    let comment_form_4 = CommentInsertForm {
      language_id: Some(polish_id),
      ..CommentInsertForm::new(inserted_timmy_person.id, post.id, "Comment 4".into())
    };

    let inserted_comment_4 = Comment::create(pool, &comment_form_4, Some(&comment_1.path)).await?;

    let comment_form_5 =
      CommentInsertForm::new(inserted_timmy_person.id, post.id, "Comment 5".into());
    let _comment_5 = Comment::create(pool, &comment_form_5, Some(&inserted_comment_4.path)).await?;

    let timmy_blocks_sara_form = PersonBlockForm::new(inserted_timmy_person.id, sara_person.id);
    let inserted_block = PersonActions::block(pool, &timmy_blocks_sara_form).await?;

    assert_eq!(
      (inserted_timmy_person.id, sara_person.id, true),
      (
        inserted_block.person_id,
        inserted_block.target_id,
        inserted_block.blocked_at.is_some()
      )
    );

    let comment_like_form = CommentLikeForm::new(inserted_timmy_person.id, comment_0.id, 1);

    CommentActions::like(pool, &comment_like_form).await?;

    let timmy_local_user_view = LocalUserView {
      local_user: inserted_timmy_local_user.clone(),
      person: inserted_timmy_person.clone(),
      banned: false,
    };
    let site_form = SiteInsertForm::new("test site".to_string(), inserted_instance.id);
    let site = Site::create(pool, &site_form).await?;
    Ok(Data {
      instance: inserted_instance,
      comment_0,
      comment_1,
      comment_2,
      _comment_5,
      post,
      timmy_local_user_view,
      sara_person,
      community,
      site,
    })
  }

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let read_comment_views_no_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.post.id)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert!(read_comment_views_no_person[0].comment_actions.is_none());
    assert!(!read_comment_views_no_person[0].can_mod);

    let read_comment_views_with_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.post.id)),
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert!(read_comment_views_with_person[0]
      .comment_actions
      .as_ref()
      .is_some_and(|x| x.like_score == Some(1)));
    assert!(read_comment_views_with_person[0].can_mod);

    // Make sure its 1, not showing the blocked comment
    assert_length!(5, read_comment_views_with_person);

    let read_comment_from_blocked_person = CommentView::read(
      pool,
      data.comment_1.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await?;

    // Make sure block set the creator blocked
    assert!(read_comment_from_blocked_person
      .person_actions
      .is_some_and(|x| x.blocked_at.is_some()));

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_comment_tree() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let top_path = data.comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery {
      post_id: (Some(data.post.id)),
      parent_path: (Some(top_path)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    let child_path = data.comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery {
      post_id: (Some(data.post.id)),
      parent_path: (Some(child_path)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure the comment parent-limited fetch is correct
    assert_length!(6, read_comment_views_top_path);
    assert_length!(4, read_comment_views_child_path);

    // Make sure it contains the parent, but not the comment from the other tree
    let child_comments = read_comment_views_child_path
      .into_iter()
      .map(|c| c.comment.id)
      .collect::<Vec<CommentId>>();
    assert!(child_comments.contains(&data.comment_1.id));
    assert!(!child_comments.contains(&data.comment_2.id));

    let read_comment_views_top_max_depth = CommentQuery {
      post_id: (Some(data.post.id)),
      max_depth: (Some(1)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure a depth limited one only has the top comment
    assert_length!(1, read_comment_views_top_max_depth);

    let child_path = data.comment_1.path.clone();
    let read_comment_views_parent_max_depth = CommentQuery {
      post_id: (Some(data.post.id)),
      parent_path: (Some(child_path)),
      max_depth: (Some(1)),
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure a depth limited one, and given child comment 1, has 3
    // 1, 3, 4
    assert_eq!(
      vec!["Comment 1", "Comment 3", "Comment 4"],
      read_comment_views_parent_max_depth
        .iter()
        .map(|r| r.comment.content.as_str())
        .collect::<Vec<&str>>()
    );
    assert!(read_comment_views_parent_max_depth[1]
      .comment
      .content
      .eq("Comment 3"));
    assert_length!(3, read_comment_views_parent_max_depth);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_languages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(5, all_languages);

    // change user lang to finnish, should only show one post in finnish and one undetermined
    let finnish_id = Language::read_id_from_code(pool, "fi").await?;
    LocalUserLanguage::update(
      pool,
      vec![finnish_id],
      data.timmy_local_user_view.local_user.id,
    )
    .await?;
    let finnish_comments = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(1, finnish_comments);
    let finnish_comment = finnish_comments
      .iter()
      .find(|c| c.comment.language_id == finnish_id);
    assert!(finnish_comment.is_some());
    assert_eq!(
      Some(&data.comment_2.content),
      finnish_comment.map(|c| &c.comment.content)
    );

    // now show all comments with undetermined language (which is the default value)
    LocalUserLanguage::update(
      pool,
      vec![UNDETERMINED_ID],
      data.timmy_local_user_view.local_user.id,
    )
    .await?;
    let undetermined_comment = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(1, undetermined_comment);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_distinguished_first() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = CommentUpdateForm {
      distinguished: Some(true),
      ..Default::default()
    };
    Comment::update(pool, data.comment_2.id, &form).await?;

    let comments = CommentQuery {
      post_id: Some(data.comment_2.post_id),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(comments[0].comment.id, data.comment_2.id);
    assert!(comments[0].comment.distinguished);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_moderator() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Make one of the inserted persons a moderator
    let person_id = data.sara_person.id;
    let community_id = data.community.id;
    let form = CommunityModeratorForm::new(community_id, person_id);
    CommunityActions::join(pool, &form).await?;

    // Make sure that they come back as a mod in the list
    let comments = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_eq!(comments[1].creator.name, "sara");
    assert!(comments[1].creator_is_moderator);

    assert!(!comments[0].creator_is_moderator);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_admin() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comments = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Timmy is an admin, and make sure that field is true
    assert_eq!(comments[0].creator.name, "timmy");
    assert!(comments[0].creator_is_admin);

    // Sara isn't, make sure its false
    assert_eq!(comments[1].creator.name, "sara");
    assert!(!comments[1].creator_is_admin);

    cleanup(data, pool).await
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    CommentActions::remove_like(
      pool,
      data.timmy_local_user_view.person.id,
      data.comment_0.id,
    )
    .await?;
    Comment::delete(pool, data.comment_0.id).await?;
    Comment::delete(pool, data.comment_1.id).await?;
    Post::delete(pool, data.post.id).await?;
    Community::delete(pool, data.community.id).await?;
    Person::delete(pool, data.timmy_local_user_view.person.id).await?;
    LocalUser::delete(pool, data.timmy_local_user_view.local_user.id).await?;
    Person::delete(pool, data.sara_person.id).await?;
    Instance::delete(pool, data.instance.id).await?;
    Site::delete(pool, data.site.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn local_only_instance() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Community::update(
      pool,
      data.community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::LocalOnlyPrivate),
        ..Default::default()
      },
    )
    .await?;

    let unauthenticated_query = CommentQuery {
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, unauthenticated_query.len());

    let authenticated_query = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, authenticated_query.len());

    let unauthenticated_comment =
      CommentView::read(pool, data.comment_0.id, None, data.instance.id).await;
    assert!(unauthenticated_comment.is_err());

    let authenticated_comment = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await;
    assert!(authenticated_comment.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Test that comment view shows if local user is blocked from community
    let banned_from_comm_person = PersonInsertForm::test_form(data.instance.id, "jill");

    let inserted_banned_from_comm_person = Person::create(pool, &banned_from_comm_person).await?;

    let inserted_banned_from_comm_local_user = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form(inserted_banned_from_comm_person.id),
      vec![],
    )
    .await?;

    CommunityActions::ban(
      pool,
      &CommunityPersonBanForm::new(data.community.id, inserted_banned_from_comm_person.id),
    )
    .await?;

    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&inserted_banned_from_comm_local_user),
      data.instance.id,
    )
    .await?;

    assert!(comment_view
      .community_actions
      .is_some_and(|x| x.received_ban_at.is_some()));

    Person::delete(pool, inserted_banned_from_comm_person.id).await?;
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_not_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await?;

    assert!(comment_view.community_actions.is_none());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listings_hide_nsfw() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Mark a post as nsfw
    let update_form = PostUpdateForm {
      nsfw: Some(true),
      ..Default::default()
    };
    Post::update(pool, data.post.id, &update_form).await?;

    // Make sure comments of this post are not returned
    let comments = CommentQuery::default().list(&data.site, pool).await?;
    assert_eq!(0, comments.len());

    // Mark site as nsfw
    let mut site = data.site.clone();
    site.content_warning = Some("nsfw".to_string());

    // Now comments of nsfw post are returned
    let comments = CommentQuery::default().list(&site, pool).await?;
    assert_eq!(6, comments.len());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_private_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    // Mark community as private
    Community::update(
      pool,
      data.community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::Private),
        ..Default::default()
      },
    )
    .await?;

    // No comments returned without auth
    let read_comment_listing = CommentQuery::default().list(&data.site, pool).await?;
    assert_eq!(0, read_comment_listing.len());
    let comment_view = CommentView::read(pool, data.comment_0.id, None, data.instance.id).await;
    assert!(comment_view.is_err());

    // No comments returned for non-follower who is not admin
    data.timmy_local_user_view.local_user.admin = false;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await;
    assert!(comment_view.is_err());

    // Admin can view content without following
    data.timmy_local_user_view.local_user.admin = true;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await;
    assert!(comment_view.is_ok());
    data.timmy_local_user_view.local_user.admin = false;

    // User can view after following
    CommunityActions::follow(
      pool,
      &CommunityFollowerForm::new(
        data.community.id,
        data.timmy_local_user_view.person.id,
        CommunityFollowerState::Accepted,
      ),
    )
    .await?;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await;
    assert!(comment_view.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_removed() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    // Mark a comment as removed
    let form = CommentUpdateForm {
      removed: Some(true),
      ..Default::default()
    };
    Comment::update(pool, data.comment_0.id, &form).await?;

    // Read as normal user, content is cleared
    // Timmy leaves admin
    LocalUser::update(
      pool,
      data.timmy_local_user_view.local_user.id,
      &LocalUserUpdateForm {
        admin: Some(false),
        ..Default::default()
      },
    )
    .await?;
    data.timmy_local_user_view.local_user.admin = false;
    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await?;
    assert_eq!("", comment_view.comment.content);
    let comment_listing = CommentQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      sort: Some(CommentSortType::Old),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!("", comment_listing[0].comment.content);

    // Read as admin, content is returned
    LocalUser::update(
      pool,
      data.timmy_local_user_view.local_user.id,
      &LocalUserUpdateForm {
        admin: Some(true),
        ..Default::default()
      },
    )
    .await?;
    data.timmy_local_user_view.local_user.admin = true;
    let comment_view = CommentView::read(
      pool,
      data.comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
      data.instance.id,
    )
    .await?;
    assert_eq!(data.comment_0.content, comment_view.comment.content);
    let comment_listing = CommentQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      sort: Some(CommentSortType::Old),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(data.comment_0.content, comment_listing[0].comment.content);

    cleanup(data, pool).await
  }
}
