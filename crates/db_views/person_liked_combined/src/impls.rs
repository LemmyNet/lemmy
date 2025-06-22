use crate::{
  CommentView,
  LocalUserView,
  PersonLikedCombinedView,
  PersonLikedCombinedViewInternal,
  PostView,
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
  newtypes::{InstanceId, PaginationCursor, PersonId},
  source::combined::person_liked::{person_liked_combined_keys as key, PersonLikedCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      community_join,
      creator_community_actions_join,
      creator_community_instance_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      creator_local_user_admin_join,
      image_details_join,
      my_comment_actions_join,
      my_community_actions_join,
      my_instance_actions_person_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    DbPool,
  },
  LikeType,
  PersonContentType,
};
use lemmy_db_schema_file::schema::{comment, person, person_liked_combined, post};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[derive(Default)]
pub struct PersonLikedCombinedQuery {
  pub type_: Option<PersonContentType>,
  pub like_type: Option<LikeType>,
  pub cursor_data: Option<PersonLikedCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl PaginationCursorBuilder for PersonLikedCombinedView {
  type CursorData = PersonLikedCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      PersonLikedCombinedView::Comment(v) => ('C', v.comment.id.0),
      PersonLikedCombinedView::Post(v) => ('P', v.post.id.0),
    };
    PaginationCursor::new_single(prefix, id)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let pids = cursor.prefixes_and_ids();
    let (prefix, id) = pids
      .as_slice()
      .first()
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;

    let mut query = person_liked_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(person_liked_combined::comment_id.eq(id)),
      'P' => query.filter(person_liked_combined::post_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

impl PersonLikedCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  pub(crate) fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let comment_join =
      comment::table.on(person_liked_combined::comment_id.eq(comment::id.nullable()));

    let post_join = post::table.on(
      person_liked_combined::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id)),
    );

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        // Need to filter out the post rows where the post_id given is null
        // Otherwise you'll get duped post rows
        .or(
          post::creator_id
            .eq(item_creator)
            .and(person_liked_combined::post_id.is_not_null()),
        ),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    person_liked_combined::table
      .left_join(comment_join)
      .inner_join(post_join)
      .inner_join(item_creator_join)
      .inner_join(community_join())
      .left_join(creator_community_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_community_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(image_details_join())
  }
}

impl PersonLikedCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<PersonLikedCombinedView>> {
    let my_person_id = user.local_user.person_id;
    let local_instance_id = user.person.instance_id;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let mut query = PersonLikedCombinedViewInternal::joins(my_person_id, local_instance_id)
      .filter(person_liked_combined::person_id.eq(my_person_id))
      .select(PersonLikedCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_liked_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_liked_combined::post_id.is_not_null()),
      }
    }

    if let Some(like_type) = self.like_type {
      query = match like_type {
        LikeType::All => query,
        LikeType::LikedOnly => query.filter(person_liked_combined::like_score.eq(1)),
        LikeType::DislikedOnly => query.filter(person_liked_combined::like_score.eq(-1)),
      }
    }

    // Sorting by liked desc
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::liked_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<PersonLikedCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for PersonLikedCombinedViewInternal {
  type CombinedView = PersonLikedCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let Some(comment) = v.comment {
      Some(PersonLikedCombinedView::Comment(CommentView {
        comment,
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        community_actions: v.community_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
      }))
    } else {
      Some(PersonLikedCombinedView::Post(PostView {
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        image_details: v.image_details,
        community_actions: v.community_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
      }))
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{impls::PersonLikedCombinedQuery, LocalUserView, PersonLikedCombinedView};
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Crud, Likeable},
    utils::{build_db_pool_for_tests, DbPool},
    LikeType,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    timmy_view: LocalUserView,
    sara: Person,
    timmy_post: Post,
    sara_comment: Comment,
    sara_comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: timmy.clone(),
      banned: false,
    };

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_pcv");
    let sara = Person::create(pool, &sara_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community pcv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let timmy_post_form = PostInsertForm::new("timmy post prv".into(), timmy.id, community.id);
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let timmy_post_form_2 = PostInsertForm::new("timmy post prv 2".into(), timmy.id, community.id);
    let timmy_post_2 = Post::create(pool, &timmy_post_form_2).await?;

    let sara_post_form = PostInsertForm::new("sara post prv".into(), sara.id, community.id);
    let _sara_post = Post::create(pool, &sara_post_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv".into());
    let _timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.id, timmy_post.id, "sara comment prv".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

    let sara_comment_form_2 =
      CommentInsertForm::new(sara.id, timmy_post_2.id, "sara comment prv 2".into());
    let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

    Ok(Data {
      instance,
      timmy,
      timmy_view,
      sara,
      timmy_post,
      sara_comment,
      sara_comment_2,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_combined() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Do a batch read of timmy liked
    let timmy_liked = PersonLikedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(0, timmy_liked.len());

    // Like a few things
    let like_sara_comment_2 = CommentLikeForm::new(data.timmy.id, data.sara_comment_2.id, 1);
    CommentActions::like(pool, &like_sara_comment_2).await?;

    let dislike_sara_comment = CommentLikeForm::new(data.timmy.id, data.sara_comment.id, -1);
    CommentActions::like(pool, &dislike_sara_comment).await?;

    let post_like_form = PostLikeForm::new(data.timmy_post.id, data.timmy.id, 1);
    PostActions::like(pool, &post_like_form).await?;

    let timmy_liked_all = PersonLikedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(3, timmy_liked_all.len());

    // Make sure the types and order are correct
    if let PersonLikedCombinedView::Post(v) = &timmy_liked_all[0] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(Some(1), v.post_actions.as_ref().and_then(|l| l.like_score));
    } else {
      panic!("wrong type");
    }
    if let PersonLikedCombinedView::Comment(v) = &timmy_liked_all[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
      assert_eq!(
        Some(-1),
        v.comment_actions.as_ref().and_then(|l| l.like_score)
      );
    } else {
      panic!("wrong type");
    }
    if let PersonLikedCombinedView::Comment(v) = &timmy_liked_all[2] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
      assert_eq!(
        Some(1),
        v.comment_actions.as_ref().and_then(|l| l.like_score)
      );
    } else {
      panic!("wrong type");
    }

    let timmy_disliked = PersonLikedCombinedQuery {
      like_type: Some(LikeType::DislikedOnly),
      ..PersonLikedCombinedQuery::default()
    }
    .list(pool, &data.timmy_view)
    .await?;
    assert_eq!(1, timmy_disliked.len());

    if let PersonLikedCombinedView::Comment(v) = &timmy_disliked[0] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
      assert_eq!(
        Some(-1),
        v.comment_actions.as_ref().and_then(|l| l.like_score)
      );
    } else {
      panic!("wrong type");
    }

    // Try unliking 2 things
    CommentActions::remove_like(pool, data.timmy.id, data.sara_comment.id).await?;
    PostActions::remove_like(pool, data.timmy.id, data.timmy_post.id).await?;

    let timmy_likes_removed = PersonLikedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(1, timmy_likes_removed.len());

    if let PersonLikedCombinedView::Comment(v) = &timmy_likes_removed[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
