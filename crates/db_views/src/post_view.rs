use crate::structs::{LocalUserView, PostView};
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
  OptionalExtension,
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
    community_moderator,
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
    community::{Community, CommunityFollower},
    person::Person,
    post::Post,
  },
  traits::JoinView,
  utils::{fuzzy_search, get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  ListingType,
  SortType,
  SubscribedType,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

type PostViewTuple = (
  Post,
  Person,
  Community,
  bool,
  PostAggregates,
  SubscribedType,
  bool,
  bool,
  bool,
  Option<i16>,
  i64,
);

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PostView, (PostId, Option<PersonId>, bool)>,
  impl ListFn<'a, PostView, PostQuery<'a>>,
> {
  let all_joins = |query: post_aggregates::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    query
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(
        community_person_ban::table.on(
          post_aggregates::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post_aggregates::creator_id)),
        ),
      )
      .inner_join(post::table)
      .left_join(
        community_follower::table.on(
          post_aggregates::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_moderator::table.on(
          post::community_id
            .eq(community_moderator::community_id)
            .and(community_moderator::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post_aggregates::post_id
            .eq(post_saved::post_id)
            .and(post_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post_aggregates::post_id
            .eq(post_read::post_id)
            .and(post_read::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          post_aggregates::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post_aggregates::post_id
            .eq(post_like::post_id)
            .and(post_like::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_post_aggregates::table.on(
          post_aggregates::post_id
            .eq(person_post_aggregates::post_id)
            .and(person_post_aggregates::person_id.eq(person_id_join)),
        ),
      )
  };

  let selection = (
    post::all_columns,
    person::all_columns,
    community::all_columns,
    community_person_ban::id.nullable().is_not_null(),
    post_aggregates::all_columns,
    CommunityFollower::select_subscribed_type(),
    post_saved::id.nullable().is_not_null(),
    post_read::id.nullable().is_not_null(),
    person_block::id.nullable().is_not_null(),
    post_like::score.nullable(),
    coalesce(
      post_aggregates::comments.nullable() - person_post_aggregates::read_comments.nullable(),
      post_aggregates::comments,
    ),
  );

  let read =
    move |mut conn: DbConn<'a>,
          (post_id, my_person_id, is_mod_or_admin): (PostId, Option<PersonId>, bool)| async move {
      // The left join below will return None in this case
      let person_id_join = my_person_id.unwrap_or(PersonId(-1));

      let mut query = all_joins(
        post_aggregates::table
          .filter(post_aggregates::post_id.eq(post_id))
          .into_boxed(),
        my_person_id,
      )
      .select(selection);

      // Hide deleted and removed for non-admins or mods
      if !is_mod_or_admin {
        query = query
          .filter(community::removed.eq(false))
          .filter(post::removed.eq(false))
          // users can see their own deleted posts
          .filter(
            community::deleted
              .eq(false)
              .or(post::creator_id.eq(person_id_join)),
          )
          .filter(
            post::deleted
              .eq(false)
              .or(post::creator_id.eq(person_id_join)),
          );
      }

      query.first::<PostViewTuple>(&mut conn).await
    };

  macro_rules! order_and_page_filter_desc {
    ($query:ident, $options:ident, $column_name:ident) => {{
      let mut query = $query.then_order_by(post_aggregates::$column_name.desc());
      if let Some(before) = &$options.page_before_or_equal {
        query = query.filter(post_aggregates::$column_name.ge(before.0.$column_name));
      }
      if let Some(after) = &$options.page_after {
        query = query.filter(post_aggregates::$column_name.le(after.0.$column_name));
      }
      query
    }};
  }
  macro_rules! order_and_page_filter_asc {
    ($query:ident, $options:ident, $column_name:ident) => {{
      let mut query = $query.then_order_by(post_aggregates::$column_name.asc());
      if let Some(before) = &$options.page_before_or_equal {
        query = query.filter(post_aggregates::$column_name.le(before.0.$column_name));
      }
      if let Some(after) = &$options.page_after {
        query = query.filter(post_aggregates::$column_name.ge(after.0.$column_name));
      }
      query
    }};
  }

  let list = move |mut conn: DbConn<'a>, options: PostQuery<'a>| async move {
    let person_id = options.local_user.map(|l| l.person.id);
    let local_user_id = options.local_user.map(|l| l.local_user.id);

    // The left join below will return None in this case
    let person_id_join = person_id.unwrap_or(PersonId(-1));
    let local_user_id_join = local_user_id.unwrap_or(LocalUserId(-1));

    let mut query = all_joins(post_aggregates::table.into_boxed(), person_id)
      .left_join(
        community_block::table.on(
          post_aggregates::community_id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        local_user_language::table.on(
          post::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select(selection);

    let is_creator = options.creator_id == options.local_user.map(|l| l.person.id);
    // only show deleted posts to creator
    if is_creator {
      query = query
        .filter(community::deleted.eq(false))
        .filter(post::deleted.eq(false));
    }

    let is_admin = options.local_user.map(|l| l.person.admin).unwrap_or(false);
    // only show removed posts to admin when viewing user profile
    if !(options.is_profile_view && is_admin) {
      query = query
        .filter(community::removed.eq(false))
        .filter(post::removed.eq(false));
    }
    if options.community_id.is_none() || options.community_id_just_for_prefetch {
      query = order_and_page_filter_desc!(query, options, featured_local);
    } else {
      query = order_and_page_filter_desc!(query, options, featured_community);
    }
    if let Some(community_id) = options.community_id {
      query = query.filter(post_aggregates::community_id.eq(community_id));
    }

    if let Some(creator_id) = options.creator_id {
      query = query.filter(post_aggregates::creator_id.eq(creator_id));
    }

    if let Some(listing_type) = options.listing_type {
      match listing_type {
        ListingType::Subscribed => query = query.filter(community_follower::pending.is_not_null()),
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

    if let Some(url_search) = options.url_search {
      query = query.filter(post::url.eq(url_search));
    }

    if let Some(search_term) = options.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query.filter(
        post::name
          .ilike(searcher.clone())
          .or(post::body.ilike(searcher)),
      );
    }

    if !options
      .local_user
      .map(|l| l.local_user.show_nsfw)
      .unwrap_or(false)
    {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    if !options
      .local_user
      .map(|l| l.local_user.show_bot_accounts)
      .unwrap_or(true)
    {
      query = query.filter(person::bot_account.eq(false));
    };

    if options.saved_only {
      query = query.filter(post_saved::id.is_not_null());
    }

    if options.moderator_view {
      query = query.filter(community_moderator::person_id.is_not_null());
    }
    // Only hide the read posts, if the saved_only is false. Otherwise ppl with the hide_read
    // setting wont be able to see saved posts.
    else if !options
      .local_user
      .map(|l| l.local_user.show_read_posts)
      .unwrap_or(true)
    {
      // Do not hide read posts when it is a user profile view
      if !options.is_profile_view {
        query = query.filter(post_read::post_id.is_null());
      }
    }

    if options.liked_only {
      query = query.filter(post_like::score.eq(1));
    } else if options.disliked_only {
      query = query.filter(post_like::score.eq(-1));
    }

    if options.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::language_id.is_not_null());

      // Don't show blocked communities or persons
      query = query.filter(community_block::person_id.is_null());
      if !options.moderator_view {
        query = query.filter(person_block::person_id.is_null());
      }
    }

    query = match options.sort.unwrap_or(SortType::Hot) {
      SortType::Active => {
        let query = order_and_page_filter_desc!(query, options, hot_rank_active);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::Hot => {
        let query = order_and_page_filter_desc!(query, options, hot_rank);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::Controversial => order_and_page_filter_desc!(query, options, controversy_rank),
      SortType::New => order_and_page_filter_desc!(query, options, published),
      SortType::Old => order_and_page_filter_asc!(query, options, published),
      SortType::NewComments => order_and_page_filter_desc!(query, options, newest_comment_time),
      SortType::MostComments => {
        let query = order_and_page_filter_desc!(query, options, comments);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopAll => {
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopYear => {
        let query = query.filter(post_aggregates::published.gt(now - 1.years()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopMonth => {
        let query = query.filter(post_aggregates::published.gt(now - 1.months()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopWeek => {
        let query = query.filter(post_aggregates::published.gt(now - 1.weeks()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopDay => {
        let query = query.filter(post_aggregates::published.gt(now - 1.days()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopHour => {
        let query = query.filter(post_aggregates::published.gt(now - 1.hours()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopSixHour => {
        let query = query.filter(post_aggregates::published.gt(now - 6.hours()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopTwelveHour => {
        let query = query.filter(post_aggregates::published.gt(now - 12.hours()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopThreeMonths => {
        let query = query.filter(post_aggregates::published.gt(now - 3.months()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopSixMonths => {
        let query = query.filter(post_aggregates::published.gt(now - 6.months()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
      SortType::TopNineMonths => {
        let query = query.filter(post_aggregates::published.gt(now - 9.months()));
        let query = order_and_page_filter_desc!(query, options, score);
        let query = order_and_page_filter_desc!(query, options, published);
        query
      }
    };

    let (limit, mut offset) = limit_and_offset(options.page, options.limit)?;
    if options.page_after.is_some() {
      // always skip exactly one post because that's the last post of the previous page
      // fixing the where clause is more difficult because we'd have to change only the last order-by-where clause
      // e.g. WHERE (featured_local<=, hot_rank<=, published<=) to WHERE (<=, <=, <)
      offset = 1;
    }
    query = query.limit(limit).offset(offset);

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));

    query.load::<PostViewTuple>(&mut conn).await
  };

  Queries::new(read, list)
}

impl PostView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    my_person_id: Option<PersonId>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    let mut res = queries()
      .read(pool, (post_id, my_person_id, is_mod_or_admin))
      .await?;

    // If a person is given, then my_vote, if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    if my_person_id.is_some() && res.my_vote.is_none() {
      res.my_vote = Some(0)
    };

    Ok(res)
  }
}

/// currently this is just a wrapper around post id, but should be seen as opaque from the client's perspective
/// stringified since we might want to use arbitrary info later, with a P prepended to prevent ossification
/// (api users love to make assumptions (e.g. parse stuff that looks like numbers as numbers) about apis that aren't part of the spec
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
pub struct PaginationCursor(String);
impl PaginationCursor {
  // get cursor for page after the given posts
  pub fn after(posts: &[PostView]) -> Option<PaginationCursor> {
    posts
      .last()
      .map(|p| PaginationCursor(format!("P{:x}", p.post.id.0)))
  }
  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    Ok(PaginationCursorData(
      PostAggregates::read(
        pool,
        PostId(
          i32::from_str_radix(&self.0[1..], 16)
            .map_err(|_| Error::QueryBuilderError("Could not parse pagination token".into()))?,
        ),
      )
      .await?,
    ))
  }
}

// currently we use a postaggregates struct as the pagination token.
// we only use some of the properties of the post aggregates, depending on which sort type we page by
#[derive(Clone)]
pub struct PaginationCursorData(PostAggregates);

#[derive(Default, Clone)]
pub struct PostQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<SortType>,
  pub creator_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  // if true, the query should be handled as if community_id was not given except adding the literal filter
  pub community_id_just_for_prefetch: bool,
  pub local_user: Option<&'a LocalUserView>,
  pub search_term: Option<String>,
  pub url_search: Option<String>,
  pub saved_only: bool,
  pub liked_only: bool,
  pub disliked_only: bool,
  pub moderator_view: bool,
  pub is_profile_view: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub page_after: Option<PaginationCursorData>,
  pub page_before_or_equal: Option<PaginationCursorData>,
}

impl<'a> PostQuery<'a> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PostView>, Error> {
    if self.listing_type == Some(ListingType::Subscribed)
      && self.community_id == None
      && self.local_user.is_some()
      && self.page_before_or_equal.is_none()
    {
      // first get one page for the most popular community to get an upper bound for the the page end for the real query
      // the reason this is needed is that when fetching posts for a single community PostgreSQL can optimize
      // the query to use an index on e.g. (=, >=, >=, >=) and fetch only LIMIT rows
      // but for the followed-communities query it has to query the index on (IN, >=, >=, >=)
      // which it currently can't do at all (as of PG 16). see the discussion here:
      // https://github.com/LemmyNet/lemmy/issues/2877#issuecomment-1673597190
      //
      // the results are correct no matter which community we fetch these for, since it basically covers the "worst case" of the whole page consisting of posts from one community
      // but using the largest community decreases the pagination-frame so make the real query more efficient
      use lemmy_db_schema::schema::{
        community_aggregates::dsl::{community_aggregates, community_id, users_active_month},
        community_follower::dsl::{
          community_follower,
          community_id as follower_community_id,
          person_id,
        },
      };
      let (limit, offset) = limit_and_offset(self.page, self.limit)?;
      if offset != 0 {
        return Err(Error::QueryBuilderError(
          "legacy pagination cannot be combined with v2 pagination".into(),
        ));
      }
      let self_person_id = self
        .local_user
        .expect("part of the above if")
        .local_user
        .person_id;
      let largest_subscribed = {
        let conn = &mut get_conn(pool).await?;
        community_follower
          .filter(person_id.eq(self_person_id))
          .inner_join(community_aggregates.on(community_id.eq(follower_community_id)))
          .order_by(users_active_month.desc())
          .select(community_id)
          .limit(1)
          .get_result::<CommunityId>(conn)
          .await
          .optional()?
      };
      let Some(largest_subscribed) = largest_subscribed else {
        // nothing subscribed to? no posts
        return Ok(vec![]);
      };
      let upper_bound_for_page_before = {
        let mut v = queries()
          .list(
            pool,
            PostQuery {
              community_id: Some(largest_subscribed),
              community_id_just_for_prefetch: true,
              ..self.clone()
            },
          )
          .await?;
        // take last element of array. if this query returned less than LIMIT elements,
        // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT results (return None)
        if (v.len() as i64) < limit {
          None
        } else {
          v.pop()
        }
      };
      if let Some(last_ele) = upper_bound_for_page_before {
        return queries()
          .list(
            pool,
            PostQuery {
              page_before_or_equal: Some(PaginationCursorData(last_ele.counts)),
              ..self.clone()
            },
          )
          .await;
      }
    }

    queries().list(pool, self).await
  }
}

impl JoinView for PostView {
  type JoinTuple = PostViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      post: a.0,
      creator: a.1,
      community: a.2,
      creator_banned_from_community: a.3,
      counts: a.4,
      subscribed: a.5,
      saved: a.6,
      read: a.7,
      creator_blocked: a.8,
      my_vote: a.9,
      unread_comments: a.10,
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    post_view::{PostQuery, PostView},
    structs::LocalUserView,
  };
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
    local_user_view: LocalUserView,
    inserted_blocked_person: Person,
    inserted_bot: Person,
    inserted_community: Community,
    inserted_post: Post,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> Data {
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
    let local_user_view = LocalUserView {
      local_user: inserted_local_user,
      person: inserted_person,
      counts: Default::default(),
    };

    Data {
      inserted_instance,
      local_user_view,
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
    let pool = &mut pool.into();
    let mut data = init_data(pool).await;

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let inserted_local_user =
      LocalUser::update(pool, data.local_user_view.local_user.id, &local_user_form)
        .await
        .unwrap();
    data.local_user_view.local_user = inserted_local_user;

    let read_post_listing = PostQuery {
      sort: (Some(SortType::New)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(data.local_user_view.person.id),
      false,
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

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(true),
      ..Default::default()
    };
    let inserted_local_user =
      LocalUser::update(pool, data.local_user_view.local_user.id, &local_user_form)
        .await
        .unwrap();
    data.local_user_view.local_user = inserted_local_user;

    let post_listings_with_bots = PostQuery {
      sort: (Some(SortType::New)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
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
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let read_post_listing_multiple_no_person = PostQuery {
      sort: (Some(SortType::New)),
      community_id: (Some(data.inserted_community.id)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    let read_post_listing_single_no_person =
      PostView::read(pool, data.inserted_post.id, None, false)
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
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let community_block = CommunityBlockForm {
      person_id: data.local_user_view.person.id,
      community_id: data.inserted_community.id,
    };
    CommunityBlock::block(pool, &community_block).await.unwrap();

    let read_post_listings_with_person_after_block = PostQuery {
      sort: (Some(SortType::New)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
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
    let pool = &mut pool.into();
    let mut data = init_data(pool).await;

    let post_like_form = PostLikeForm {
      post_id: data.inserted_post.id,
      person_id: data.local_user_view.person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(pool, &post_like_form).await.unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: data.inserted_post.id,
      person_id: data.local_user_view.person.id,
      published: inserted_post_like.published,
      score: 1,
    };
    assert_eq!(expected_post_like, inserted_post_like);

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(data.local_user_view.person.id),
      false,
    )
    .await
    .unwrap();

    let mut expected_post_with_upvote = expected_post_view(&data, pool).await;
    expected_post_with_upvote.my_vote = Some(1);
    expected_post_with_upvote.counts.score = 1;
    expected_post_with_upvote.counts.upvotes = 1;
    assert_eq!(expected_post_with_upvote, post_listing_single_with_person);

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let inserted_local_user =
      LocalUser::update(pool, data.local_user_view.local_user.id, &local_user_form)
        .await
        .unwrap();
    data.local_user_view.local_user = inserted_local_user;

    let read_post_listing = PostQuery {
      sort: (Some(SortType::New)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(1, read_post_listing.len());

    assert_eq!(expected_post_with_upvote, read_post_listing[0]);

    let read_liked_post_listing = PostQuery {
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      liked_only: (true),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(read_post_listing, read_liked_post_listing);

    let read_disliked_post_listing = PostQuery {
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      disliked_only: (true),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert!(read_disliked_post_listing.is_empty());

    let like_removed =
      PostLike::remove(pool, data.local_user_view.person.id, data.inserted_post.id)
        .await
        .unwrap();
    assert_eq!(1, like_removed);
    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_person_language() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let spanish_id = Language::read_id_from_code(pool, Some("es"))
      .await
      .unwrap()
      .unwrap();
    let post_spanish = PostInsertForm::builder()
      .name("asffgdsc".to_string())
      .creator_id(data.local_user_view.person.id)
      .community_id(data.inserted_community.id)
      .language_id(Some(spanish_id))
      .build();

    Post::create(pool, &post_spanish).await.unwrap();

    let post_listings_all = PostQuery {
      sort: (Some(SortType::New)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    // no language filters specified, all posts should be returned
    assert_eq!(3, post_listings_all.len());

    let french_id = Language::read_id_from_code(pool, Some("fr"))
      .await
      .unwrap()
      .unwrap();
    LocalUserLanguage::update(pool, vec![french_id], data.local_user_view.local_user.id)
      .await
      .unwrap();

    let post_listing_french = PostQuery {
      sort: (Some(SortType::New)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
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
      data.local_user_view.local_user.id,
    )
    .await
    .unwrap();
    let post_listings_french_und = PostQuery {
      sort: (Some(SortType::New)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
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
  async fn post_listings_removed() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let mut data = init_data(pool).await;

    // Remove the post
    Post::update(
      pool,
      data.inserted_post.id,
      &PostUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    // Make sure you don't see the removed post in the results
    let post_listings_no_admin = PostQuery {
      sort: Some(SortType::New),
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(1, post_listings_no_admin.len());

    // Removed post is shown to admins on profile page
    data.local_user_view.person.admin = true;
    let post_listings_is_admin = PostQuery {
      sort: Some(SortType::New),
      local_user: Some(&data.local_user_view),
      is_profile_view: true,
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(2, post_listings_is_admin.len());

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn post_listings_deleted() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    // Delete the post
    Post::update(
      pool,
      data.inserted_post.id,
      &PostUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    // Make sure you don't see the deleted post in the results
    let post_listings_no_creator = PostQuery {
      sort: Some(SortType::New),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    let not_contains_deleted = post_listings_no_creator
      .iter()
      .map(|p| p.post.id)
      .all(|p| p != data.inserted_post.id);
    assert!(not_contains_deleted);

    // Deleted post is shown to creator
    let post_listings_is_creator = PostQuery {
      sort: Some(SortType::New),
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    let contains_deleted = post_listings_is_creator
      .iter()
      .map(|p| p.post.id)
      .any(|p| p == data.inserted_post.id);
    assert!(contains_deleted);

    cleanup(data, pool).await;
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) {
    let num_deleted = Post::delete(pool, data.inserted_post.id).await.unwrap();
    Community::delete(pool, data.inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, data.local_user_view.person.id)
      .await
      .unwrap();
    Person::delete(pool, data.inserted_bot.id).await.unwrap();
    Person::delete(pool, data.inserted_blocked_person.id)
      .await
      .unwrap();
    Instance::delete(pool, data.inserted_instance.id)
      .await
      .unwrap();
    assert_eq!(1, num_deleted);
  }

  async fn expected_post_view(data: &Data, pool: &mut DbPool<'_>) -> PostView {
    let (inserted_person, inserted_community, inserted_post) = (
      &data.local_user_view.person,
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
        controversy_rank: 0.0,
        community_id: inserted_post.community_id,
        creator_id: inserted_post.creator_id,
      },
      subscribed: SubscribedType::NotSubscribed,
      read: false,
      saved: false,
      creator_blocked: false,
    }
  }
}
