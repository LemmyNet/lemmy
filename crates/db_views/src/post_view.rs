use crate::structs::{LocalUserView, PaginationCursor, PostView};
use diesel::{
  data_types::PgInterval,
  debug_query,
  dsl::{self, exists, not, InnerJoin, InnerJoinQuerySource, IntervalDsl},
  expression::AsExpression,
  pg::Pg,
  query_builder::{AstPass, Query, QueryFragment},
  result::Error,
  sql_function,
  sql_types::{self as st, SingleValue, SqlType},
  BoolExpressionMethods,
  Expression,
  ExpressionMethods,
  IntoSql,
  NullableExpressionMethods,
  OptionalExtension,
  PgTextExpressionMethods,
  QueryDsl,
  QueryId,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PostAggregates,
  newtypes::{CommunityId, PersonId, PostId},
  schema::{
    community,
    community_aggregates,
    community_block,
    community_follower,
    community_moderator,
    community_person_ban,
    instance_block,
    local_user,
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
  utils::{
    boxed_meth,
    fuzzy_search,
    get_conn,
    limit_and_offset,
    now,
    BoxExpr,
    DbPool,
    FilterVarEq,
  },
  ListingType,
  SortType,
};
use lemmy_utils::type_chain;
use tracing::debug;

sql_function!(fn coalesce(x: st::Nullable<st::BigInt>, y: st::BigInt) -> st::BigInt);

#[derive(Clone, Copy)]
enum Ord {
  Asc,
  Desc,
}

trait OrderAndPageFilter {
  fn order_and_page_filter<'a>(
    &self,
    query: BoxedQuery<'a>,
    range: [Option<&PaginationCursorData>; 2],
  ) -> BoxedQuery<'a>;
}

impl<C, T, F> OrderAndPageFilter for (Ord, C, F)
where
  for<'a> BoxedQuery<'a>: boxed_meth::ThenOrderDsl<dsl::Desc<C>>
    + boxed_meth::ThenOrderDsl<dsl::Asc<C>>
    + boxed_meth::FilterDsl<dsl::GtEq<C, T>>
    + boxed_meth::FilterDsl<dsl::LtEq<C, T>>,
  C: Expression + Copy,
  C::SqlType: SqlType + SingleValue,
  T: AsExpression<C::SqlType>,
  F: Fn(&PostAggregates) -> T + Copy,
{
  fn order_and_page_filter<'a>(
    &self,
    query: BoxedQuery<'a>,
    [first, last]: [Option<&PaginationCursorData>; 2],
  ) -> BoxedQuery<'a> {
    let (order, column, getter) = *self;
    let (mut query, min, max) = match order {
      Ord::Desc => (query.then_order_by(column.desc()), last, first),
      Ord::Asc => (query.then_order_by(column.asc()), first, last),
    };
    if let Some(min) = min {
      query = query.filter(column.ge(getter(&min.0)));
    }
    if let Some(max) = max {
      query = query.filter(column.le(getter(&max.0)));
    }
    query
  }
}

macro_rules! desc {
  ($name:ident) => {{
    &(Ord::Desc, post_aggregates::$name, |e: &PostAggregates| {
      e.$name
    })
  }};
}

macro_rules! asc {
  ($name:ident) => {{
    &(Ord::Asc, post_aggregates::$name, |e: &PostAggregates| {
      e.$name
    })
  }};
}

type BoxedQuery<'a> = dsl::IntoBoxed<
  'a,
  type_chain!(post_aggregates::table.InnerJoin<person::table>.InnerJoin<community::table>.InnerJoin<post::table>),
  Pg,
>;

type QS = type_chain!(post_aggregates::table.InnerJoinQuerySource<person::table>.InnerJoinQuerySource<community::table>.InnerJoinQuerySource<post::table>);

fn new_query<'a>() -> BoxedQuery<'a> {
  post_aggregates::table
    .inner_join(person::table)
    .inner_join(community::table)
    .inner_join(post::table)
    .into_boxed()
}

struct SelectionBuilder {
  subscribe: Box<dyn Fn() -> BoxExpr<QS, st::Nullable<st::Bool>>>,
  saved: BoxExpr<QS, st::Bool>,
  read: BoxExpr<QS, st::Bool>,
  creator_blocked: BoxExpr<QS, st::Bool>,
  my_vote: BoxExpr<QS, st::Nullable<st::SmallInt>>,
  me: Option<PersonId>,
}

impl SelectionBuilder {
  fn new(me: Option<PersonId>) -> Self {
    if let Some(me) = me {
      SelectionBuilder {
        subscribe: Box::new(move || {
          Box::new(
            community_follower::table
              .find((me, post_aggregates::community_id))
              .select(community_follower::pending.nullable())
              .single_value(),
          )
        }),
        saved: Box::new(exists(
          post_saved::table.find((me, post_aggregates::post_id)),
        )),
        read: Box::new(exists(
          post_read::table.find((me, post_aggregates::post_id)),
        )),
        creator_blocked: Box::new(exists(
          person_block::table.find((me, post_aggregates::creator_id)),
        )),
        my_vote: Box::new(
          post_like::table
            .find((me, post_aggregates::post_id))
            .select(post_like::score.nullable())
            .single_value(),
        ),
        me: Some(me),
      }
    } else {
      SelectionBuilder {
        subscribe: Box::new(|| Box::new(None::<bool>.into_sql::<st::Nullable<st::Bool>>())),
        saved: Box::new(false.into_sql::<st::Bool>()),
        read: Box::new(false.into_sql::<st::Bool>()),
        creator_blocked: Box::new(false.into_sql::<st::Bool>()),
        my_vote: Box::new(None::<i16>.into_sql::<st::Nullable<st::SmallInt>>()),
        me: None,
      }
    }
  }

  fn build(
    self,
  ) -> BoxExpr<
    QS,
    (
      post::SqlType,
      person::SqlType,
      community::SqlType,
      st::Bool,
      st::Bool,
      st::Bool,
      post_aggregates::SqlType,
      st::Nullable<st::Bool>,
      st::Bool,
      st::Bool,
      st::Bool,
      st::Nullable<st::SmallInt>,
      st::BigInt,
    ),
  > {
    let read_comments: BoxExpr<_, st::Nullable<st::BigInt>> = if let Some(me) = self.me {
      Box::new(
        person_post_aggregates::table
          .find((me, post_aggregates::post_id))
          .select(person_post_aggregates::read_comments.nullable())
          .single_value(),
      )
    } else {
      Box::new(None::<i64>.into_sql::<st::Nullable<st::BigInt>>())
    };

    let creator_banned_from_community = exists(
      community_person_ban::table
        .find((post_aggregates::creator_id, post_aggregates::community_id)),
    );
    let creator_is_moderator = exists(
      community_moderator::table.find((post_aggregates::creator_id, post_aggregates::community_id)),
    );
    let creator_is_admin = exists(
      local_user::table
        .filter(local_user::person_id.eq(post_aggregates::creator_id))
        .filter(local_user::admin),
    );

    Box::new((
      post::all_columns,
      person::all_columns,
      community::all_columns,
      creator_banned_from_community,
      creator_is_moderator,
      creator_is_admin,
      post_aggregates::all_columns,
      (self.subscribe)(),
      self.saved,
      self.read,
      self.creator_blocked,
      self.my_vote,
      post_aggregates::comments - coalesce(read_comments, 0),
    ))
  }
}

fn not_removed() -> dsl::not<dsl::Or<community::removed, post::removed>> {
  not(community::removed.or(post::removed))
}

fn not_deleted() -> dsl::not<dsl::Or<community::deleted, post::deleted>> {
  not(community::deleted.or(post::deleted))
}

fn is_creator(
  me: Option<PersonId>,
) -> dsl::Eq<dsl::Nullable<post_aggregates::creator_id>, Option<PersonId>> {
  post_aggregates::creator_id.nullable().eq(me)
}

impl PostView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    local_user_view: Option<&LocalUserView>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    let me = local_user_view.map(|l| l.person.id);
    let mut query = new_query().filter(post_aggregates::post_id.eq(post_id));
    if !is_mod_or_admin {
      query = query.filter(is_creator(me).or(not_removed().and(not_deleted())));
    }
    let query = query.select(SelectionBuilder::new(me).build());
    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));
    query.first(&mut *get_conn(pool).await?).await
  }
}

impl PaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &PostView) -> PaginationCursor {
    // hex encoding to prevent ossification
    PaginationCursor(format!("P{:x}", view.counts.post_id.0))
  }
  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    Ok(PaginationCursorData(
      PostAggregates::read(
        pool,
        PostId(
          self
            .0
            .get(1..)
            .and_then(|e| i32::from_str_radix(e, 16).ok())
            .ok_or_else(|| Error::QueryBuilderError("Could not parse pagination token".into()))?,
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
    let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    if offset != 0 && self.page_after.is_some() {
      return Err(Error::QueryBuilderError(
        "legacy pagination cannot be combined with v2 pagination".into(),
      ));
    }

    let build_query = |page_before_or_equal: Option<PaginationCursorData>| {
      let l = self.local_user.map(|l| &l.local_user);
      let admin = l.map(|l| l.admin).unwrap_or(false);
      let show_nsfw = l.map(|l| l.show_nsfw).unwrap_or(false);
      let show_bot_accounts = l.map(|l| l.show_bot_accounts).unwrap_or(true);
      let show_read_posts = l.map(|l| l.show_read_posts).unwrap_or(true);

      let mut query = new_query();
      let mut selection_builder = SelectionBuilder::new(l.map(|l| l.person_id));

      let i_am_creator = post_aggregates::creator_id
        .nullable()
        .eq(l.map(|l| l.person_id));
      query = query.filter(i_am_creator.or(not_deleted()));

      if !(admin && self.is_profile_view) {
        query = query.filter(not_removed());
      }

      let mut i_am_moderator: BoxExpr<_, st::Bool>;

      if let Some(me) = self.local_user {
        let l = &me.local_user;
        i_am_moderator = Box::new(exists(
          community_moderator::table.find((l.person_id, post_aggregates::community_id)),
        ));
        if self.listing_type != Some(ListingType::ModeratorView) {
          query = query.filter(exists(
            local_user_language::table.find((l.id, post::language_id)),
          ));
          query = query.filter_var_eq(&mut selection_builder.creator_blocked, false);
          query = query.filter(not(exists(
            community_block::table.find((l.person_id, post_aggregates::community_id)),
          )));
          query = query.filter(not(exists(
            instance_block::table.find((l.person_id, post_aggregates::instance_id)),
          )));
        }
      } else {
        i_am_moderator = Box::new(false.into_sql::<st::Bool>());
      }

      if self.listing_type == Some(ListingType::ModeratorView) {
        query = query.filter_var_eq(&mut i_am_moderator, true);
      } else {
        query =
          query.filter(not(community::hidden).or((selection_builder.subscribe)().is_not_null()))
      }

      if self.listing_type == Some(ListingType::Subscribed) {
        query = query.filter((selection_builder.subscribe)().is_not_null());
      }
      if self.listing_type == Some(ListingType::Local) {
        query = query.filter(community::local);
      }

      if self.saved_only {
        query = query.filter_var_eq(&mut selection_builder.saved, true);
      }
      if self.liked_only {
        query = query.filter_var_eq(&mut selection_builder.my_vote, 1);
      }
      if self.disliked_only {
        query = query.filter_var_eq(&mut selection_builder.my_vote, -1);
      }
      if !(show_read_posts || self.saved_only || self.is_profile_view) {
        query = query.filter_var_eq(&mut selection_builder.read, false);
      }

      if let Some(search_term) = &self.search_term {
        let searcher = fuzzy_search(search_term);
        query = query.filter(
          post::name
            .ilike(searcher.clone())
            .or(post::body.ilike(searcher)),
        );
      }
      if let Some(community_id) = self.community_id {
        query = query.filter(post_aggregates::community_id.eq(community_id));
      }
      if let Some(creator_id) = self.creator_id {
        query = query.filter(post_aggregates::creator_id.eq(creator_id));
      }
      if let Some(url_search) = &self.url_search {
        query = query.filter(post::url.eq(url_search));
      }
      if !show_nsfw {
        query = query.filter(not(post::nsfw.or(community::nsfw)));
      }
      if !show_bot_accounts {
        query = query.filter(not(person::bot_account));
      }

      let range = [self.page_after.as_ref(), page_before_or_equal.as_ref()];
      let top: &[&dyn OrderAndPageFilter] = &[desc!(score), desc!(published)];

      let featured_sort: &dyn OrderAndPageFilter = if self.community_id.is_some() {
        desc!(featured_community)
      } else {
        desc!(featured_local)
      };
      let (sorts, interval): (&[&dyn OrderAndPageFilter], Option<PgInterval>) =
        match self.sort.unwrap_or(SortType::Hot) {
          SortType::Active => (&[desc!(hot_rank_active), desc!(published)], None),
          SortType::Hot => (&[desc!(hot_rank), desc!(published)], None),
          SortType::Scaled => (&[desc!(scaled_rank), desc!(published)], None),
          SortType::Controversial => (&[desc!(controversy_rank), desc!(published)], None),
          SortType::New => (&[desc!(published)], None),
          SortType::Old => (&[asc!(published)], None),
          SortType::NewComments => (&[desc!(newest_comment_time)], None),
          SortType::MostComments => (&[desc!(comments), desc!(published)], None),
          SortType::TopAll => (&[desc!(score), desc!(published)], None),
          SortType::TopYear => (top, Some(1.years())),
          SortType::TopMonth => (top, Some(1.months())),
          SortType::TopWeek => (top, Some(1.weeks())),
          SortType::TopDay => (top, Some(1.days())),
          SortType::TopHour => (top, Some(1.hours())),
          SortType::TopSixHour => (top, Some(6.hours())),
          SortType::TopTwelveHour => (top, Some(12.hours())),
          SortType::TopThreeMonths => (top, Some(3.months())),
          SortType::TopSixMonths => (top, Some(6.months())),
          SortType::TopNineMonths => (top, Some(9.months())),
        };

      if let Some(interval) = interval {
        query = query.filter(post_aggregates::published.gt(now() - interval));
      }

      for i in [&[featured_sort], sorts].into_iter().flatten() {
        query = i.order_and_page_filter(query, range);
      }

      query
        .limit(limit)
        .offset(if self.page_after.is_some() {
          // always skip exactly one post because that's the last post of the previous page
          // fixing the where clause is more difficult because we'd have to change only the last order-by-where clause
          // e.g. WHERE (featured_local<=, hot_rank<=, published<=) to WHERE (<=, <=, <)
          1
        } else {
          offset
        })
        .select(selection_builder.build())
    };

    let mut page_before_or_equal = self.page_before_or_equal;

    if let (Some(me), Some(ListingType::Subscribed), None, None) = (
      self.local_user,
      self.listing_type,
      self.community_id,
      &page_before_or_equal,
    ) {
      // first get one page for the most popular community to get an upper bound for the the page end for the real query
      // the reason this is needed is that when fetching posts for a single community PostgreSQL can optimize
      // the query to use an index on e.g. (=, >=, >=, >=) and fetch only LIMIT rows
      // but for the followed-communities query it has to query the index on (IN, >=, >=, >=)
      // which it currently can't do at all (as of PG 16). see the discussion here:
      // https://github.com/LemmyNet/lemmy/issues/2877#issuecomment-1673597190
      //
      // the results are correct no matter which community we fetch these for, since it basically covers the "worst case" of the whole page consisting of posts from one community
      // but using the largest community decreases the pagination-frame so make the real query more efficient.

      /// Gets the number of rows and the last row for the wrapped query
      #[derive(QueryId)]
      struct PrefetchQuery<T>(T);

      impl<T: Query> Query for PrefetchQuery<T> {
        type SqlType = (<dsl::CountStar as Expression>::SqlType, T::SqlType);
      }

      impl<T: QueryFragment<Pg>> QueryFragment<Pg> for PrefetchQuery<T> {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> Result<(), Error> {
          let count = "(SELECT count(*) FROM q)";

          // Set `q` to the wrapped query.
          out.push_sql("WITH q AS (");
          self.0.walk_ast(out.reborrow())?;
          out.push_sql(") ");

          // Select the number of rows and the wrapped query's selection, then omit `{count} - 1` rows
          // to get the last row. Offset can't be negative, so `GREATEST` changes it from -1 to 0 when
          // there's 0 rows.
          out.push_sql(&format!(
            "SELECT {count}, * FROM q OFFSET GREATEST(0, {count} - 1)"
          ));

          Ok(())
        }
      }

      let largest_subscribed: Option<CommunityId> = community_aggregates::table
        .filter(exists(
          community_follower::table.find((me.person.id, community_aggregates::community_id)),
        ))
        .order_by(community_aggregates::users_active_month.desc())
        .select(community_aggregates::community_id)
        .first(&mut *get_conn(pool).await?)
        .await
        .optional()?;

      let Some(largest_subscribed) = largest_subscribed else {
        // nothing subscribed to? no posts
        return Ok(vec![]);
      };

      let result: Option<(i64, PostAggregates)> = PrefetchQuery(
        build_query(None)
          .filter(post_aggregates::community_id.eq(largest_subscribed))
          .select(post_aggregates::all_columns),
      )
      .get_result(&mut *get_conn(pool).await?)
      .await
      .optional()?;

      // take last element of array. if this query returned less than LIMIT elements,
      // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT results (return original query)
      if let Some((len, last_item)) = result {
        if len >= limit {
          page_before_or_equal = Some(PaginationCursorData(last_item));
        }
      }
    };

    build_query(page_before_or_equal)
      .load(&mut *get_conn(pool).await?)
      .await
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
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      community_block::{CommunityBlock, CommunityBlockForm},
      instance::Instance,
      instance_block::{InstanceBlock, InstanceBlockForm},
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm, PostUpdateForm},
    },
    traits::{Blockable, Crud, Joinable, Likeable},
    utils::{build_db_pool_for_tests, DbPool, RANK_DEFAULT},
    SortType,
    SubscribedType,
  };
  use serial_test::serial;

  struct Data {
    inserted_instance: Instance,
    local_user_view: LocalUserView,
    blocked_local_user_view: LocalUserView,
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
      .admin(Some(true))
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

    let blocked_local_user = LocalUserInsertForm::builder()
      .person_id(inserted_blocked_person.id)
      .password_encrypted(String::new())
      .build();

    let inserted_blocked_local_user = LocalUser::create(pool, &blocked_local_user).await.unwrap();

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
    let blocked_local_user_view = LocalUserView {
      local_user: inserted_blocked_local_user,
      person: inserted_blocked_person,
      counts: Default::default(),
    };

    Data {
      inserted_instance,
      local_user_view,
      blocked_local_user_view,
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
      Some(&data.local_user_view),
      false,
    )
    .await
    .unwrap();

    let mut expected_post_listing_with_user = expected_post_view(&data, pool).await;

    // Should be only one person, IE the bot post, and blocked should be missing
    assert_eq!(1, read_post_listing.len());

    assert_eq!(expected_post_listing_with_user, read_post_listing[0]);
    expected_post_listing_with_user.my_vote = None;
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
      post_id: data.inserted_post.id,
      person_id: data.local_user_view.person.id,
      published: inserted_post_like.published,
      score: 1,
    };
    assert_eq!(expected_post_like, inserted_post_like);

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(&data.local_user_view),
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
  async fn creator_is_moderator() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    // Make one of the inserted persons a moderator
    let person_id = data.local_user_view.person.id;
    let community_id = data.inserted_community.id;
    let form = CommunityModeratorForm {
      community_id,
      person_id,
    };
    CommunityModerator::join(pool, &form).await.unwrap();

    let post_listing = PostQuery {
      sort: (Some(SortType::Old)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    assert_eq!(post_listing[0].creator.name, "tegan");
    assert!(post_listing[0].creator_is_moderator);

    assert_eq!(post_listing[1].creator.name, "mybot");
    assert!(!post_listing[1].creator_is_moderator);

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn creator_is_admin() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let post_listing = PostQuery {
      sort: (Some(SortType::Old)),
      community_id: (Some(data.inserted_community.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    assert_eq!(post_listing[0].creator.name, "tegan");
    assert!(post_listing[0].creator_is_admin);

    assert_eq!(post_listing[1].creator.name, "mybot");
    assert!(!post_listing[1].creator_is_admin);

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
    data.local_user_view.local_user.admin = true;
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

    // Deleted post is hidden from other users
    let post_listings_is_other_user = PostQuery {
      sort: Some(SortType::New),
      local_user: Some(&data.blocked_local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    let not_contains_deleted_2 = post_listings_is_other_user
      .iter()
      .map(|p| p.post.id)
      .all(|p| p != data.inserted_post.id);
    assert!(not_contains_deleted_2);

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

  #[tokio::test]
  #[serial]
  async fn post_listing_instance_block() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let blocked_instance = Instance::read_or_create(pool, "another_domain.tld".to_string())
      .await
      .unwrap();

    let community_form = CommunityInsertForm::builder()
      .name("test_community_4".to_string())
      .title("none".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(blocked_instance.id)
      .build();
    let inserted_community = Community::create(pool, &community_form).await.unwrap();

    let post_form = PostInsertForm::builder()
      .name("blocked instance post".to_string())
      .creator_id(data.inserted_bot.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(1)))
      .build();

    let post_from_blocked_instance = Post::create(pool, &post_form).await.unwrap();

    // no instance block, should return all posts
    let post_listings_all = PostQuery {
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(post_listings_all.len(), 3);

    // block the instance
    let block_form = InstanceBlockForm {
      person_id: data.local_user_view.person.id,
      instance_id: blocked_instance.id,
    };
    InstanceBlock::block(pool, &block_form).await.unwrap();

    // now posts from communities on that instance should be hidden
    let post_listings_blocked = PostQuery {
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(post_listings_blocked.len(), 2);
    assert_ne!(
      post_listings_blocked[0].post.id,
      post_from_blocked_instance.id
    );
    assert_ne!(
      post_listings_blocked[1].post.id,
      post_from_blocked_instance.id
    );

    // after unblocking it should return all posts again
    InstanceBlock::unblock(pool, &block_form).await.unwrap();
    let post_listings_blocked = PostQuery {
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(post_listings_blocked.len(), 3);

    Instance::delete(pool, blocked_instance.id).await.unwrap();
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
    Person::delete(pool, data.blocked_local_user_view.person.id)
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
      creator_is_moderator: false,
      creator_is_admin: true,
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
        hot_rank: RANK_DEFAULT,
        hot_rank_active: RANK_DEFAULT,
        controversy_rank: 0.0,
        scaled_rank: RANK_DEFAULT,
        community_id: inserted_post.community_id,
        creator_id: inserted_post.creator_id,
        instance_id: data.inserted_instance.id,
      },
      subscribed: SubscribedType::NotSubscribed,
      read: false,
      saved: false,
      creator_blocked: false,
    }
  }
}
