use crate::{
  CommentReplyView,
  InboxCombinedView,
  InboxCombinedViewInternal,
  PersonCommentMentionView,
  PersonPostMentionView,
};
use diesel::{
  dsl::not,
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
  aliases::{self},
  newtypes::{InstanceId, PaginationCursor, PersonId},
  source::combined::inbox::{inbox_combined_keys as key, InboxCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      community_join,
      creator_community_actions_join,
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
  InboxDataType,
};
use lemmy_db_schema_file::schema::{
  comment,
  comment_reply,
  inbox_combined,
  instance_actions,
  person,
  person_actions,
  person_comment_mention,
  person_post_mention,
  post,
  private_message,
};
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl InboxCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        .or(
          inbox_combined::person_post_mention_id
            .is_not_null()
            .and(post::creator_id.eq(item_creator)),
        )
        .or(private_message::creator_id.eq(item_creator)),
    );

    let recipient_join = aliases::person1.on(
      comment_reply::recipient_id
        .eq(recipient_person)
        .or(person_comment_mention::recipient_id.eq(recipient_person))
        .or(person_post_mention::recipient_id.eq(recipient_person))
        .or(private_message::recipient_id.eq(recipient_person)),
    );

    let comment_join = comment::table.on(
      comment_reply::comment_id
        .eq(comment::id)
        .or(person_comment_mention::comment_id.eq(comment::id))
        // Filter out the deleted / removed
        .and(not(comment::deleted))
        .and(not(comment::removed)),
    );

    let post_join = post::table.on(
      person_post_mention::post_id
        .eq(post::id)
        .or(comment::post_id.eq(post::id))
        // Filter out the deleted / removed
        .and(not(post::deleted))
        .and(not(post::removed)),
    );

    // This could be a simple join, but you need to check for deleted here
    let private_message_join = private_message::table.on(
      inbox_combined::private_message_id
        .eq(private_message::id.nullable())
        .and(not(private_message::deleted))
        .and(not(private_message::removed)),
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

    inbox_combined::table
      .left_join(comment_reply::table)
      .left_join(person_comment_mention::table)
      .left_join(person_post_mention::table)
      .left_join(private_message_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(community_join())
      .inner_join(item_creator_join)
      .inner_join(recipient_join)
      .left_join(image_details_join())
      .left_join(creator_community_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_community_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
    show_bot_accounts: bool,
  ) -> LemmyResult<i64> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

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

    let mut query = Self::joins(my_person_id, local_instance_id)
      // Filter for your user
      .filter(recipient_person.eq(my_person_id))
      // Filter unreads
      .filter(unread_filter)
      // Don't count replies from blocked users
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null())
      .select(count(inbox_combined::id))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !show_bot_accounts {
      query = query.filter(not(person::bot_account));
    }

    query
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for InboxCombinedView {
  type CursorData = InboxCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      InboxCombinedView::CommentReply(v) => ('R', v.comment_reply.id.0),
      InboxCombinedView::CommentMention(v) => ('C', v.person_comment_mention.id.0),
      InboxCombinedView::PostMention(v) => ('P', v.person_post_mention.id.0),
      InboxCombinedView::PrivateMessage(v) => ('M', v.private_message.id.0),
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

    let mut query = inbox_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'R' => query.filter(inbox_combined::comment_reply_id.eq(id)),
      'C' => query.filter(inbox_combined::person_comment_mention_id.eq(id)),
      'P' => query.filter(inbox_combined::person_post_mention_id.eq(id)),
      'M' => query.filter(inbox_combined::private_message_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct InboxCombinedQuery {
  pub type_: Option<InboxDataType>,
  pub unread_only: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub cursor_data: Option<InboxCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl InboxCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Vec<InboxCombinedView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let mut query = InboxCombinedViewInternal::joins(my_person_id, local_instance_id)
      .select(InboxCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    // Filters
    if self.unread_only.unwrap_or_default() {
      query = query
        // The recipient filter (IE only show replies to you)
        .filter(recipient_person.eq(my_person_id))
        .filter(
          comment_reply::read
            .eq(false)
            .or(person_comment_mention::read.eq(false))
            .or(person_post_mention::read.eq(false))
            // If its unread, I only want the messages to me
            .or(private_message::read.eq(false)),
        );
    } else {
      // A special case for private messages: show messages FROM you also.
      // Use a not-null checks to catch the others
      query = query.filter(
        inbox_combined::comment_reply_id
          .is_not_null()
          .and(recipient_person.eq(my_person_id))
          .or(
            inbox_combined::person_comment_mention_id
              .is_not_null()
              .and(recipient_person.eq(my_person_id)),
          )
          .or(
            inbox_combined::person_post_mention_id
              .is_not_null()
              .and(recipient_person.eq(my_person_id)),
          )
          .or(
            inbox_combined::private_message_id.is_not_null().and(
              recipient_person
                .eq(my_person_id)
                .or(item_creator.eq(my_person_id)),
            ),
          ),
      );
    }

    if !(self.show_bot_accounts.unwrap_or_default()) {
      query = query.filter(not(person::bot_account));
    };

    // Dont show replies from blocked users or instances
    query = query
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null());

    if let Some(type_) = self.type_ {
      query = match type_ {
        InboxDataType::All => query,
        InboxDataType::CommentReply => query.filter(inbox_combined::comment_reply_id.is_not_null()),
        InboxDataType::CommentMention => {
          query.filter(inbox_combined::person_comment_mention_id.is_not_null())
        }
        InboxDataType::PostMention => {
          query.filter(inbox_combined::person_post_mention_id.is_not_null())
        }
        InboxDataType::PrivateMessage => {
          query.filter(inbox_combined::private_message_id.is_not_null())
        }
      }
    }

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

    let res = paginated_query
      .load::<InboxCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for InboxCombinedViewInternal {
  type CombinedView = InboxCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(comment_reply), Some(comment), Some(post), Some(community)) = (
      v.comment_reply,
      v.comment.clone(),
      v.post.clone(),
      v.community.clone(),
    ) {
      Some(InboxCombinedView::CommentReply(CommentReplyView {
        comment_reply,
        comment,
        recipient: v.item_recipient,
        post,
        community,
        creator: v.item_creator,
        community_actions: v.community_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_home_instance_actions: v.creator_home_instance_actions,
        creator_local_instance_actions: v.creator_local_instance_actions,
        creator_community_actions: v.creator_community_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
      }))
    } else if let (Some(person_comment_mention), Some(comment), Some(post), Some(community)) = (
      v.person_comment_mention,
      v.comment,
      v.post.clone(),
      v.community.clone(),
    ) {
      Some(InboxCombinedView::CommentMention(
        PersonCommentMentionView {
          person_comment_mention,
          comment,
          recipient: v.item_recipient,
          post,
          community,
          creator: v.item_creator,
          community_actions: v.community_actions,
          comment_actions: v.comment_actions,
          person_actions: v.person_actions,
          instance_actions: v.instance_actions,
          creator_home_instance_actions: v.creator_home_instance_actions,
          creator_local_instance_actions: v.creator_local_instance_actions,
          creator_community_actions: v.creator_community_actions,
          creator_is_admin: v.item_creator_is_admin,
          can_mod: v.can_mod,
          creator_banned: v.creator_banned,
        },
      ))
    } else if let (Some(person_post_mention), Some(post), Some(community)) =
      (v.person_post_mention, v.post, v.community)
    {
      Some(InboxCombinedView::PostMention(PersonPostMentionView {
        person_post_mention,
        post,
        community,
        creator: v.item_creator,
        recipient: v.item_recipient,
        community_actions: v.community_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_home_instance_actions: v.creator_home_instance_actions,
        creator_local_instance_actions: v.creator_local_instance_actions,
        post_actions: v.post_actions,
        image_details: v.image_details,
        creator_community_actions: v.creator_community_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
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

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use crate::{impls::InboxCombinedQuery, InboxCombinedView, InboxCombinedViewInternal};
  use lemmy_db_schema::{
    assert_length,
    source::{
      comment::{Comment, CommentInsertForm},
      comment_reply::{CommentReply, CommentReplyInsertForm, CommentReplyUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::{Instance, InstanceActions, InstanceBlockForm},
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm, PersonUpdateForm},
      person_comment_mention::{PersonCommentMention, PersonCommentMentionInsertForm},
      person_post_mention::{PersonPostMention, PersonPostMentionInsertForm},
      post::{Post, PostInsertForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
    },
    traits::{Blockable, Crud},
    utils::{build_db_pool_for_tests, DbPool},
    InboxDataType,
  };
  use lemmy_db_views_private_message::PrivateMessageView;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    timmy_post: Post,
    jessica_post: Post,
    timmy_comment: Comment,
    sara_comment: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_pcv");
    let sara = Person::create(pool, &sara_form).await?;

    let jessica_form = PersonInsertForm::test_form(instance.id, "jessica_mrv");
    let jessica = Person::create(pool, &jessica_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community pcv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let timmy_post_form = PostInsertForm::new("timmy post prv".into(), timmy.id, community.id);
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let jessica_post_form =
      PostInsertForm::new("jessica post prv".into(), jessica.id, community.id);
    let jessica_post = Post::create(pool, &jessica_post_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv".into());
    let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.id, timmy_post.id, "sara comment prv".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, Some(&timmy_comment.path)).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      timmy_post,
      jessica_post,
      timmy_comment,
      sara_comment,
    })
  }

  async fn setup_private_messages(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    let sara_timmy_message_form =
      PrivateMessageInsertForm::new(data.sara.id, data.timmy.id, "sara to timmy".into());
    PrivateMessage::create(pool, &sara_timmy_message_form).await?;

    let sara_jessica_message_form =
      PrivateMessageInsertForm::new(data.sara.id, data.jessica.id, "sara to jessica".into());
    PrivateMessage::create(pool, &sara_jessica_message_form).await?;

    let timmy_sara_message_form =
      PrivateMessageInsertForm::new(data.timmy.id, data.sara.id, "timmy to sara".into());
    PrivateMessage::create(pool, &timmy_sara_message_form).await?;

    let jessica_timmy_message_form =
      PrivateMessageInsertForm::new(data.jessica.id, data.timmy.id, "jessica to timmy".into());
    PrivateMessage::create(pool, &jessica_timmy_message_form).await?;

    Ok(())
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn replies() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Sara replied to timmys comment, but lets create the row now
    let form = CommentReplyInsertForm {
      recipient_id: data.timmy.id,
      comment_id: data.sara_comment.id,
      read: None,
    };
    let reply = CommentReply::create(pool, &form).await?;

    let timmy_unread_replies =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, true)
        .await?;
    assert_eq!(1, timmy_unread_replies);

    let timmy_inbox = InboxCombinedQuery::default()
      .list(pool, data.timmy.id, data.instance.id)
      .await?;
    assert_length!(1, timmy_inbox);

    if let InboxCombinedView::CommentReply(v) = &timmy_inbox[0] {
      assert_eq!(data.sara_comment.id, v.comment_reply.comment_id);
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    // Mark it as read
    let form = CommentReplyUpdateForm { read: Some(true) };
    CommentReply::update(pool, reply.id, &form).await?;

    let timmy_unread_replies =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, true)
        .await?;
    assert_eq!(0, timmy_unread_replies);

    let timmy_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.timmy.id, data.instance.id)
    .await?;
    assert_length!(0, timmy_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mentions() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Timmy mentions sara in a comment
    let timmy_mention_sara_comment_form = PersonCommentMentionInsertForm {
      recipient_id: data.sara.id,
      comment_id: data.timmy_comment.id,
      read: None,
    };
    PersonCommentMention::create(pool, &timmy_mention_sara_comment_form).await?;

    // Jessica mentions sara in a post
    let jessica_mention_sara_post_form = PersonPostMentionInsertForm {
      recipient_id: data.sara.id,
      post_id: data.jessica_post.id,
      read: None,
    };
    PersonPostMention::create(pool, &jessica_mention_sara_post_form).await?;

    // Test to make sure counts and blocks work correctly
    let sara_unread_mentions =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, true)
        .await?;
    assert_eq!(2, sara_unread_mentions);

    let sara_inbox = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(2, sara_inbox);

    if let InboxCombinedView::PostMention(v) = &sara_inbox[0] {
      assert_eq!(data.jessica_post.id, v.person_post_mention.post_id);
      assert_eq!(data.jessica_post.id, v.post.id);
      assert_eq!(data.jessica.id, v.creator.id);
      assert_eq!(data.sara.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    if let InboxCombinedView::CommentMention(v) = &sara_inbox[1] {
      assert_eq!(data.timmy_comment.id, v.person_comment_mention.comment_id);
      assert_eq!(data.timmy_comment.id, v.comment.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.creator.id);
      assert_eq!(data.sara.id, v.recipient.id);
    } else {
      panic!("wrong type");
    }

    // Sara blocks timmy, and make sure these counts are now empty
    let sara_blocks_timmy_form = PersonBlockForm::new(data.sara.id, data.timmy.id);
    PersonActions::block(pool, &sara_blocks_timmy_form).await?;

    let sara_unread_mentions_after_block =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, true)
        .await?;
    assert_eq!(1, sara_unread_mentions_after_block);

    let sara_inbox_after_block = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_block);

    // Make sure the comment mention which timmy made is the hidden one
    assert!(matches!(
      sara_inbox_after_block[0],
      InboxCombinedView::PostMention(_)
    ));

    // Unblock user so we can reuse the same person
    PersonActions::unblock(pool, &sara_blocks_timmy_form).await?;

    // Test the type filter
    let sara_inbox_post_mentions_only = InboxCombinedQuery {
      type_: Some(InboxDataType::PostMention),
      ..Default::default()
    }
    .list(pool, data.sara.id, data.instance.id)
    .await?;
    assert_length!(1, sara_inbox_post_mentions_only);

    assert!(matches!(
      sara_inbox_post_mentions_only[0],
      InboxCombinedView::PostMention(_)
    ));

    // Turn Jessica into a bot account
    let person_update_form = PersonUpdateForm {
      bot_account: Some(true),
      ..Default::default()
    };
    Person::update(pool, data.jessica.id, &person_update_form).await?;

    // Make sure sara hides bots
    let sara_unread_mentions_after_hide_bots =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, false)
        .await?;
    assert_eq!(1, sara_unread_mentions_after_hide_bots);

    let sara_inbox_after_hide_bots = InboxCombinedQuery::default()
      .list(pool, data.sara.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_hide_bots);

    // Make sure the post mention which jessica made is the hidden one
    assert!(matches!(
      sara_inbox_after_hide_bots[0],
      InboxCombinedView::CommentMention(_)
    ));

    // Mark them all as read
    PersonPostMention::mark_all_as_read(pool, data.sara.id).await?;
    PersonCommentMention::mark_all_as_read(pool, data.sara.id).await?;

    // Make sure none come back
    let sara_unread_mentions =
      InboxCombinedViewInternal::get_unread_count(pool, data.sara.id, data.instance.id, false)
        .await?;
    assert_eq!(0, sara_unread_mentions);

    let sara_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.sara.id, data.instance.id)
    .await?;
    assert_length!(0, sara_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  /// A helper function to coerce to a private message type for tests
  fn map_to_pm(inbox: &[InboxCombinedView]) -> Vec<PrivateMessageView> {
    inbox
      .iter()
      // Filter map to collect private messages
      .filter_map(|f| {
        if let InboxCombinedView::PrivateMessage(v) = f {
          Some(v)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<PrivateMessageView>>()
  }

  #[tokio::test]
  #[serial]
  async fn read_private_messages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    let timmy_messages = map_to_pm(
      &InboxCombinedQuery::default()
        .list(pool, data.timmy.id, data.instance.id)
        .await?,
    );

    // The read even shows timmy's sent messages
    assert_length!(3, &timmy_messages);
    assert_eq!(timmy_messages[0].creator.id, data.jessica.id);
    assert_eq!(timmy_messages[0].recipient.id, data.timmy.id);
    assert_eq!(timmy_messages[1].creator.id, data.timmy.id);
    assert_eq!(timmy_messages[1].recipient.id, data.sara.id);
    assert_eq!(timmy_messages[2].creator.id, data.sara.id);
    assert_eq!(timmy_messages[2].recipient.id, data.timmy.id);

    let timmy_unread =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, false)
        .await?;
    assert_eq!(2, timmy_unread);

    let timmy_unread_messages = map_to_pm(
      &InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.id, data.instance.id)
      .await?,
    );

    // The unread hides timmy's sent messages
    assert_length!(2, &timmy_unread_messages);
    assert_eq!(timmy_unread_messages[0].creator.id, data.jessica.id);
    assert_eq!(timmy_unread_messages[0].recipient.id, data.timmy.id);
    assert_eq!(timmy_unread_messages[1].creator.id, data.sara.id);
    assert_eq!(timmy_unread_messages[1].recipient.id, data.timmy.id);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn ensure_private_message_person_block() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    // Make sure blocks are working
    let timmy_blocks_sara_form = PersonBlockForm::new(data.timmy.id, data.sara.id);

    let inserted_block = PersonActions::block(pool, &timmy_blocks_sara_form).await?;

    assert_eq!(
      (data.timmy.id, data.sara.id, true),
      (
        inserted_block.person_id,
        inserted_block.target_id,
        inserted_block.blocked_at.is_some()
      )
    );

    let timmy_messages = map_to_pm(
      &InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.id, data.instance.id)
      .await?,
    );

    assert_length!(1, &timmy_messages);

    let timmy_unread =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, false)
        .await?;
    assert_eq!(1, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn ensure_private_message_instance_block() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    // Make sure instance_blocks are working
    let timmy_blocks_instance_form = InstanceBlockForm::new(data.timmy.id, data.sara.instance_id);

    let inserted_instance_block = InstanceActions::block(pool, &timmy_blocks_instance_form).await?;

    assert_eq!(
      (data.timmy.id, data.sara.instance_id, true),
      (
        inserted_instance_block.person_id,
        inserted_instance_block.instance_id,
        inserted_instance_block.blocked_at.is_some()
      )
    );

    let timmy_messages = map_to_pm(
      &InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.id, data.instance.id)
      .await?,
    );

    assert_length!(0, &timmy_messages);

    let timmy_unread =
      InboxCombinedViewInternal::get_unread_count(pool, data.timmy.id, data.instance.id, false)
        .await?;
    assert_eq!(0, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }
}
