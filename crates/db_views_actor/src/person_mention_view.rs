use crate::structs::PersonMentionView;
use diesel::{
  dsl::exists,
  pg::Pg,
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::{self, creator_community_actions},
  newtypes::{PersonId, PersonMentionId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    person_mention,
    post,
  },
  source::{community::CommunityFollower, local_user::LocalUser},
  utils::{
    actions,
    actions_alias,
    functions::coalesce,
    get_conn,
    limit_and_offset,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
  CommentSortType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PersonMentionView, (PersonMentionId, Option<PersonId>)>,
  impl ListFn<'a, PersonMentionView, PersonMentionQuery>,
> {
  let creator_is_admin = exists(
    local_user::table.filter(
      comment::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    ),
  );

  let all_joins = move |query: person_mention::BoxedQuery<'a, Pg>,
                        my_person_id: Option<PersonId>| {
    query
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(aliases::person1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(actions(
        community_actions::table,
        my_person_id,
        post::community_id,
      ))
      .left_join(actions(comment_actions::table, my_person_id, comment::id))
      .left_join(actions(
        person_actions::table,
        my_person_id,
        comment::creator_id,
      ))
      .left_join(actions_alias(
        creator_community_actions,
        comment::creator_id,
        post::community_id,
      ))
      .select((
        person_mention::all_columns,
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        aliases::person1.fields(person::all_columns),
        comment_aggregates::all_columns,
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        community_actions::received_ban.nullable().is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        creator_is_admin,
        CommunityFollower::select_subscribed_type(),
        comment_actions::saved.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
      ))
  };

  let read =
    move |mut conn: DbConn<'a>,
          (person_mention_id, my_person_id): (PersonMentionId, Option<PersonId>)| async move {
      all_joins(
        person_mention::table.find(person_mention_id).into_boxed(),
        my_person_id,
      )
      .first(&mut conn)
      .await
    };

  let list = move |mut conn: DbConn<'a>, options: PersonMentionQuery| async move {
    // These filters need to be kept in sync with the filters in
    // PersonMentionView::get_unread_mentions()
    let mut query = all_joins(person_mention::table.into_boxed(), options.my_person_id);

    if let Some(recipient_id) = options.recipient_id {
      query = query.filter(person_mention::recipient_id.eq(recipient_id));
    }

    if options.unread_only {
      query = query.filter(person_mention::read.eq(false));
    }

    if !options.show_bot_accounts {
      query = query.filter(not(person::bot_account));
    };

    query = match options.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query.then_order_by(comment_aggregates::hot_rank.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    // Don't show mentions from blocked persons
    query = query.filter(person_actions::blocked.is_null());

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .load::<PersonMentionView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl PersonMentionView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_mention_id: PersonMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Option<Self>, Error> {
    queries()
      .read(pool, (person_mention_id, my_person_id))
      .await
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_mentions(
    pool: &mut DbPool<'_>,
    local_user: &LocalUser,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let mut query = person_mention::table
      .inner_join(comment::table)
      .left_join(actions(
        person_actions::table,
        Some(local_user.person_id),
        comment::creator_id,
      ))
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !local_user.show_bot_accounts {
      query = query.filter(not(person::bot_account));
    }

    query
      // Dont count replies from blocked users
      .filter(person_actions::blocked.is_null())
      .filter(person_mention::recipient_id.eq(local_user.person_id))
      .filter(person_mention::read.eq(false))
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .select(count(person_mention::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default, Clone)]
pub struct PersonMentionQuery {
  pub my_person_id: Option<PersonId>,
  pub recipient_id: Option<PersonId>,
  pub sort: Option<CommentSortType>,
  pub unread_only: bool,
  pub show_bot_accounts: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PersonMentionQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PersonMentionView>, Error> {
    queries().list(pool, self).await
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{person_mention_view::PersonMentionQuery, structs::PersonMentionView};
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm, PersonUpdateForm},
      person_block::{PersonBlock, PersonBlockForm},
      person_mention::{PersonMention, PersonMentionInsertForm, PersonMentionUpdateForm},
      post::{Post, PostInsertForm},
    },
    traits::{Blockable, Crud},
    utils::build_db_pool_for_tests,
  };
  use lemmy_db_views::structs::LocalUserView;
  use lemmy_utils::{error::LemmyResult, LemmyErrorType};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::builder()
      .name("terrylake".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await?;

    let recipient_form = PersonInsertForm::builder()
      .name("terrylakes recipient".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(pool, &recipient_form).await?;
    let recipient_id = inserted_recipient.id;

    let recipient_local_user =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(recipient_id), vec![]).await?;

    let new_community = CommunityInsertForm::builder()
      .name("test community lake".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let person_mention_form = PersonMentionInsertForm {
      recipient_id: inserted_recipient.id,
      comment_id: inserted_comment.id,
      read: None,
    };

    let inserted_mention = PersonMention::create(pool, &person_mention_form).await?;

    let expected_mention = PersonMention {
      id: inserted_mention.id,
      recipient_id: inserted_mention.recipient_id,
      comment_id: inserted_mention.comment_id,
      read: false,
      published: inserted_mention.published,
    };

    let read_mention = PersonMention::read(pool, inserted_mention.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindComment)?;

    let person_mention_update_form = PersonMentionUpdateForm { read: Some(false) };
    let updated_mention =
      PersonMention::update(pool, inserted_mention.id, &person_mention_update_form).await?;

    // Test to make sure counts and blocks work correctly
    let unread_mentions =
      PersonMentionView::get_unread_mentions(pool, &recipient_local_user).await?;

    let query = PersonMentionQuery {
      recipient_id: Some(recipient_id),
      my_person_id: Some(recipient_id),
      sort: None,
      unread_only: false,
      show_bot_accounts: true,
      page: None,
      limit: None,
    };
    let mentions = query.clone().list(pool).await?;
    assert_eq!(1, unread_mentions);
    assert_eq!(1, mentions.len());

    // Block the person, and make sure these counts are now empty
    let block_form = PersonBlockForm {
      person_id: recipient_id,
      target_id: inserted_person.id,
    };
    PersonBlock::block(pool, &block_form).await?;

    let unread_mentions_after_block =
      PersonMentionView::get_unread_mentions(pool, &recipient_local_user).await?;
    let mentions_after_block = query.clone().list(pool).await?;
    assert_eq!(0, unread_mentions_after_block);
    assert_eq!(0, mentions_after_block.len());

    // Unblock user so we can reuse the same person
    PersonBlock::unblock(pool, &block_form).await?;

    // Turn Terry into a bot account
    let person_update_form = PersonUpdateForm {
      bot_account: Some(true),
      ..Default::default()
    };
    Person::update(pool, inserted_person.id, &person_update_form).await?;

    let recipient_local_user_update_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      recipient_local_user.id,
      &recipient_local_user_update_form,
    )
    .await?;
    let recipient_local_user_view = LocalUserView::read(pool, recipient_local_user.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindLocalUser)?;

    let unread_mentions_after_hide_bots =
      PersonMentionView::get_unread_mentions(pool, &recipient_local_user_view.local_user).await?;

    let mut query_without_bots = query.clone();
    query_without_bots.show_bot_accounts = false;
    let replies_after_hide_bots = query_without_bots.list(pool).await?;
    assert_eq!(0, unread_mentions_after_hide_bots);
    assert_eq!(0, replies_after_hide_bots.len());

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Person::delete(pool, inserted_recipient.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_mention, read_mention);
    assert_eq!(expected_mention, inserted_mention);
    assert_eq!(expected_mention, updated_mention);

    Ok(())
  }
}
