use crate::structs::{
  LocalUserView,
  PersonContentCombinedView,
  PersonContentCombinedViewInternal,
  PersonSavedCombinedPaginationCursor,
};
use diesel::{
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  aliases::{creator_community_actions, creator_local_user},
  impls::{community::community_follower_select_subscribed_type, local_user::local_user_can_mod},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    image_details,
    local_user,
    person,
    person_actions,
    person_saved_combined,
    post,
    post_actions,
    post_aggregates,
    post_tag,
    tag,
  },
  source::combined::person_saved::{person_saved_combined_keys as key, PersonSavedCombined},
  traits::InternalToCombinedView,
  utils::{functions::coalesce, get_conn, DbPool},
  PersonContentType,
};
use lemmy_utils::error::LemmyResult;

impl PersonSavedCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &PersonContentCombinedView) -> PersonSavedCombinedPaginationCursor {
    let (prefix, id) = match view {
      PersonContentCombinedView::Comment(v) => ('C', v.comment.id.0),
      PersonContentCombinedView::Post(v) => ('P', v.post.id.0),
    };
    // hex encoding to prevent ossification
    PersonSavedCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = person_saved_combined::table
      .select(PersonSavedCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
    query = match prefix {
      "C" => query.filter(person_saved_combined::comment_id.eq(id)),
      "P" => query.filter(person_saved_combined::post_id.eq(id)),
      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

#[derive(Clone)]
pub struct PaginationCursorData(PersonSavedCombined);

#[derive(Default)]
pub struct PersonSavedCombinedQuery {
  pub type_: Option<PersonContentType>,
  pub page_after: Option<PaginationCursorData>,
  pub page_back: Option<bool>,
}

impl PersonSavedCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<PersonContentCombinedView>> {
    let my_person_id = user.local_user.person_id;

    let conn = &mut get_conn(pool).await?;

    let post_tags = post_tag::table
      .inner_join(tag::table)
      .select(diesel::dsl::sql::<diesel::sql_types::Json>(
        "json_agg(tag.*)",
      ))
      .filter(post_tag::post_id.eq(post::id))
      .filter(tag::deleted.eq(false))
      .single_value();

    let mut query = PersonContentCombinedViewInternal::joins_saved(my_person_id)
      .filter(person_saved_combined::person_id.eq(my_person_id))
      .select((
        // Post-specific
        post_aggregates::all_columns,
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        ),
        post_actions::saved.nullable(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        image_details::all_columns.nullable(),
        post_tags,
        // Comment-specific
        comment::all_columns.nullable(),
        comment_aggregates::all_columns.nullable(),
        comment_actions::saved.nullable(),
        comment_actions::like_score.nullable(),
        // Shared
        post::all_columns,
        community::all_columns,
        person::all_columns,
        community_follower_select_subscribed_type(),
        creator_local_user
          .field(local_user::admin)
          .nullable()
          .is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        community_actions::received_ban.nullable().is_not_null(),
        local_user_can_mod(),
      ))
      .into_boxed();

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_saved_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_saved_combined::post_id.is_not_null()),
      }
    }

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    // Sorting by saved desc
    query = query
      .then_desc(key::saved)
      // Tie breaker
      .then_desc(key::id);

    let res = query
      .load::<PersonContentCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{
    combined::person_saved_combined_view::PersonSavedCombinedQuery,
    structs::{LocalUserView, PersonContentCombinedView},
  };
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm, CommentSaved, CommentSavedForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostSaved, PostSavedForm},
    },
    traits::{Crud, Saveable},
    utils::{build_db_pool_for_tests, DbPool},
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
      local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
      person: timmy.clone(),
      counts: Default::default(),
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
      CommentSavedForm::new(data.sara_comment_2.id, data.timmy_view.person.id);
    CommentSaved::save(pool, &save_sara_comment_2).await?;

    let save_sara_comment = CommentSavedForm::new(data.sara_comment.id, data.timmy_view.person.id);
    CommentSaved::save(pool, &save_sara_comment).await?;

    let post_save_form = PostSavedForm::new(data.timmy_post.id, data.timmy.id);
    PostSaved::save(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(3, timmy_saved.len());

    // Make sure the types and order are correct
    if let PersonContentCombinedView::Post(v) = &timmy_saved[0] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PersonContentCombinedView::Comment(v) = &timmy_saved[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PersonContentCombinedView::Comment(v) = &timmy_saved[2] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    // Try unsaving 2 things
    CommentSaved::unsave(pool, &save_sara_comment).await?;
    PostSaved::unsave(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(1, timmy_saved.len());

    if let PersonContentCombinedView::Comment(v) = &timmy_saved[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
