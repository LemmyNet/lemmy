use crate::structs::PostView;
use diesel::{
  debug_query,
  dsl::{now, IntervalDsl},
  pg::Pg,
  result::Error,
  sql_function,
  sql_types,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PostAggregates,
  newtypes::{CommunityId, LocalUserId, PersonId, PostId},
  schema::{
    community,
    community_block,
    community_follower,
    community_person_ban,
    local_user_language,
    person,
    person_block,
    person_post_aggregates,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
  },
  source::{
    community::{Community, CommunityFollower, CommunityPersonBan},
    local_user::LocalUser,
    person::Person,
    person_block::PersonBlock,
    post::{Post, PostRead, PostSaved},
  },
  traits::JoinView,
  utils::{fuzzy_search, get_conn, limit_and_offset, DbPool},
  ListingType,
  SortType,
};
use tracing::debug;
use typed_builder::TypedBuilder;

type PostViewTuple = (
  Post,
  Person,
  Community,
  Option<CommunityPersonBan>,
  PostAggregates,
  Option<CommunityFollower>,
  Option<PostSaved>,
  Option<PostRead>,
  Option<PersonBlock>,
  Option<i16>,
  i64,
);

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

impl PostView {
  pub async fn read(
    pool: &DbPool,
    post_id: PostId,
    my_person_id: Option<PersonId>,
    is_mod_or_admin: Option<bool>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));
    let mut query = post::table
      .find(post_id)
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id)),
        ),
      )
      .inner_join(post_aggregates::table)
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post::id
            .eq(post_saved::post_id)
            .and(post_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post::id
            .eq(post_read::post_id)
            .and(post_read::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          post::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_post_aggregates::table.on(
          post::id
            .eq(person_post_aggregates::post_id)
            .and(person_post_aggregates::person_id.eq(person_id_join)),
        ),
      )
      .select((
        post::all_columns,
        person::all_columns,
        community::all_columns,
        community_person_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        person_block::all_columns.nullable(),
        post_like::score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - person_post_aggregates::read_comments.nullable(),
          post_aggregates::comments,
        ),
      ))
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin.unwrap_or(false) {
      query = query
        .filter(community::removed.eq(false))
        .filter(community::deleted.eq(false))
        .filter(post::removed.eq(false))
        .filter(post::deleted.eq(false));
    }

    let (
      post,
      creator,
      community,
      creator_banned_from_community,
      counts,
      follower,
      saved,
      read,
      creator_blocked,
      post_like,
      unread_comments,
    ) = query.first::<PostViewTuple>(conn).await?;

    // If a person is given, then my_vote, if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    let my_vote = if my_person_id.is_some() && post_like.is_none() {
      Some(0)
    } else {
      post_like
    };

    Ok(PostView {
      post,
      creator,
      community,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      counts,
      subscribed: CommunityFollower::to_subscribed_type(&follower),
      saved: saved.is_some(),
      read: read.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
      unread_comments,
    })
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  listing_type: Option<ListingType>,
  sort: Option<SortType>,
  creator_id: Option<PersonId>,
  community_id: Option<CommunityId>,
  local_user: Option<&'a LocalUser>,
  search_term: Option<String>,
  url_search: Option<String>,
  saved_only: Option<bool>,
  /// Used to show deleted or removed posts for admins
  is_mod_or_admin: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PostQuery<'a> {
  pub async fn list(self) -> Result<Vec<PostView>, Error> {
    let conn = &mut get_conn(self.pool).await?;

    // The left join below will return None in this case
    let person_id_join = self.local_user.map(|l| l.person_id).unwrap_or(PersonId(-1));
    let local_user_id_join = self.local_user.map(|l| l.id).unwrap_or(LocalUserId(-1));

    let mut query = post::table
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id)),
        ),
      )
      .inner_join(post_aggregates::table)
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post::id
            .eq(post_saved::post_id)
            .and(post_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post::id
            .eq(post_read::post_id)
            .and(post_read::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          post::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_block::table.on(
          post::community_id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_post_aggregates::table.on(
          post::id
            .eq(person_post_aggregates::post_id)
            .and(person_post_aggregates::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        local_user_language::table.on(
          post::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select((
        post::all_columns,
        person::all_columns,
        community::all_columns,
        community_person_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        person_block::all_columns.nullable(),
        post_like::score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - person_post_aggregates::read_comments.nullable(),
          post_aggregates::comments,
        ),
      ))
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    // TODO This eventually needs to show posts where you are the creator
    if !self.is_mod_or_admin.unwrap_or(false) {
      query = query
        .filter(community::removed.eq(false))
        .filter(community::deleted.eq(false))
        .filter(post::removed.eq(false))
        .filter(post::deleted.eq(false));
    }

    if self.community_id.is_none() {
      query = query.then_order_by(post_aggregates::featured_local.desc());
    } else if let Some(community_id) = self.community_id {
      query = query
        .filter(post::community_id.eq(community_id))
        .then_order_by(post_aggregates::featured_community.desc());
    }

    if let Some(creator_id) = self.creator_id {
      query = query.filter(post::creator_id.eq(creator_id));
    }

    if let Some(listing_type) = self.listing_type {
      match listing_type {
        ListingType::Subscribed => {
          query = query.filter(community_follower::person_id.is_not_null())
        }
        ListingType::Local => {
          query = query.filter(community::local.eq(true)).filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          );
        }
        ListingType::All => {
          query = query.filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          )
        }
      }
    }

    if let Some(url_search) = self.url_search {
      query = query.filter(post::url.eq(url_search));
    }

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query.filter(
        post::name
          .ilike(searcher.clone())
          .or(post::body.ilike(searcher)),
      );
    }

    if !self.local_user.map(|l| l.show_nsfw).unwrap_or(false) {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    if !self.local_user.map(|l| l.show_bot_accounts).unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    if self.saved_only.unwrap_or(false) {
      query = query.filter(post_saved::post_id.is_not_null());
    }
    // Only hide the read posts, if the saved_only is false. Otherwise ppl with the hide_read
    // setting wont be able to see saved posts.
    else if !self.local_user.map(|l| l.show_read_posts).unwrap_or(true) {
      query = query.filter(post_read::post_id.is_null());
    }

    if self.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::language_id.is_not_null());

      // Don't show blocked communities or persons
      query = query.filter(community_block::person_id.is_null());
      query = query.filter(person_block::person_id.is_null());
    }

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::Active => query.then_order_by(post_aggregates::hot_rank_active.desc()),
      SortType::Hot => query.then_order_by(post_aggregates::hot_rank.desc()),
      SortType::New => query.then_order_by(post_aggregates::published.desc()),
      SortType::Old => query.then_order_by(post_aggregates::published.asc()),
      SortType::NewComments => query.then_order_by(post_aggregates::newest_comment_time.desc()),
      SortType::MostComments => query
        .then_order_by(post_aggregates::comments.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopAll => query
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopYear => query
        .filter(post_aggregates::published.gt(now - 1.years()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopMonth => query
        .filter(post_aggregates::published.gt(now - 1.months()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopWeek => query
        .filter(post_aggregates::published.gt(now - 1.weeks()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopDay => query
        .filter(post_aggregates::published.gt(now - 1.days()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopHour => query
        .filter(post_aggregates::published.gt(now - 1.hours()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopSixHour => query
        .filter(post_aggregates::published.gt(now - 6.hours()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::TopTwelveHour => query
        .filter(post_aggregates::published.gt(now - 12.hours()))
        .then_order_by(post_aggregates::score.desc())
        .then_order_by(post_aggregates::published.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    query = query.limit(limit).offset(offset);

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));

    let res = query.load::<PostViewTuple>(conn).await?;

    Ok(res.into_iter().map(PostView::from_tuple).collect())
  }
}

impl JoinView for PostView {
  type JoinTuple = PostViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      post: a.0,
      creator: a.1,
      community: a.2,
      creator_banned_from_community: a.3.is_some(),
      counts: a.4,
      subscribed: CommunityFollower::to_subscribed_type(&a.5),
      saved: a.6.is_some(),
      read: a.7.is_some(),
      creator_blocked: a.8.is_some(),
      my_vote: a.9,
      unread_comments: a.10,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::post_view::{PostQuery, PostView};
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      community::{Community, CommunityInsertForm},
      community_block::{CommunityBlock, CommunityBlockForm},
      instance::Instance,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm, PostUpdateForm},
    },
    traits::{Blockable, Crud, Likeable},
    utils::{build_db_pool_for_tests, DbPool},
    SortType,
    SubscribedType,
  };
  use serial_test::serial;

  struct Data {
    inserted_instance: Instance,
    inserted_person: Person,
    inserted_local_user: LocalUser,
    inserted_blocked_person: Person,
    inserted_bot: Person,
    inserted_community: Community,
    inserted_post: Post,
  }

  async fn init_data(pool: &DbPool) -> Data {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let person_name = "tegan".to_string();

    let new_person = PersonInsertForm::builder()
      .name(person_name.clone())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted(String::new())
      .build();
    let inserted_local_user = LocalUser::create(pool, &local_user_form).await.unwrap();

    let new_bot = PersonInsertForm::builder()
      .name("mybot".to_string())
      .bot_account(Some(true))
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_bot = Person::create(pool, &new_bot).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test_community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    // Test a person block, make sure the post query doesn't include their post
    let blocked_person = PersonInsertForm::builder()
      .name(person_name)
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_blocked_person = Person::create(pool, &blocked_person).await.unwrap();

    let post_from_blocked_person = PostInsertForm::builder()
      .name("blocked_person_post".to_string())
      .creator_id(inserted_blocked_person.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(1)))
      .build();

    Post::create(pool, &post_from_blocked_person).await.unwrap();

    // block that person
    let person_block = PersonBlockForm {
      person_id: inserted_person.id,
      target_id: inserted_blocked_person.id,
    };

    PersonBlock::block(pool, &person_block).await.unwrap();

    // A sample post
    let new_post = PostInsertForm::builder()
      .name("test post 3".to_string())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(47)))
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let new_bot_post = PostInsertForm::builder()
      .name("test bot post".to_string())
      .creator_id(inserted_bot.id)
      .community_id(inserted_community.id)
      .build();

    let _inserted_bot_post = Post::create(pool, &new_bot_post).await.unwrap();

    Data {
      inserted_instance,
      inserted_person,
      inserted_local_user,
      inserted_blocked_person,
      inserted_bot,
      inserted_community,
      inserted_post,
    }
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_with_person() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let local_user_form = LocalUserUpdateForm::builder()
      .show_bot_accounts(Some(false))
      .build();
    let inserted_local_user =
      LocalUser::update(pool, data.inserted_local_user.id, &local_user_form)
        .await
        .unwrap();

    let read_post_listing = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(data.inserted_person.id),
      None,
    )
    .await
    .unwrap();

    let mut expected_post_listing_with_user = expected_post_view(&data, pool).await;

    // Should be only one person, IE the bot post, and blocked should be missing
    assert_eq!(1, read_post_listing.len());

    assert_eq!(expected_post_listing_with_user, read_post_listing[0]);
    expected_post_listing_with_user.my_vote = Some(0);
    assert_eq!(
      expected_post_listing_with_user,
      post_listing_single_with_person
    );

    let local_user_form = LocalUserUpdateForm::builder()
      .show_bot_accounts(Some(true))
      .build();
    let inserted_local_user =
      LocalUser::update(pool, data.inserted_local_user.id, &local_user_form)
        .await
        .unwrap();

    let post_listings_with_bots = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();
    // should include bot post which has "undetermined" language
    assert_eq!(2, post_listings_with_bots.len());

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_no_person() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let read_post_listing_multiple_no_person = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .build()
      .list()
      .await
      .unwrap();

    let read_post_listing_single_no_person =
      PostView::read(pool, data.inserted_post.id, None, None)
        .await
        .unwrap();

    let expected_post_listing_no_person = expected_post_view(&data, pool).await;

    // Should be 2 posts, with the bot post, and the blocked
    assert_eq!(3, read_post_listing_multiple_no_person.len());

    assert_eq!(
      expected_post_listing_no_person,
      read_post_listing_multiple_no_person[1]
    );
    assert_eq!(
      expected_post_listing_no_person,
      read_post_listing_single_no_person
    );

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_block_community() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let community_block = CommunityBlockForm {
      person_id: data.inserted_person.id,
      community_id: data.inserted_community.id,
    };
    CommunityBlock::block(pool, &community_block).await.unwrap();

    let read_post_listings_with_person_after_block = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();
    // Should be 0 posts after the community block
    assert_eq!(0, read_post_listings_with_person_after_block.len());

    CommunityBlock::unblock(pool, &community_block)
      .await
      .unwrap();
    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_like() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let post_like_form = PostLikeForm {
      post_id: data.inserted_post.id,
      person_id: data.inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(pool, &post_like_form).await.unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: data.inserted_post.id,
      person_id: data.inserted_person.id,
      published: inserted_post_like.published,
      score: 1,
    };
    assert_eq!(expected_post_like, inserted_post_like);

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(data.inserted_person.id),
      None,
    )
    .await
    .unwrap();

    let mut expected_post_with_upvote = expected_post_view(&data, pool).await;
    expected_post_with_upvote.my_vote = Some(1);
    expected_post_with_upvote.counts.score = 1;
    expected_post_with_upvote.counts.upvotes = 1;
    assert_eq!(expected_post_with_upvote, post_listing_single_with_person);

    let local_user_form = LocalUserUpdateForm::builder()
      .show_bot_accounts(Some(false))
      .build();
    let inserted_local_user =
      LocalUser::update(pool, data.inserted_local_user.id, &local_user_form)
        .await
        .unwrap();

    let read_post_listing = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();
    assert_eq!(1, read_post_listing.len());

    assert_eq!(expected_post_with_upvote, read_post_listing[0]);

    let like_removed = PostLike::remove(pool, data.inserted_person.id, data.inserted_post.id)
      .await
      .unwrap();
    assert_eq!(1, like_removed);
    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_person_language() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let spanish_id = Language::read_id_from_code(pool, Some("es"))
      .await
      .unwrap()
      .unwrap();
    let post_spanish = PostInsertForm::builder()
      .name("asffgdsc".to_string())
      .creator_id(data.inserted_person.id)
      .community_id(data.inserted_community.id)
      .language_id(Some(spanish_id))
      .build();

    Post::create(pool, &post_spanish).await.unwrap();

    let post_listings_all = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();

    // no language filters specified, all posts should be returned
    assert_eq!(3, post_listings_all.len());

    let french_id = Language::read_id_from_code(pool, Some("fr"))
      .await
      .unwrap()
      .unwrap();
    LocalUserLanguage::update(pool, vec![french_id], data.inserted_local_user.id)
      .await
      .unwrap();

    let post_listing_french = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();

    // only one post in french and one undetermined should be returned
    assert_eq!(2, post_listing_french.len());
    assert!(post_listing_french
      .iter()
      .any(|p| p.post.language_id == french_id));

    LocalUserLanguage::update(
      pool,
      vec![french_id, UNDETERMINED_ID],
      data.inserted_local_user.id,
    )
    .await
    .unwrap();
    let post_listings_french_und = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();

    // french post and undetermined language post should be returned
    assert_eq!(2, post_listings_french_und.len());
    assert_eq!(
      UNDETERMINED_ID,
      post_listings_french_und[0].post.language_id
    );
    assert_eq!(french_id, post_listings_french_und[1].post.language_id);

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listings_deleted() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    // Delete the post
    Post::update(
      pool,
      data.inserted_post.id,
      &PostUpdateForm::builder().deleted(Some(true)).build(),
    )
    .await
    .unwrap();

    // Make sure you don't see the deleted post in the results
    let post_listings_no_admin = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .is_mod_or_admin(Some(false))
      .build()
      .list()
      .await
      .unwrap();

    assert_eq!(1, post_listings_no_admin.len());

    // Make sure they see both
    let post_listings_is_admin = PostQuery::builder()
      .pool(pool)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .is_mod_or_admin(Some(true))
      .build()
      .list()
      .await
      .unwrap();

    assert_eq!(2, post_listings_is_admin.len());

    cleanup(data, pool).await;
  }

  async fn cleanup(data: Data, pool: &DbPool) {
    let num_deleted = Post::delete(pool, data.inserted_post.id).await.unwrap();
    Community::delete(pool, data.inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, data.inserted_person.id).await.unwrap();
    Person::delete(pool, data.inserted_bot.id).await.unwrap();
    Person::delete(pool, data.inserted_blocked_person.id)
      .await
      .unwrap();
    Instance::delete(pool, data.inserted_instance.id)
      .await
      .unwrap();
    assert_eq!(1, num_deleted);
  }

  async fn expected_post_view(data: &Data, pool: &DbPool) -> PostView {
    let (inserted_person, inserted_community, inserted_post) = (
      &data.inserted_person,
      &data.inserted_community,
      &data.inserted_post,
    );
    let agg = PostAggregates::read(pool, inserted_post.id).await.unwrap();

    PostView {
      post: Post {
        id: inserted_post.id,
        name: inserted_post.name.clone(),
        creator_id: inserted_person.id,
        url: None,
        body: None,
        published: inserted_post.published,
        updated: None,
        community_id: inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_video_url: None,
        thumbnail_url: None,
        ap_id: inserted_post.ap_id.clone(),
        local: true,
        language_id: LanguageId(47),
        featured_community: false,
        featured_local: false,
      },
      my_vote: None,
      unread_comments: 0,
      creator: Person {
        id: inserted_person.id,
        name: inserted_person.name.clone(),
        display_name: None,
        published: inserted_person.published,
        avatar: None,
        actor_id: inserted_person.actor_id.clone(),
        local: true,
        admin: false,
        bot_account: false,
        banned: false,
        deleted: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_person.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
        instance_id: data.inserted_instance.id,
        private_key: inserted_person.private_key.clone(),
        public_key: inserted_person.public_key.clone(),
        last_refreshed_at: inserted_person.last_refreshed_at,
      },
      creator_banned_from_community: false,
      community: Community {
        id: inserted_community.id,
        name: inserted_community.name.clone(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.clone(),
        local: true,
        title: "nada".to_owned(),
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: inserted_community.published,
        instance_id: data.inserted_instance.id,
        private_key: inserted_community.private_key.clone(),
        public_key: inserted_community.public_key.clone(),
        last_refreshed_at: inserted_community.last_refreshed_at,
        followers_url: inserted_community.followers_url.clone(),
        inbox_url: inserted_community.inbox_url.clone(),
        shared_inbox_url: inserted_community.shared_inbox_url.clone(),
        moderators_url: inserted_community.moderators_url.clone(),
        featured_url: inserted_community.featured_url.clone(),
      },
      counts: PostAggregates {
        id: agg.id,
        post_id: inserted_post.id,
        comments: 0,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        published: agg.published,
        newest_comment_time_necro: inserted_post.published,
        newest_comment_time: inserted_post.published,
        featured_community: false,
        featured_local: false,
        hot_rank: 1728,
        hot_rank_active: 1728,
      },
      subscribed: SubscribedType::NotSubscribed,
      read: false,
      saved: false,
      creator_blocked: false,
    }
  }
}
