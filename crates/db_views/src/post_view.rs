use crate::structs::{LocalUserView, PaginationCursor, PostView};
use diesel::{
  debug_query,
  dsl::{self, exists, not, IntervalDsl},
  expression::AsExpression,
  pg::Pg,
  result::Error,
  sql_function,
  sql_types::{self, SingleValue, SqlType, Timestamptz},
  BoolExpressionMethods,
  BoxableExpression,
  Expression,
  ExpressionMethods,
  IntoSql,
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
    instance_block,
    local_user,
    local_user_language,
    person,
    person_block,
    person_post_aggregates,
    post,
    post_aggregates::{self, newest_comment_time},
    post_like,
    post_read,
    post_saved,
  },
  utils::{fuzzy_search, get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  ListingType,
  SortType,
};
use tracing::debug;

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

fn order_and_page_filter_desc<Q, C, T>(
  query: Q,
  column: C,
  options: &PostQuery,
  getter: impl Fn(&PostAggregates) -> T,
) -> Q
where
  Q: diesel::query_dsl::methods::ThenOrderDsl<dsl::Desc<C>, Output = Q>
    + diesel::query_dsl::methods::ThenOrderDsl<dsl::Asc<C>, Output = Q>
    + diesel::query_dsl::methods::FilterDsl<dsl::GtEq<C, T>, Output = Q>
    + diesel::query_dsl::methods::FilterDsl<dsl::LtEq<C, T>, Output = Q>,
  C: Expression + Copy,
  C::SqlType: SingleValue + SqlType,
  T: AsExpression<C::SqlType>,
{
  let mut query = query.then_order_by(column.desc());
  if let Some(before) = &options.page_before_or_equal {
    query = query.filter(column.ge(getter(&before.0)));
  }
  if let Some(after) = &options.page_after {
    query = query.filter(column.le(getter(&after.0)));
  }
  query
}

fn order_and_page_filter_asc<Q, C, T>(
  query: Q,
  column: C,
  options: &PostQuery,
  getter: impl Fn(&PostAggregates) -> T,
) -> Q
where
  Q: diesel::query_dsl::methods::ThenOrderDsl<dsl::Asc<C>, Output = Q>
    + diesel::query_dsl::methods::FilterDsl<dsl::LtEq<C, T>, Output = Q>
    + diesel::query_dsl::methods::FilterDsl<dsl::GtEq<C, T>, Output = Q>,
  C: Expression + Copy,
  C::SqlType: SingleValue + SqlType,
  T: AsExpression<C::SqlType>,
{
  let mut query = query.then_order_by(column.asc());
  if let Some(before) = &options.page_before_or_equal {
    query = query.filter(column.le(getter(&before.0)));
  }
  if let Some(after) = &options.page_after {
    query = query.filter(column.ge(getter(&after.0)));
  }
  query
}

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PostView, (PostId, Option<PersonId>, bool)>,
  impl ListFn<'a, PostView, PostQuery<'a>>,
> {
  let is_creator_banned_from_community = exists(
    community_person_ban::table.filter(
      post_aggregates::community_id
        .eq(community_person_ban::community_id)
        .and(community_person_ban::person_id.eq(post_aggregates::creator_id)),
    ),
  );
  let creator_is_moderator = exists(
    community_moderator::table.filter(
      post_aggregates::community_id
        .eq(community_moderator::community_id)
        .and(community_moderator::person_id.eq(post_aggregates::creator_id)),
    ),
  );

  let creator_is_admin = exists(
    local_user::table.filter(
      post_aggregates::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    ),
  );

  let is_saved = |person_id| {
    exists(
      post_saved::table.filter(
        post_aggregates::post_id
          .eq(post_saved::post_id)
          .and(post_saved::person_id.eq(person_id)),
      ),
    )
  };

  let is_read = |person_id| {
    exists(
      post_read::table.filter(
        post_aggregates::post_id
          .eq(post_read::post_id)
          .and(post_read::person_id.eq(person_id)),
      ),
    )
  };

  let is_creator_blocked = |person_id| {
    exists(
      person_block::table.filter(
        post_aggregates::creator_id
          .eq(person_block::target_id)
          .and(person_block::person_id.eq(person_id)),
      ),
    )
  };

  let score = |person_id| {
    post_like::table
      .filter(
        post_aggregates::post_id
          .eq(post_like::post_id)
          .and(post_like::person_id.eq(person_id)),
      )
      .select(post_like::score.nullable())
      .single_value()
  };

  let all_joins = move |query: post_aggregates::BoxedQuery<'a, Pg>,
                        my_person_id: Option<PersonId>,
                        saved_only: bool| {
    let is_saved_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if saved_only {
        Box::new(true.into_sql::<sql_types::Bool>())
      } else if let Some(person_id) = my_person_id {
        Box::new(is_saved(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    let is_read_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_read(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    let is_creator_blocked_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_creator_blocked(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    let subscribed_type_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::Bool>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(
        community_follower::table
          .filter(
            post_aggregates::community_id
              .eq(community_follower::community_id)
              .and(community_follower::person_id.eq(person_id)),
          )
          .select(community_follower::pending.nullable())
          .single_value(),
      )
    } else {
      Box::new(None::<bool>.into_sql::<sql_types::Nullable<sql_types::Bool>>())
    };

    let score_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(score(person_id))
    } else {
      Box::new(None::<i16>.into_sql::<sql_types::Nullable<sql_types::SmallInt>>())
    };

    let read_comments: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::BigInt>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(
        person_post_aggregates::table
          .filter(
            post_aggregates::post_id
              .eq(person_post_aggregates::post_id)
              .and(person_post_aggregates::person_id.eq(person_id)),
          )
          .select(person_post_aggregates::read_comments.nullable())
          .single_value(),
      )
    } else {
      Box::new(None::<i64>.into_sql::<sql_types::Nullable<sql_types::BigInt>>())
    };

    query
      .inner_join(person::table)
      .inner_join(community::table)
      .inner_join(post::table)
      .select((
        post::all_columns,
        person::all_columns,
        community::all_columns,
        is_creator_banned_from_community,
        creator_is_moderator,
        creator_is_admin,
        post_aggregates::all_columns,
        subscribed_type_selection,
        is_saved_selection,
        is_read_selection,
        is_creator_blocked_selection,
        score_selection,
        coalesce(
          post_aggregates::comments.nullable() - read_comments,
          post_aggregates::comments,
        ),
      ))
  };

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
        false,
      );

      // Hide deleted and removed for non-admins or mods
      if !is_mod_or_admin {
        query = query
          .filter(
            community::removed
              .eq(false)
              .or(post::creator_id.eq(person_id_join)),
          )
          .filter(
            post::removed
              .eq(false)
              .or(post::creator_id.eq(person_id_join)),
          )
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

      query.first::<PostView>(&mut conn).await
    };

  let list = move |mut conn: DbConn<'a>, options: PostQuery<'a>| async move {
    let my_person_id = options.local_user.map(|l| l.person.id);
    let my_local_user_id = options.local_user.map(|l| l.local_user.id);

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));
    let local_user_id_join = my_local_user_id.unwrap_or(LocalUserId(-1));

    let mut query = all_joins(
      post_aggregates::table.into_boxed(),
      my_person_id,
      options.saved_only,
    );

    // hide posts from deleted communities
    query = query.filter(community::deleted.eq(false));

    // only show deleted posts to creator
    if let Some(person_id) = my_person_id {
      query = query.filter(post::deleted.eq(false).or(post::creator_id.eq(person_id)));
    } else {
      query = query.filter(post::deleted.eq(false));
    }

    let is_admin = options
      .local_user
      .map(|l| l.local_user.admin)
      .unwrap_or(false);
    // only show removed posts to admin when viewing user profile
    if !(options.creator_id.is_some() && is_admin) {
      query = query
        .filter(community::removed.eq(false))
        .filter(post::removed.eq(false));
    }
    if options.community_id.is_none() || options.community_id_just_for_prefetch {
      query = order_and_page_filter_desc(query, post_aggregates::featured_local, &options, |e| {
        e.featured_local
      });
    } else {
      query =
        order_and_page_filter_desc(query, post_aggregates::featured_community, &options, |e| {
          e.featured_community
        });
    }
    if let Some(community_id) = options.community_id {
      query = query.filter(post_aggregates::community_id.eq(community_id));
    }

    if let Some(creator_id) = options.creator_id {
      query = query.filter(post_aggregates::creator_id.eq(creator_id));
    }

    if let Some(listing_type) = options.listing_type {
      if let Some(person_id) = my_person_id {
        let is_subscribed = exists(
          community_follower::table.filter(
            post_aggregates::community_id
              .eq(community_follower::community_id)
              .and(community_follower::person_id.eq(person_id)),
          ),
        );
        match listing_type {
          ListingType::Subscribed => query = query.filter(is_subscribed),
          ListingType::Local => {
            query = query
              .filter(community::local.eq(true))
              .filter(community::hidden.eq(false).or(is_subscribed));
          }
          ListingType::All => query = query.filter(community::hidden.eq(false).or(is_subscribed)),
          ListingType::ModeratorView => {
            query = query.filter(exists(
              community_moderator::table.filter(
                post::community_id
                  .eq(community_moderator::community_id)
                  .and(community_moderator::person_id.eq(person_id)),
              ),
            ));
          }
        }
      }
      // If your person_id is missing, only show local
      else {
        match listing_type {
          ListingType::Local => {
            query = query
              .filter(community::local.eq(true))
              .filter(community::hidden.eq(false));
          }
          _ => query = query.filter(community::hidden.eq(false)),
        }
      }
    } else {
      query = query.filter(community::hidden.eq(false));
    }

    if let Some(url_search) = &options.url_search {
      query = query.filter(post::url.eq(url_search));
    }

    if let Some(search_term) = &options.search_term {
      let searcher = fuzzy_search(search_term);
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

    if let (true, Some(person_id)) = (options.saved_only, my_person_id) {
      query = query.filter(is_saved(person_id));
    }
    // Only hide the read posts, if the saved_only is false. Otherwise ppl with the hide_read
    // setting wont be able to see saved posts.
    else if !options
      .local_user
      .map(|l| l.local_user.show_read_posts)
      .unwrap_or(true)
    {
      // Do not hide read posts when it is a user profile view
      if let (Some(_creator_id), Some(person_id)) = (options.creator_id, my_person_id) {
        query = query.filter(not(is_read(person_id)));
      }
    }

    if let Some(person_id) = my_person_id {
      if options.liked_only {
        query = query.filter(score(person_id).eq(1));
      } else if options.disliked_only {
        query = query.filter(score(person_id).eq(-1));
      }
    };

    // Dont filter blocks or missing languages for moderator view type
    if let (Some(person_id), false) = (
      my_person_id,
      options.listing_type.unwrap_or_default() == ListingType::ModeratorView,
    ) {
      // Filter out the rows with missing languages
      query = query.filter(exists(
        local_user_language::table.filter(
          post::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      ));

      // Don't show blocked instances, communities or persons
      query = query.filter(not(exists(
        community_block::table.filter(
          post_aggregates::community_id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )));
      query = query.filter(not(exists(
        instance_block::table.filter(
          post_aggregates::instance_id
            .eq(instance_block::instance_id)
            .and(instance_block::person_id.eq(person_id_join)),
        ),
      )));
      query = query.filter(not(is_creator_blocked(person_id)));
    }
    let now = diesel::dsl::now.into_sql::<Timestamptz>();

    {
      use post_aggregates::{
        comments,
        controversy_rank,
        hot_rank,
        hot_rank_active,
        published,
        scaled_rank,
        score,
      };
      match options.sort.as_ref().unwrap_or(&SortType::Hot) {
        SortType::Active => {
          query =
            order_and_page_filter_desc(query, hot_rank_active, &options, |e| e.hot_rank_active);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        SortType::Hot => {
          query = order_and_page_filter_desc(query, hot_rank, &options, |e| e.hot_rank);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        SortType::Scaled => {
          query = order_and_page_filter_desc(query, scaled_rank, &options, |e| e.scaled_rank);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        SortType::Controversial => {
          query =
            order_and_page_filter_desc(query, controversy_rank, &options, |e| e.controversy_rank);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        SortType::New => {
          query = order_and_page_filter_desc(query, published, &options, |e| e.published)
        }
        SortType::Old => {
          query = order_and_page_filter_asc(query, published, &options, |e| e.published)
        }
        SortType::NewComments => {
          query = order_and_page_filter_desc(query, newest_comment_time, &options, |e| {
            e.newest_comment_time
          })
        }
        SortType::MostComments => {
          query = order_and_page_filter_desc(query, comments, &options, |e| e.comments);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        SortType::TopAll => {
          query = order_and_page_filter_desc(query, score, &options, |e| e.score);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
        o @ (SortType::TopYear
        | SortType::TopMonth
        | SortType::TopWeek
        | SortType::TopDay
        | SortType::TopHour
        | SortType::TopSixHour
        | SortType::TopTwelveHour
        | SortType::TopThreeMonths
        | SortType::TopSixMonths
        | SortType::TopNineMonths) => {
          let interval = match o {
            SortType::TopYear => 1.years(),
            SortType::TopMonth => 1.months(),
            SortType::TopWeek => 1.weeks(),
            SortType::TopDay => 1.days(),
            SortType::TopHour => 1.hours(),
            SortType::TopSixHour => 6.hours(),
            SortType::TopTwelveHour => 12.hours(),
            SortType::TopThreeMonths => 3.months(),
            SortType::TopSixMonths => 6.months(),
            SortType::TopNineMonths => 9.months(),
            _ => return Err(Error::NotFound),
          };
          query = query.filter(post_aggregates::published.gt(now - interval));
          query = order_and_page_filter_desc(query, score, &options, |e| e.score);
          query = order_and_page_filter_desc(query, published, &options, |e| e.published);
        }
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

    query.load::<PostView>(&mut conn).await
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
    let res = queries()
      .read(pool, (post_id, my_person_id, is_mod_or_admin))
      .await?;

    Ok(res)
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
  // if true, the query should be handled as if community_id was not given except adding the literal filter
  pub community_id_just_for_prefetch: bool,
  pub local_user: Option<&'a LocalUserView>,
  pub search_term: Option<String>,
  pub url_search: Option<String>,
  pub saved_only: bool,
  pub liked_only: bool,
  pub disliked_only: bool,
  pub moderator_view: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub page_after: Option<PaginationCursorData>,
  pub page_before_or_equal: Option<PaginationCursorData>,
}

impl<'a> PostQuery<'a> {
  async fn prefetch_upper_bound_for_page_before(
    &self,
    pool: &mut DbPool<'_>,
  ) -> Result<Option<PostQuery<'a>>, Error> {
    // first get one page for the most popular community to get an upper bound for the the page end for the real query
    // the reason this is needed is that when fetching posts for a single community PostgreSQL can optimize
    // the query to use an index on e.g. (=, >=, >=, >=) and fetch only LIMIT rows
    // but for the followed-communities query it has to query the index on (IN, >=, >=, >=)
    // which it currently can't do at all (as of PG 16). see the discussion here:
    // https://github.com/LemmyNet/lemmy/issues/2877#issuecomment-1673597190
    //
    // the results are correct no matter which community we fetch these for, since it basically covers the "worst case" of the whole page consisting of posts from one community
    // but using the largest community decreases the pagination-frame so make the real query more efficient.
    use lemmy_db_schema::schema::{
      community_aggregates::dsl::{community_aggregates, community_id, users_active_month},
      community_follower::dsl::{
        community_follower,
        community_id as follower_community_id,
        person_id,
      },
    };
    let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    if offset != 0 && self.page_after.is_some() {
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
      return Ok(None);
    };

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
    // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT results (return original query)
    if (v.len() as i64) < limit {
      Ok(Some(self.clone()))
    } else {
      let page_before_or_equal = Some(PaginationCursorData(v.pop().expect("else case").counts));
      Ok(Some(PostQuery {
        page_before_or_equal,
        ..self.clone()
      }))
    }
  }

  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PostView>, Error> {
    if self.listing_type == Some(ListingType::Subscribed)
      && self.community_id.is_none()
      && self.local_user.is_some()
      && self.page_before_or_equal.is_none()
    {
      if let Some(query) = self.prefetch_upper_bound_for_page_before(pool).await? {
        queries().list(pool, query).await
      } else {
        Ok(vec![])
      }
    } else {
      queries().list(pool, self).await
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
    inserted_bot_post: Post,
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

    let blocked_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_blocked_person.id)
      .password_encrypted(String::new())
      .build();
    let inserted_blocked_local_user = LocalUser::create(pool, &blocked_local_user_form).await.unwrap();

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

    let inserted_bot_post = Post::create(pool, &new_bot_post).await.unwrap();
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
      inserted_bot_post,
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
      data.inserted_bot_post.id,
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

    // Removed bot post is shown to admins on its profile page
    data.local_user_view.local_user.admin = true;
    let post_listings_is_admin = PostQuery {
      sort: Some(SortType::New),
      creator_id: Some(data.inserted_bot.id),
      local_user: Some(&data.local_user_view),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(
      data.inserted_bot.id,
      post_listings_is_admin[0].post.creator_id
    );

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

    // Deleted post is only shown to creator
    for (local_user, expect_contains_deleted) in [
       (None, false),
       (Some(&data.blocked_local_user_view), false),
       (Some(&data.local_user_view), true),
    ] {
      let contains_deleted = PostQuery {
        sort: Some(SortType::New),
        local_user,
        ..data.default_post_query()
      }
      .list(pool)
      .await
      .unwrap()
      .iter()
      .any(|p| p.post.id == data.inserted_post.id);

      assert_eq!(expect_contains_deleted, contains_deleted);
    }

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
