use crate::structs::{
  CommentReplyView,
  InboxCombinedPaginationCursor,
  InboxCombinedView,
  InboxCombinedViewInternal,
  PersonCommentMentionView,
  PersonPostMentionView,
  PrivateMessageView,
};
use diesel::{
  dsl::not,
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
  aliases::{self, creator_community_actions},
  newtypes::PersonId,
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    comment_reply,
    community,
    community_actions,
    image_details,
    inbox_combined,
    instance_actions,
    local_user,
    person,
    person_actions,
    person_comment_mention,
    person_post_mention,
    post,
    post_actions,
    post_aggregates,
    private_message,
  },
  source::{
    combined::inbox::{inbox_combined_keys as key, InboxCombined},
    community::CommunityFollower,
  },
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool},
  InternalToCombinedView,
};
use lemmy_utils::error::LemmyResult;

impl InboxCombinedViewInternal {
  /// Gets the number of unread mentions
  // TODO need to test this
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    show_bot_accounts: bool,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let unread_filter = comment_reply::read
      .eq(false)
      .or(person_comment_mention::read.eq(false))
      .or(person_post_mention::read.eq(false))
      // If its unread, I only want the messages to me
      .or(
        private_message::read
          .eq(false)
          .and(private_message::recipient_id.eq(my_person_id)),
      );

    let item_creator_join = comment::creator_id
      .eq(item_creator)
      .or(
        inbox_combined::person_post_mention_id
          .is_not_null()
          .and(post::creator_id.eq(item_creator)),
      )
      .or(private_message::creator_id.eq(item_creator));

    let recipient_join = comment_reply::recipient_id
      .eq(recipient_person)
      .or(person_comment_mention::recipient_id.eq(recipient_person))
      .or(person_post_mention::recipient_id.eq(recipient_person));

    let comment_join = comment_reply::comment_id
      .eq(comment::id)
      .or(person_comment_mention::comment_id.eq(comment::id));

    let post_join = person_post_mention::post_id
      .eq(post::id)
      .or(comment::post_id.eq(post::id));

    let mut query = inbox_combined::table
      .left_join(comment_reply::table)
      .left_join(person_comment_mention::table)
      .left_join(person_post_mention::table)
      .left_join(private_message::table)
      .left_join(comment::table.on(comment_join))
      .left_join(post::table.on(post_join))
      // The item creator
      .inner_join(person::table.on(item_creator_join))
      // The recipient
      .inner_join(aliases::person1.on(recipient_join))
      .left_join(actions(
        instance_actions::table,
        Some(my_person_id),
        person::instance_id,
      ))
      .left_join(actions(
        person_actions::table,
        Some(my_person_id),
        item_creator,
      ))
      // Filter for your user
      .filter(recipient_person.eq(my_person_id))
      // Filter unreads
      .filter(unread_filter)
      // Don't count replies from blocked users
      .filter(person_actions::blocked.is_null())
      .filter(instance_actions::blocked.is_null())
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(private_message::deleted.eq(false))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !show_bot_accounts {
      query = query.filter(not(person::bot_account));
    }

    query
      .select(count(inbox_combined::id))
      .first::<i64>(conn)
      .await
  }
}

impl InboxCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &InboxCombinedView) -> InboxCombinedPaginationCursor {
    let (prefix, id) = match view {
      InboxCombinedView::CommentReply(v) => ('R', v.comment_reply.id.0),
      InboxCombinedView::CommentMention(v) => ('C', v.person_comment_mention.id.0),
      InboxCombinedView::PostMention(v) => ('P', v.person_post_mention.id.0),
      InboxCombinedView::PrivateMessage(v) => ('M', v.private_message.id.0),
    };
    // hex encoding to prevent ossification
    InboxCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = inbox_combined::table
      .select(InboxCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
    query = match prefix {
      "R" => query.filter(inbox_combined::comment_reply_id.eq(id)),
      "C" => query.filter(inbox_combined::person_comment_mention_id.eq(id)),
      "P" => query.filter(inbox_combined::person_post_mention_id.eq(id)),
      "M" => query.filter(inbox_combined::private_message_id.eq(id)),
      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

#[derive(Clone)]
pub struct PaginationCursorData(InboxCombined);

#[derive(derive_new::new)]
pub struct InboxCombinedQuery {
  pub my_person_id: PersonId,
  #[new(default)]
  pub unread_only: Option<bool>,
  #[new(default)]
  pub show_bot_accounts: Option<bool>,
  #[new(default)]
  pub page_after: Option<PaginationCursorData>,
  #[new(default)]
  pub page_back: Option<bool>,
}

impl InboxCombinedQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<InboxCombinedView>> {
    let conn = &mut get_conn(pool).await?;

    let my_person_id = Some(self.my_person_id);
    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let item_creator_join = comment::creator_id
      .eq(item_creator)
      .or(
        inbox_combined::person_post_mention_id
          .is_not_null()
          .and(post::creator_id.eq(item_creator)),
      )
      .or(private_message::creator_id.eq(item_creator));

    let recipient_join = comment_reply::recipient_id
      .eq(recipient_person)
      .or(person_comment_mention::recipient_id.eq(recipient_person))
      .or(person_post_mention::recipient_id.eq(recipient_person));
    // TODO this might need fixing, because if its not unread, you want all pms, even the ones you
    // sent
    // .or(private_message::recipient_id.eq(recipient_person));

    let comment_join = comment_reply::comment_id
      .eq(comment::id)
      .or(person_comment_mention::comment_id.eq(comment::id));

    let post_join = person_post_mention::post_id
      .eq(post::id)
      .or(comment::post_id.eq(post::id));

    let community_join = post::id.eq(community::id);

    let mut query = inbox_combined::table
      .left_join(comment_reply::table)
      .left_join(person_comment_mention::table)
      .left_join(person_post_mention::table)
      .left_join(private_message::table)
      .left_join(comment::table.on(comment_join))
      .left_join(post::table.on(post_join))
      .left_join(community::table.on(community_join))
      // The item creator
      .inner_join(person::table.on(item_creator_join))
      // The recipient
      .inner_join(aliases::person1.on(recipient_join))
      .left_join(actions_alias(
        creator_community_actions,
        item_creator,
        post::community_id,
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
        post::community_id,
      ))
      .left_join(actions(
        instance_actions::table,
        my_person_id,
        person::instance_id,
      ))
      .left_join(actions(post_actions::table, my_person_id, post::id))
      .left_join(actions(person_actions::table, my_person_id, item_creator))
      .left_join(post_aggregates::table.on(post::id.eq(post_aggregates::post_id)))
      .left_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(actions(comment_actions::table, my_person_id, comment::id))
      .left_join(image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable())))
      // The recipient filter (IE only show replies to you)
      .filter(recipient_person.eq(self.my_person_id))
      .select((
        // Specific
        comment_reply::all_columns.nullable(),
        person_comment_mention::all_columns.nullable(),
        person_post_mention::all_columns.nullable(),
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
        private_message::all_columns.nullable(),
        // Shared
        post::all_columns.nullable(),
        community::all_columns.nullable(),
        comment::all_columns.nullable(),
        comment_aggregates::all_columns.nullable(),
        comment_actions::saved.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
        CommunityFollower::select_subscribed_type(),
        person::all_columns,
        aliases::person1.fields(person::all_columns),
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

    // Filters
    if self.unread_only.unwrap_or_default() {
      query = query.filter(
        comment_reply::read
          .eq(false)
          .or(person_comment_mention::read.eq(false))
          .or(person_post_mention::read.eq(false))
          // If its unread, I only want the messages to me
          .or(
            private_message::read
              .eq(false)
              .and(private_message::recipient_id.eq(self.my_person_id)),
          ),
      );
    }

    if !(self.show_bot_accounts.unwrap_or_default()) {
      query = query.filter(not(person::bot_account));
    };

    // Dont show replies from blocked users or instances
    query = query
      .filter(person_actions::blocked.is_null())
      .filter(instance_actions::blocked.is_null());

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

    let res = query.load::<InboxCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(|u| u.map_to_enum()).collect();

    Ok(out)
  }
}

impl InternalToCombinedView for InboxCombinedViewInternal {
  type CombinedView = InboxCombinedView;

  fn map_to_enum(&self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self.clone();

    if let (Some(comment_reply), Some(comment), Some(counts), Some(post), Some(community)) = (
      v.comment_reply,
      v.comment.clone(),
      v.comment_counts.clone(),
      v.post.clone(),
      v.community.clone(),
    ) {
      Some(InboxCombinedView::CommentReply(CommentReplyView {
        comment_reply,
        comment,
        counts,
        recipient: v.item_recipient,
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
    } else if let (
      Some(person_comment_mention),
      Some(comment),
      Some(counts),
      Some(post),
      Some(community),
    ) = (
      v.person_comment_mention,
      v.comment,
      v.comment_counts,
      v.post.clone(),
      v.community.clone(),
    ) {
      Some(InboxCombinedView::CommentMention(
        PersonCommentMentionView {
          person_comment_mention,
          comment,
          counts,
          recipient: v.item_recipient,
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
        },
      ))
    } else if let (
      Some(person_post_mention),
      Some(post),
      Some(counts),
      Some(unread_comments),
      Some(community),
    ) = (
      v.person_post_mention,
      v.post,
      v.post_counts,
      v.post_unread_comments,
      v.community,
    ) {
      Some(InboxCombinedView::PostMention(PersonPostMentionView {
        person_post_mention,
        counts,
        post,
        community,
        recipient: v.item_recipient,
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
    } else if let Some(private_message) = v.private_message {
      Some(InboxCombinedView::PrivateMessage(PrivateMessageView {
        private_message,
        creator: v.item_creator,
        recipient: v.item_recipient,
      }))
    } else {
      None
    }
  }
}

// TODO Dont delete these
// #[cfg(test)]
// #[expect(clippy::indexing_slicing)]
// mod tests {

//   use crate::{inbox_combined_view::InboxCombinedQuery, structs::InboxCombinedView};
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
//     let timmy_content = InboxCombinedQuery::new(data.timmy.id)
//       .list(pool, &None)
//       .await?;
//     assert_eq!(3, timmy_content.len());

//     // Make sure the types are correct
//     if let InboxCombinedView::Comment(v) = &timmy_content[0] {
//       assert_eq!(data.timmy_comment.id, v.comment.id);
//       assert_eq!(data.timmy.id, v.creator.id);
//     } else {
//       panic!("wrong type");
//     }
//     if let InboxCombinedView::Post(v) = &timmy_content[1] {
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let InboxCombinedView::Post(v) = &timmy_content[2] {
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     // Do a batch read of sara
//     let sara_content = InboxCombinedQuery::new(data.sara.id)
//       .list(pool, &None)
//       .await?;
//     assert_eq!(3, sara_content.len());

//     // Make sure the report types are correct
//     if let InboxCombinedView::Comment(v) = &sara_content[0] {
//       assert_eq!(data.sara_comment_2.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       // This one was to timmy_post_2
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let InboxCombinedView::Comment(v) = &sara_content[1] {
//       assert_eq!(data.sara_comment.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let InboxCombinedView::Post(v) = &sara_content[2] {
//       assert_eq!(data.sara_post.id, v.post.id);
//       assert_eq!(data.sara.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     cleanup(data, pool).await?;

//     Ok(())
//   }
// }

//

// #[cfg(test)]
// mod tests {

//   use crate::{
//     person_comment_mention_view::PersonCommentMentionQuery,
//     structs::PersonCommentMentionView,
//   };
//   use lemmy_db_schema::{
//     source::{
//       comment::{Comment, CommentInsertForm},
//       community::{Community, CommunityInsertForm},
//       instance::Instance,
//       local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
//       person::{Person, PersonInsertForm, PersonUpdateForm},
//       person_block::{PersonBlock, PersonBlockForm},
//       person_comment_mention::{
//         PersonCommentMention,
//         PersonCommentMentionInsertForm,
//         PersonCommentMentionUpdateForm,
//       },
//       post::{Post, PostInsertForm},
//     },
//     traits::{Blockable, Crud},
//     utils::build_db_pool_for_tests,
//   };
//   use lemmy_db_views::structs::LocalUserView;
//   use lemmy_utils::error::LemmyResult;
//   use pretty_assertions::assert_eq;
//   use serial_test::serial;

//   #[tokio::test]
//   #[serial]
//   async fn test_crud() -> LemmyResult<()> {
//     let pool = &build_db_pool_for_tests();
//     let pool = &mut pool.into();

//     let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

//     let new_person = PersonInsertForm::test_form(inserted_instance.id, "terrylake");

//     let inserted_person = Person::create(pool, &new_person).await?;

//     let recipient_form = PersonInsertForm::test_form(inserted_instance.id, "terrylakes
// recipient");

//     let inserted_recipient = Person::create(pool, &recipient_form).await?;
//     let recipient_id = inserted_recipient.id;

//     let recipient_local_user =
//       LocalUser::create(pool, &LocalUserInsertForm::test_form(recipient_id), vec![]).await?;

//     let new_community = CommunityInsertForm::new(
//       inserted_instance.id,
//       "test community lake".to_string(),
//       "nada".to_owned(),
//       "pubkey".to_string(),
//     );
//     let inserted_community = Community::create(pool, &new_community).await?;

//     let new_post = PostInsertForm::new(
//       "A test post".into(),
//       inserted_person.id,
//       inserted_community.id,
//     );
//     let inserted_post = Post::create(pool, &new_post).await?;

//     let comment_form = CommentInsertForm::new(
//       inserted_person.id,
//       inserted_post.id,
//       "A test comment".into(),
//     );
//     let inserted_comment = Comment::create(pool, &comment_form, None).await?;

//     let person_comment_mention_form = PersonCommentMentionInsertForm {
//       recipient_id: inserted_recipient.id,
//       comment_id: inserted_comment.id,
//       read: None,
//     };

//     let inserted_mention = PersonCommentMention::create(pool,
// &person_comment_mention_form).await?;

//     let expected_mention = PersonCommentMention {
//       id: inserted_mention.id,
//       recipient_id: inserted_mention.recipient_id,
//       comment_id: inserted_mention.comment_id,
//       read: false,
//       published: inserted_mention.published,
//     };

//     let read_mention = PersonCommentMention::read(pool, inserted_mention.id).await?;

//     let person_comment_mention_update_form = PersonCommentMentionUpdateForm { read: Some(false)
// };     let updated_mention = PersonCommentMention::update(
//       pool,
//       inserted_mention.id,
//       &person_comment_mention_update_form,
//     )
//     .await?;

//     // Test to make sure counts and blocks work correctly
//     let unread_mentions =
//       PersonCommentMentionView::get_unread_count(pool, &recipient_local_user).await?;

//     let query = PersonCommentMentionQuery {
//       recipient_id: Some(recipient_id),
//       my_person_id: Some(recipient_id),
//       sort: None,
//       unread_only: false,
//       show_bot_accounts: true,
//       page: None,
//       limit: None,
//     };
//     let mentions = query.clone().list(pool).await?;
//     assert_eq!(1, unread_mentions);
//     assert_eq!(1, mentions.len());

//     // Block the person, and make sure these counts are now empty
//     let block_form = PersonBlockForm {
//       person_id: recipient_id,
//       target_id: inserted_person.id,
//     };
//     PersonBlock::block(pool, &block_form).await?;

//     let unread_mentions_after_block =
//       PersonCommentMentionView::get_unread_count(pool, &recipient_local_user).await?;
//     let mentions_after_block = query.clone().list(pool).await?;
//     assert_eq!(0, unread_mentions_after_block);
//     assert_eq!(0, mentions_after_block.len());

//     // Unblock user so we can reuse the same person
//     PersonBlock::unblock(pool, &block_form).await?;

//     // Turn Terry into a bot account
//     let person_update_form = PersonUpdateForm {
//       bot_account: Some(true),
//       ..Default::default()
//     };
//     Person::update(pool, inserted_person.id, &person_update_form).await?;

//     let recipient_local_user_update_form = LocalUserUpdateForm {
//       show_bot_accounts: Some(false),
//       ..Default::default()
//     };
//     LocalUser::update(
//       pool,
//       recipient_local_user.id,
//       &recipient_local_user_update_form,
//     )
//     .await?;
//     let recipient_local_user_view = LocalUserView::read(pool, recipient_local_user.id).await?;

//     let unread_mentions_after_hide_bots =
//       PersonCommentMentionView::get_unread_count(pool, &recipient_local_user_view.local_user)
//         .await?;

//     let mut query_without_bots = query.clone();
//     query_without_bots.show_bot_accounts = false;
//     let replies_after_hide_bots = query_without_bots.list(pool).await?;
//     assert_eq!(0, unread_mentions_after_hide_bots);
//     assert_eq!(0, replies_after_hide_bots.len());

//     Comment::delete(pool, inserted_comment.id).await?;
//     Post::delete(pool, inserted_post.id).await?;
//     Community::delete(pool, inserted_community.id).await?;
//     Person::delete(pool, inserted_person.id).await?;
//     Person::delete(pool, inserted_recipient.id).await?;
//     Instance::delete(pool, inserted_instance.id).await?;

//     assert_eq!(expected_mention, read_mention);
//     assert_eq!(expected_mention, inserted_mention);
//     assert_eq!(expected_mention, updated_mention);

//     Ok(())
//   }
// }
// #[cfg(test)]
// mod tests {

//   use crate::{person_post_mention_view::PersonPostMentionQuery, structs::PersonPostMentionView};
//   use lemmy_db_schema::{
//     source::{
//       community::{Community, CommunityInsertForm},
//       instance::Instance,
//       local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
//       person::{Person, PersonInsertForm, PersonUpdateForm},
//       person_block::{PersonBlock, PersonBlockForm},
//       person_post_mention::{
//         PersonPostMention,
//         PersonPostMentionInsertForm,
//         PersonPostMentionUpdateForm,
//       },
//       post::{Post, PostInsertForm},
//     },
//     traits::{Blockable, Crud},
//     utils::build_db_pool_for_tests,
//   };
//   use lemmy_db_views::structs::LocalUserView;
//   use lemmy_utils::error::LemmyResult;
//   use pretty_assertions::assert_eq;
//   use serial_test::serial;

//   #[tokio::test]
//   #[serial]
//   async fn test_crud() -> LemmyResult<()> {
//     let pool = &build_db_pool_for_tests().await;
//     let pool = &mut pool.into();

//     let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

//     let new_person = PersonInsertForm::test_form(inserted_instance.id, "terrylake");

//     let inserted_person = Person::create(pool, &new_person).await?;

//     let recipient_form = PersonInsertForm::test_form(inserted_instance.id, "terrylakes
// recipient");

//     let inserted_recipient = Person::create(pool, &recipient_form).await?;
//     let recipient_id = inserted_recipient.id;

//     let recipient_local_user =
//       LocalUser::create(pool, &LocalUserInsertForm::test_form(recipient_id), vec![]).await?;

//     let new_community = CommunityInsertForm::new(
//       inserted_instance.id,
//       "test community lake".to_string(),
//       "nada".to_owned(),
//       "pubkey".to_string(),
//     );
//     let inserted_community = Community::create(pool, &new_community).await?;

//     let new_post = PostInsertForm::new(
//       "A test post".into(),
//       inserted_person.id,
//       inserted_community.id,
//     );
//     let inserted_post = Post::create(pool, &new_post).await?;

//     let person_post_mention_form = PersonPostMentionInsertForm {
//       recipient_id: inserted_recipient.id,
//       post_id: inserted_post.id,
//       read: None,
//     };

//     let inserted_mention = PersonPostMention::create(pool, &person_post_mention_form).await?;

//     let expected_mention = PersonPostMention {
//       id: inserted_mention.id,
//       recipient_id: inserted_mention.recipient_id,
//       post_id: inserted_mention.post_id,
//       read: false,
//       published: inserted_mention.published,
//     };

//     let read_mention = PersonPostMention::read(pool, inserted_mention.id).await?;

//     let person_post_mention_update_form = PersonPostMentionUpdateForm { read: Some(false) };
//     let updated_mention =
//       PersonPostMention::update(pool, inserted_mention.id, &person_post_mention_update_form)
//         .await?;

//     // Test to make sure counts and blocks work correctly
//     let unread_mentions =
//       PersonPostMentionView::get_unread_count(pool, &recipient_local_user).await?;

//     let query = PersonPostMentionQuery {
//       recipient_id: Some(recipient_id),
//       my_person_id: Some(recipient_id),
//       sort: None,
//       unread_only: false,
//       show_bot_accounts: true,
//       page: None,
//       limit: None,
//     };
//     let mentions = query.clone().list(pool).await?;
//     assert_eq!(1, unread_mentions);
//     assert_eq!(1, mentions.len());

//     // Block the person, and make sure these counts are now empty
//     let block_form = PersonBlockForm {
//       person_id: recipient_id,
//       target_id: inserted_person.id,
//     };
//     PersonBlock::block(pool, &block_form).await?;

//     let unread_mentions_after_block =
//       PersonPostMentionView::get_unread_count(pool, &recipient_local_user).await?;
//     let mentions_after_block = query.clone().list(pool).await?;
//     assert_eq!(0, unread_mentions_after_block);
//     assert_eq!(0, mentions_after_block.len());

//     // Unblock user so we can reuse the same person
//     PersonBlock::unblock(pool, &block_form).await?;

//     // Turn Terry into a bot account
//     let person_update_form = PersonUpdateForm {
//       bot_account: Some(true),
//       ..Default::default()
//     };
//     Person::update(pool, inserted_person.id, &person_update_form).await?;

//     let recipient_local_user_update_form = LocalUserUpdateForm {
//       show_bot_accounts: Some(false),
//       ..Default::default()
//     };
//     LocalUser::update(
//       pool,
//       recipient_local_user.id,
//       &recipient_local_user_update_form,
//     )
//     .await?;
//     let recipient_local_user_view = LocalUserView::read(pool, recipient_local_user.id).await?;

//     let unread_mentions_after_hide_bots =
//       PersonPostMentionView::get_unread_count(pool,
// &recipient_local_user_view.local_user).await?;

//     let mut query_without_bots = query.clone();
//     query_without_bots.show_bot_accounts = false;
//     let replies_after_hide_bots = query_without_bots.list(pool).await?;
//     assert_eq!(0, unread_mentions_after_hide_bots);
//     assert_eq!(0, replies_after_hide_bots.len());

//     Post::delete(pool, inserted_post.id).await?;
//     Post::delete(pool, inserted_post.id).await?;
//     Community::delete(pool, inserted_community.id).await?;
//     Person::delete(pool, inserted_person.id).await?;
//     Person::delete(pool, inserted_recipient.id).await?;
//     Instance::delete(pool, inserted_instance.id).await?;

//     assert_eq!(expected_mention, read_mention);
//     assert_eq!(expected_mention, inserted_mention);
//     assert_eq!(expected_mention, updated_mention);

//     Ok(())
//   }
// }
