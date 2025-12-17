use crate::LocalUserView;
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
  self,
  PersonContentType,
  source::combined::person_content::{PersonContentCombined, person_content_combined_keys as key},
  traits::InternalToCombinedView,
  utils::limit_fetch,
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  joins::{
    community_join,
    creator_community_actions_join,
    creator_community_instance_actions_join,
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
    creator_local_user_admin_join,
    image_details_join,
    my_comment_actions_join,
    my_community_actions_join,
    my_local_user_admin_join,
    my_person_actions_join,
    my_post_actions_join,
  },
  schema::{comment, person, person_content_combined, post},
};
use lemmy_db_views_post_comment_combined::{
  PostCommentCombinedView,
  PostCommentCombinedViewInternal,
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
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
struct PostCommentCombinedViewWrapper(PostCommentCombinedView);

impl PaginationCursorConversion for PostCommentCombinedViewWrapper {
  type PaginatedType = PersonContentCombined;

  fn to_cursor(&self) -> CursorData {
    let (prefix, id) = match &self.0 {
      PostCommentCombinedView::Comment(v) => ('C', v.comment.id.0),
      PostCommentCombinedView::Post(v) => ('P', v.post.id.0),
    };
    CursorData::new_with_prefix(prefix, id)
  }

  async fn from_cursor(
    data: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;

    let mut query = person_content_combined::table
      .select(Self::PaginatedType::as_select())
      .into_boxed();

    let (prefix, id) = data.id_and_prefix()?;
    query = match prefix {
      'C' => query.filter(person_content_combined::comment_id.eq(id)),
      'P' => query.filter(person_content_combined::post_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(derive_new::new)]
pub struct PersonContentCombinedQuery {
  pub creator_id: PersonId,
  #[new(default)]
  pub type_: Option<PersonContentType>,
  #[new(default)]
  pub page_cursor: Option<PaginationCursor>,
  #[new(default)]
  pub limit: Option<i64>,
  #[new(default)]
  pub no_limit: Option<bool>,
}

impl PersonContentCombinedQuery {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let comment_join =
      comment::table.on(person_content_combined::comment_id.eq(comment::id.nullable()));

    let post_join = post::table.on(
      person_content_combined::post_id
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
            .and(person_content_combined::post_id.is_not_null()),
        ),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(my_person_id);
    let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    person_content_combined::table
      .left_join(comment_join)
      .inner_join(post_join)
      .inner_join(item_creator_join)
      .inner_join(community_join())
      .left_join(image_details_join())
      .left_join(creator_community_actions_join())
      .left_join(creator_local_user_admin_join())
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_community_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: Option<&LocalUserView>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<PagedResponse<PostCommentCombinedView>> {
    let my_person_id = user.as_ref().map(|u| u.local_user.person_id);
    let item_creator = person::id;

    // Notes: since the post_id and comment_id are optional columns,
    // many joins must use an OR condition.
    // For example, the creator must be the person table joined to either:
    // - post.creator_id
    // - comment.creator_id
    let mut query = Self::joins(my_person_id, local_instance_id)
      // The creator id filter
      .filter(item_creator.eq(self.creator_id))
      .select(PostCommentCombinedViewInternal::as_select())
      .into_boxed();

    let limit = limit_fetch(self.limit, self.no_limit)?;
    query = query.limit(limit);

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_content_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_content_combined::post_id.is_not_null()),
      }
    }

    // Sorting by published
    let paginated_query = PostCommentCombinedViewWrapper::paginate(
      query,
      &self.page_cursor,
      SortDirection::Desc,
      pool,
      None,
    )
    .await?
    .then_order_by(key::published_at)
    // Tie breaker
    .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<PostCommentCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .map(PostCommentCombinedViewWrapper)
      .collect();

    let res = paginate_response(out, limit, self.page_cursor)?;
    Ok(PagedResponse {
      items: res.items.into_iter().map(|i| i.0).collect(),
      next_page: res.next_page,
      prev_page: res.prev_page,
    })
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::impls::PersonContentCombinedQuery;
  use lemmy_db_schema::source::{
    comment::{Comment, CommentInsertForm},
    community::{Community, CommunityInsertForm},
    instance::Instance,
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
  };
  use lemmy_db_views_post_comment_combined::PostCommentCombinedView;
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
    timmy_post: Post,
    timmy_post_2: Post,
    sara_post: Post,
    timmy_comment: Comment,
    sara_comment: Comment,
    sara_comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;

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
    let sara_post = Post::create(pool, &sara_post_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv".into());
    let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.id, timmy_post.id, "sara comment prv".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

    let sara_comment_form_2 =
      CommentInsertForm::new(sara.id, timmy_post_2.id, "sara comment prv 2".into());
    let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      timmy_post,
      timmy_post_2,
      sara_post,
      timmy_comment,
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

    // Do a batch read of timmy
    let timmy_content = PersonContentCombinedQuery::new(data.timmy.id)
      .list(pool, None, data.instance.id)
      .await?;
    assert_eq!(3, timmy_content.len());

    // Make sure the types are correct
    if let PostCommentCombinedView::Comment(v) = &timmy_content[0] {
      assert_eq!(data.timmy_comment.id, v.comment.id);
      assert_eq!(data.timmy.id, v.creator.id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Post(v) = &timmy_content[1] {
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Post(v) = &timmy_content[2] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }

    // Do a batch read of sara
    let sara_content = PersonContentCombinedQuery::new(data.sara.id)
      .list(pool, None, data.instance.id)
      .await?;
    assert_eq!(3, sara_content.len());

    // Make sure the report types are correct
    if let PostCommentCombinedView::Comment(v) = &sara_content[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.creator.id);
      // This one was to timmy_post_2
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Comment(v) = &sara_content[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Post(v) = &sara_content[2] {
      assert_eq!(data.sara_post.id, v.post.id);
      assert_eq!(data.sara.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
