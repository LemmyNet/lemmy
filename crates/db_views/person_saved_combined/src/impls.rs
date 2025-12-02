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
  PersonContentType,
  source::combined::person_saved::{PersonSavedCombined, person_saved_combined_keys as key},
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
  schema::{comment, person, person_saved_combined, post},
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

#[derive(Default)]
pub struct PersonSavedCombinedQuery {
  pub type_: Option<PersonContentType>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct PostCommentCombinedViewWrapper(PostCommentCombinedView);

impl PaginationCursorConversion for PostCommentCombinedViewWrapper {
  type PaginatedType = PersonSavedCombined;

  fn to_cursor(&self) -> CursorData {
    let (prefix, id) = match &self.0 {
      PostCommentCombinedView::Comment(v) => ('C', v.comment.id.0),
      PostCommentCombinedView::Post(v) => ('P', v.post.id.0),
    };
    CursorData::new_with_prefix(prefix, id)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let (prefix, id) = cursor.id_and_prefix()?;

    let mut query = person_saved_combined::table
      .select(Self::PaginatedType::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(person_saved_combined::comment_id.eq(id)),
      'P' => query.filter(person_saved_combined::post_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

impl PersonSavedCombinedQuery {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let comment_join =
      comment::table.on(person_saved_combined::comment_id.eq(comment::id.nullable()));

    let post_join = post::table.on(
      person_saved_combined::post_id
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
            .and(person_saved_combined::post_id.is_not_null()),
        ),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    person_saved_combined::table
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
      .left_join(my_community_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }

  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<PagedResponse<PostCommentCombinedView>> {
    let my_person_id = user.local_user.person_id;
    let local_instance_id = user.person.instance_id;

    let mut query = Self::joins(my_person_id, local_instance_id)
      .filter(person_saved_combined::person_id.eq(my_person_id))
      .select(PostCommentCombinedViewInternal::as_select())
      .into_boxed();

    let limit = limit_fetch(self.limit, self.no_limit)?;
    query = query.limit(limit);

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_saved_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_saved_combined::post_id.is_not_null()),
      }
    }

    // Sorting by saved desc
    let paginated_query = PostCommentCombinedViewWrapper::paginate(
      query,
      &self.page_cursor,
      SortDirection::Desc,
      pool,
      None,
    )
    .await?
    .then_order_by(key::saved_at)
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
      data: res.data.into_iter().map(|i| i.0).collect(),
      next_page: res.next_page,
      prev_page: res.prev_page,
    })
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use crate::{LocalUserView, impls::PersonSavedCombinedQuery};
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentSavedForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostSavedForm},
    },
    traits::Saveable,
  };
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
    timmy_view: LocalUserView,
    sara: Person,
    timmy_post: Post,
    sara_comment: Comment,
    sara_comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: timmy.clone(),
      banned: false,
      ban_expires_at: None,
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

    // Do a batch read of timmy saved
    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(0, timmy_saved.len());

    // Save a few things
    let save_sara_comment_2 =
      CommentSavedForm::new(data.timmy_view.person.id, data.sara_comment_2.id);
    CommentActions::save(pool, &save_sara_comment_2).await?;

    let save_sara_comment = CommentSavedForm::new(data.timmy_view.person.id, data.sara_comment.id);
    CommentActions::save(pool, &save_sara_comment).await?;

    let post_save_form = PostSavedForm::new(data.timmy_post.id, data.timmy.id);
    PostActions::save(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(3, timmy_saved.len());

    // Make sure the types and order are correct
    if let PostCommentCombinedView::Post(v) = &timmy_saved[0] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Comment(v) = &timmy_saved[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PostCommentCombinedView::Comment(v) = &timmy_saved[2] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    // Try unsaving 2 things
    CommentActions::unsave(pool, &save_sara_comment).await?;
    PostActions::unsave(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(1, timmy_saved.len());

    if let PostCommentCombinedView::Comment(v) = &timmy_saved[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
