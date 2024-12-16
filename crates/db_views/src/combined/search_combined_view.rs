use crate::structs::{
  CommentView,
  CommunityView,
  LocalUserView,
  PersonView,
  PostView,
  SearchCombinedPaginationCursor,
  SearchCombinedView,
  SearchCombinedViewInternal,
};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  aliases::creator_community_actions,
  newtypes::PersonId,
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    community_aggregates,
    image_details,
    local_user,
    person,
    person_actions,
    person_aggregates,
    post,
    post_actions,
    post_aggregates,
    search_combined,
  },
  source::{
    combined::search::{search_combined_keys as key, SearchCombined},
    community::CommunityFollower,
  },
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool},
  InternalToCombinedView,
};
use lemmy_utils::error::LemmyResult;

impl SearchCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &SearchCombinedView) -> SearchCombinedPaginationCursor {
    let (prefix, id) = match view {
      SearchCombinedView::Post(v) => ('P', v.post.id.0),
      SearchCombinedView::Comment(v) => ('C', v.comment.id.0),
      SearchCombinedView::Community(v) => ('O', v.community.id.0),
      SearchCombinedView::Person(v) => ('E', v.person.id.0),
    };
    // hex encoding to prevent ossification
    SearchCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = search_combined::table
      .select(SearchCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
    query = match prefix {
      "P" => query.filter(search_combined::post_id.eq(id)),
      "C" => query.filter(search_combined::comment_id.eq(id)),
      "O" => query.filter(search_combined::community_id.eq(id)),
      "E" => query.filter(search_combined::person_id.eq(id)),
      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

#[derive(Clone)]
pub struct PaginationCursorData(SearchCombined);

#[derive(derive_new::new)]
pub struct SearchCombinedQuery {
  pub creator_id: PersonId,
  #[new(default)]
  pub page_after: Option<PaginationCursorData>,
  #[new(default)]
  pub page_back: Option<bool>,
}

impl SearchCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &Option<LocalUserView>,
  ) -> LemmyResult<Vec<SearchCombinedView>> {
    let my_person_id = user.as_ref().map(|u| u.local_user.person_id);
    let item_creator = person::id;

    let conn = &mut get_conn(pool).await?;

    let item_creator_join = search_combined::person_id
      .eq(item_creator.nullable())
      .or(
        search_combined::comment_id
          .is_not_null()
          .and(comment::creator_id.eq(item_creator)),
      )
      .or(
        search_combined::post_id
          .is_not_null()
          .and(post::creator_id.eq(item_creator)),
      );

    let comment_join = search_combined::comment_id.eq(comment::id.nullable());

    let post_join = search_combined::post_id
      .eq(post::id.nullable())
      .or(comment::post_id.eq(post::id));

    let community_join = search_combined::community_id
      .eq(community::id.nullable())
      .or(post::community_id.eq(community::id));

    // Notes: since the post_id and comment_id are optional columns,
    // many joins must use an OR condition.
    // For example, the creator must be the person table joined to either:
    // - post.creator_id
    // - comment.creator_id
    let query = search_combined::table
      // The comment
      .left_join(comment::table.on(comment_join))
      // The post
      .left_join(post::table.on(post_join))
      // The item creator
      .inner_join(person::table.on(item_creator_join))
      // The community
      .left_join(community::table.on(community_join))
      .left_join(actions_alias(
        creator_community_actions,
        item_creator,
        community::id,
      ))
      .left_join(
        local_user::table.on(
          item_creator
            .eq(local_user::person_id)
            .and(local_user::admin.eq(true)),
        ),
      )
      .left_join(actions(
        community_actions::table,
        my_person_id,
        community::id,
      ))
      .left_join(actions(post_actions::table, my_person_id, post::id))
      .left_join(actions(person_actions::table, my_person_id, item_creator))
      .left_join(
        person_aggregates::table
          .on(search_combined::person_id.eq(person_aggregates::person_id.nullable())),
      )
      .inner_join(post_aggregates::table.on(post::id.eq(post_aggregates::post_id)))
      .left_join(
        comment_aggregates::table
          .on(search_combined::comment_id.eq(comment_aggregates::comment_id.nullable())),
      )
      .left_join(
        community_aggregates::table
          .on(search_combined::community_id.eq(community_aggregates::community_id.nullable())),
      )
      .left_join(actions(comment_actions::table, my_person_id, comment::id))
      .left_join(image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable())))
      .select((
        // Post-specific
        post::all_columns.nullable(),
        post_aggregates::all_columns.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        )
        .nullable(),
        post_actions::saved.nullable().is_not_null(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        image_details::all_columns.nullable(),
        // Comment-specific
        comment::all_columns.nullable(),
        comment_aggregates::all_columns.nullable(),
        comment_actions::saved.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
        // Community-specific
        community::all_columns.nullable(),
        community_aggregates::all_columns.nullable(),
        community_actions::blocked.nullable().is_not_null(),
        CommunityFollower::select_subscribed_type(),
        // Person
        person_aggregates::all_columns.nullable(),
        // // Shared
        person::all_columns,
        local_user::admin.nullable().is_not_null(),
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
      ))
      .into_boxed();

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    // Sorting by published
    query = query
      .then_desc(key::published)
      // Tie breaker
      .then_desc(key::id);

    let res = query.load::<SearchCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(|u| u.map_to_enum()).collect();

    Ok(out)
  }
}

impl InternalToCombinedView for SearchCombinedViewInternal {
  type CombinedView = SearchCombinedView;

  fn map_to_enum(&self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self.clone();

    if let (Some(post), Some(counts), Some(community), Some(unread_comments)) = (
      v.post.clone(),
      v.post_counts,
      v.community.clone(),
      v.post_unread_comments,
    ) {
      Some(SearchCombinedView::Post(PostView {
        post,
        community,
        counts,
        unread_comments,
        creator: v.item_creator,
        creator_banned_from_community: v.item_creator_banned_from_community,
        creator_is_moderator: v.item_creator_is_moderator,
        creator_is_admin: v.item_creator_is_admin,
        creator_blocked: v.item_creator_blocked,
        subscribed: v.subscribed,
        saved: v.post_saved,
        read: v.post_read,
        hidden: v.post_hidden,
        my_vote: v.my_post_vote,
        image_details: v.image_details,
        banned_from_community: v.banned_from_community,
      }))
    } else if let (Some(comment), Some(counts), Some(post), Some(community)) =
      (v.comment, v.comment_counts, v.post, v.community.clone())
    {
      Some(SearchCombinedView::Comment(CommentView {
        comment,
        counts,
        post,
        community,
        creator: v.item_creator,
        creator_banned_from_community: v.item_creator_banned_from_community,
        creator_is_moderator: v.item_creator_is_moderator,
        creator_is_admin: v.item_creator_is_admin,
        creator_blocked: v.item_creator_blocked,
        subscribed: v.subscribed,
        saved: v.comment_saved,
        my_vote: v.my_comment_vote,
        banned_from_community: v.banned_from_community,
      }))
    } else if let (Some(community), Some(counts)) = (v.community, v.community_counts) {
      Some(SearchCombinedView::Community(CommunityView {
        community,
        counts,
        subscribed: v.subscribed,
        blocked: v.community_blocked,
        banned_from_community: v.banned_from_community,
      }))
    } else if let Some(counts) = v.item_creator_counts {
      Some(SearchCombinedView::Person(PersonView {
        person: v.item_creator,
        counts,
        is_admin: v.item_creator_is_admin,
      }))
    } else {
      None
    }
  }
}

// #[cfg(test)]
// #[expect(clippy::indexing_slicing)]
// mod tests {

//   use crate::{
//     combined::search_combined_view::SearchCombinedQuery,
//     structs::SearchCombinedView,
//   };
//   use lemmy_db_schema::{
//     source::{
//       comment::{Comment, CommentInsertForm},
//       community::{Community, CommunityInsertForm},
//       instance::Instance,
//       person::{Person, PersonInsertForm},
//       post::{Post, PostInsertForm},
//     },
//     traits::Crud,
//     utils::{build_db_pool_for_tests, DbPool},
//   };
//   use lemmy_utils::error::LemmyResult;
//   use pretty_assertions::assert_eq;
//   use serial_test::serial;

//   struct Data {
//     instance: Instance,
//     timmy: Person,
//     sara: Person,
//     timmy_post: Post,
//     timmy_post_2: Post,
//     sara_post: Post,
//     timmy_comment: Comment,
//     sara_comment: Comment,
//     sara_comment_2: Comment,
//   }

//   async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
//     let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

//     let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
//     let timmy = Person::create(pool, &timmy_form).await?;

//     let sara_form = PersonInsertForm::test_form(instance.id, "sara_pcv");
//     let sara = Person::create(pool, &sara_form).await?;

//     let community_form = CommunityInsertForm::new(
//       instance.id,
//       "test community pcv".to_string(),
//       "nada".to_owned(),
//       "pubkey".to_string(),
//     );
//     let community = Community::create(pool, &community_form).await?;

//     let timmy_post_form = PostInsertForm::new("timmy post prv".into(), timmy.id, community.id);
//     let timmy_post = Post::create(pool, &timmy_post_form).await?;

//     let timmy_post_form_2 = PostInsertForm::new("timmy post prv 2".into(), timmy.id,
// community.id);     let timmy_post_2 = Post::create(pool, &timmy_post_form_2).await?;

//     let sara_post_form = PostInsertForm::new("sara post prv".into(), sara.id, community.id);
//     let sara_post = Post::create(pool, &sara_post_form).await?;

//     let timmy_comment_form =
//       CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv".into());
//     let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

//     let sara_comment_form =
//       CommentInsertForm::new(sara.id, timmy_post.id, "sara comment prv".into());
//     let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

//     let sara_comment_form_2 =
//       CommentInsertForm::new(sara.id, timmy_post_2.id, "sara comment prv 2".into());
//     let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

//     Ok(Data {
//       instance,
//       timmy,
//       sara,
//       timmy_post,
//       timmy_post_2,
//       sara_post,
//       timmy_comment,
//       sara_comment,
//       sara_comment_2,
//     })
//   }

//   async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
//     Instance::delete(pool, data.instance.id).await?;

//     Ok(())
//   }

//   #[tokio::test]
//   #[serial]
//   async fn test_combined() -> LemmyResult<()> {
//     let pool = &build_db_pool_for_tests();
//     let pool = &mut pool.into();
//     let data = init_data(pool).await?;

//     // Do a batch read of timmy
//     let timmy_content = SearchCombinedQuery::new(data.timmy.id)
//       .list(pool, &None)
//       .await?;
//     assert_eq!(3, timmy_content.len());

//     // Make sure the types are correct
//     if let SearchCombinedView::Comment(v) = &timmy_content[0] {
//       assert_eq!(data.timmy_comment.id, v.comment.id);
//       assert_eq!(data.timmy.id, v.creator.id);
//     } else {
//       panic!("wrong type");
//     }
//     if let SearchCombinedView::Post(v) = &timmy_content[1] {
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let SearchCombinedView::Post(v) = &timmy_content[2] {
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     // Do a batch read of sara
//     let sara_content = SearchCombinedQuery::new(data.sara.id)
//       .list(pool, &None)
//       .await?;
//     assert_eq!(3, sara_content.len());

//     // Make sure the report types are correct
//     if let SearchCombinedView::Comment(v) = &sara_content[0] {
//       assert_eq!(data.sara_comment_2.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       // This one was to timmy_post_2
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let SearchCombinedView::Comment(v) = &sara_content[1] {
//       assert_eq!(data.sara_comment.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let SearchCombinedView::Post(v) = &sara_content[2] {
//       assert_eq!(data.sara_post.id, v.post.id);
//       assert_eq!(data.sara.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     cleanup(data, pool).await?;

//     Ok(())
//   }
// }
