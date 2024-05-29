use crate::structs::CommentReplyView;
use diesel::{
  dsl::{exists, not},
  pg::Pg,
  result::Error,
  sql_types,
  BoolExpressionMethods,
  BoxableExpression,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::{CommentReplyId, PersonId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_reply,
    comment_saved,
    community,
    community_follower,
    community_moderator,
    community_person_ban,
    local_user,
    person,
    person_block,
    post,
  },
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  CommentSortType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommentReplyView, (CommentReplyId, Option<PersonId>)>,
  impl ListFn<'a, CommentReplyView, CommentReplyQuery>,
> {
  let is_creator_banned_from_community = exists(
    community_person_ban::table.filter(
      community::id
        .eq(community_person_ban::community_id)
        .and(community_person_ban::person_id.eq(comment::creator_id)),
    ),
  );

  let is_local_user_banned_from_community = |person_id| {
    exists(
      community_person_ban::table.filter(
        community::id
          .eq(community_person_ban::community_id)
          .and(community_person_ban::person_id.eq(person_id)),
      ),
    )
  };

  let is_saved = |person_id| {
    exists(
      comment_saved::table.filter(
        comment::id
          .eq(comment_saved::comment_id)
          .and(comment_saved::person_id.eq(person_id)),
      ),
    )
  };

  let is_community_followed = |person_id| {
    community_follower::table
      .filter(
        post::community_id
          .eq(community_follower::community_id)
          .and(community_follower::person_id.eq(person_id)),
      )
      .select(community_follower::pending.nullable())
      .single_value()
  };

  let is_creator_blocked = |person_id| {
    exists(
      person_block::table.filter(
        comment::creator_id
          .eq(person_block::target_id)
          .and(person_block::person_id.eq(person_id)),
      ),
    )
  };

  let score = |person_id| {
    comment_like::table
      .filter(
        comment::id
          .eq(comment_like::comment_id)
          .and(comment_like::person_id.eq(person_id)),
      )
      .select(comment_like::score.nullable())
      .single_value()
  };

  let creator_is_moderator = exists(
    community_moderator::table.filter(
      community::id
        .eq(community_moderator::community_id)
        .and(community_moderator::person_id.eq(comment::creator_id)),
    ),
  );

  let creator_is_admin = exists(
    local_user::table.filter(
      comment::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    ),
  );

  let all_joins = move |query: comment_reply::BoxedQuery<'a, Pg>,
                        my_person_id: Option<PersonId>| {
    let is_local_user_banned_from_community_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_local_user_banned_from_community(person_id))
    } else {
      Box::new(false.into_sql::<sql_types::Bool>())
    };

    let score_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(score(person_id))
    } else {
      Box::new(None::<i16>.into_sql::<sql_types::Nullable<sql_types::SmallInt>>())
    };

    let subscribed_type_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::Bool>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_community_followed(person_id))
    } else {
      Box::new(None::<bool>.into_sql::<sql_types::Nullable<sql_types::Bool>>())
    };

    let is_saved_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_saved(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    let is_creator_blocked_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_creator_blocked(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    query
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .select((
        comment_reply::all_columns,
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        aliases::person1.fields(person::all_columns),
        comment_aggregates::all_columns,
        is_creator_banned_from_community,
        is_local_user_banned_from_community_selection,
        creator_is_moderator,
        creator_is_admin,
        subscribed_type_selection,
        is_saved_selection,
        is_creator_blocked_selection,
        score_selection,
      ))
  };

  let read =
    move |mut conn: DbConn<'a>,
          (comment_reply_id, my_person_id): (CommentReplyId, Option<PersonId>)| async move {
      all_joins(
        comment_reply::table.find(comment_reply_id).into_boxed(),
        my_person_id,
      )
      .first(&mut conn)
      .await
    };

  let list = move |mut conn: DbConn<'a>, options: CommentReplyQuery| async move {
    let mut query = all_joins(comment_reply::table.into_boxed(), options.my_person_id);

    if let Some(recipient_id) = options.recipient_id {
      query = query.filter(comment_reply::recipient_id.eq(recipient_id));
    }

    if options.unread_only {
      query = query.filter(comment_reply::read.eq(false));
    }

    if !options.show_bot_accounts {
      query = query.filter(person::bot_account.eq(false));
    };

    query = match options.sort.unwrap_or(CommentSortType::New) {
      CommentSortType::Hot => query.then_order_by(comment_aggregates::hot_rank.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment_reply::published.desc()),
      CommentSortType::Old => query.then_order_by(comment_reply::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    // Don't show replies from blocked persons
    if let Some(my_person_id) = options.my_person_id {
      query = query.filter(not(is_creator_blocked(my_person_id)));
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .load::<CommentReplyView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl CommentReplyView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_reply_id: CommentReplyId,
    my_person_id: Option<PersonId>,
  ) -> Result<Option<Self>, Error> {
    queries().read(pool, (comment_reply_id, my_person_id)).await
  }

  /// Gets the number of unread replies
  pub async fn get_unread_replies(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

    comment_reply::table
      .inner_join(comment::table)
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(my_person_id)),
        ),
      )
      // Dont count replies from blocked users
      .filter(person_block::person_id.is_null())
      .filter(comment_reply::recipient_id.eq(my_person_id))
      .filter(comment_reply::read.eq(false))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .select(count(comment_reply::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default, Clone)]
pub struct CommentReplyQuery {
  pub my_person_id: Option<PersonId>,
  pub recipient_id: Option<PersonId>,
  pub sort: Option<CommentSortType>,
  pub unread_only: bool,
  pub show_bot_accounts: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl CommentReplyQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<CommentReplyView>, Error> {
    queries().list(pool, self).await
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{comment_reply_view::CommentReplyQuery, structs::CommentReplyView};
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm},
      comment_reply::{CommentReply, CommentReplyInsertForm, CommentReplyUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm},
    },
    traits::{Blockable, Crud},
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::{error::LemmyResult, LemmyErrorType};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let terry_form = PersonInsertForm::builder()
      .name("terrylake".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_terry = Person::create(pool, &terry_form).await?;

    let recipient_form = PersonInsertForm::builder()
      .name("terrylakes recipient".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(pool, &recipient_form).await?;
    let recipient_id = inserted_recipient.id;

    let new_community = CommunityInsertForm::builder()
      .name("test community lake".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_terry.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_terry.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let comment_reply_form = CommentReplyInsertForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_reply = CommentReply::create(pool, &comment_reply_form).await?;

    let expected_reply = CommentReply {
      id: inserted_reply.id,
      recipient_id: inserted_reply.recipient_id,
      comment_id: inserted_reply.comment_id,
      read: false,
      published: inserted_reply.published,
    };

    let read_reply = CommentReply::read(pool, inserted_reply.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindComment)?;

    let comment_reply_update_form = CommentReplyUpdateForm { read: Some(false) };
    let updated_reply =
      CommentReply::update(pool, inserted_reply.id, &comment_reply_update_form).await?;

    // Test to make sure counts and blocks work correctly
    let unread_replies = CommentReplyView::get_unread_replies(pool, recipient_id).await?;

    let query = CommentReplyQuery {
      recipient_id: Some(recipient_id),
      my_person_id: Some(recipient_id),
      sort: None,
      unread_only: false,
      show_bot_accounts: true,
      page: None,
      limit: None,
    };
    let replies = query.clone().list(pool).await?;
    assert_eq!(1, unread_replies);
    assert_eq!(1, replies.len());

    // Block the person, and make sure these counts are now empty
    let block_form = PersonBlockForm {
      person_id: recipient_id,
      target_id: inserted_terry.id,
    };
    PersonBlock::block(pool, &block_form).await?;

    let unread_replies_after_block =
      CommentReplyView::get_unread_replies(pool, recipient_id).await?;
    let replies_after_block = query.list(pool).await?;
    assert_eq!(0, unread_replies_after_block);
    assert_eq!(0, replies_after_block.len());

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_terry.id).await?;
    Person::delete(pool, inserted_recipient.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_reply, read_reply);
    assert_eq!(expected_reply, inserted_reply);
    assert_eq!(expected_reply, updated_reply);
    Ok(())
  }
}
