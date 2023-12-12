use crate::structs::{LocalUserView, PaginationCursor, PostView};
use diesel::{
  debug_query,
  dsl::{exists, not, IntervalDsl},
  pg::Pg,
  result::Error,
  sql_types,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  OptionalExtension,
  PgTextExpressionMethods,
  QueryDsl,
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
    and_then,
    functions::coalesce,
    fuzzy_search,
    get_conn,
    is_some_and,
    limit_and_offset,
    now,
    BoxExpr,
    DbPool,
    FilterVarEq,
    FirstOrLoad,
  },
  ListingType,
  SortType,
};
use tracing::debug;

#[derive(Clone, Copy)]
enum Ord {
  Desc,
  Asc,
}

struct PaginationCursorField<Q, QS> {
  then_order_by_desc: fn(Q) -> Q,
  then_order_by_asc: fn(Q) -> Q,
  le: fn(&PostAggregates) -> BoxExpr<QS, sql_types::Bool>,
  ge: fn(&PostAggregates) -> BoxExpr<QS, sql_types::Bool>,
  ne: fn(&PostAggregates) -> BoxExpr<QS, sql_types::Bool>,
}

/// Returns `PaginationCursorField<_, _>` for the given name
macro_rules! field {
  ($name:ident) => {
    // Type inference doesn't work if normal method call syntax is used
    PaginationCursorField {
      then_order_by_desc: |query| QueryDsl::then_order_by(query, post_aggregates::$name.desc()),
      then_order_by_asc: |query| QueryDsl::then_order_by(query, post_aggregates::$name.asc()),
      le: |e| Box::new(post_aggregates::$name.le(e.$name)),
      ge: |e| Box::new(post_aggregates::$name.ge(e.$name)),
      ne: |e| Box::new(post_aggregates::$name.ne(e.$name)),
    }
  };
}

#[allow(clippy::large_enum_variant)]
enum QueryInput<'a> {
  Read {
    post_id: PostId,
    me: Option<PersonId>,
    is_mod_or_admin: bool,
  },
  List {
    options: PostQuery<'a>,
  },
}

async fn build_query<'a>(
  pool: &mut DbPool<'_>,
  input: &'a QueryInput<'_>,
) -> Result<impl FirstOrLoad<'a, PostView>, Error> {
  let me = match input {
    QueryInput::Read { me, .. } => *me,
    QueryInput::List { options } => options.local_user.map(|l| l.person.id),
  };

  // This builds parts of the selection that can be changed by `filter_var_eq`. Initializing them
  // multiple times needs to be possible because of the top community prefetch for the subscribed view.
  let init_selected_values = move || {
    (
      is_some_and(me, |me| {
        exists(post_saved::table.find((me, post_aggregates::post_id)))
      }),
      is_some_and(me, |me| {
        exists(post_read::table.find((me, post_aggregates::post_id)))
      }),
      is_some_and(me, |me| {
        exists(person_block::table.find((me, post_aggregates::creator_id)))
      }),
      and_then(me, |me| {
        post_like::table
          .find((me, post_aggregates::post_id))
          .select(post_like::score.nullable())
          .single_value()
      }),
    )
  };
  let subscribe = move || {
    and_then(me, |me| {
      community_follower::table
        .find((me, post_aggregates::community_id))
        .select(community_follower::pending.nullable())
        .single_value()
    })
  };
  let read_comments = and_then(me, |me| {
    person_post_aggregates::table
      .find((me, post_aggregates::post_id))
      .select(person_post_aggregates::read_comments.nullable())
      .single_value()
  });
  let creator_banned_from_community = exists(
    community_person_ban::table.find((post_aggregates::creator_id, post_aggregates::community_id)),
  );
  let creator_is_moderator = exists(
    community_moderator::table.find((post_aggregates::creator_id, post_aggregates::community_id)),
  );
  let creator_is_admin = exists(
    local_user::table
      .filter(local_user::person_id.eq(post_aggregates::creator_id))
      .filter(local_user::admin),
  );
  let removed = community::removed.or(post::removed);
  let deleted = community::deleted.or(post::deleted);
  let is_creator = post_aggregates::creator_id.nullable().eq(me);

  let new_query = || {
    post_aggregates::table
      .inner_join(person::table)
      .inner_join(community::table)
      .inner_join(post::table)
      .into_boxed()
  };

  let (final_query, (saved, read, creator_blocked, my_vote)) = match input {
    QueryInput::Read {
      post_id,
      me: _,
      is_mod_or_admin,
    } => {
      let mut query = new_query().filter(post_aggregates::post_id.eq(post_id));

      // only show removed or deleted posts to creator, mods, and admins
      if !is_mod_or_admin {
        query = query.filter(is_creator.or(not(removed.or(deleted))));
      }

      (query, init_selected_values())
    }
    QueryInput::List { options } => {
      let listing_type = options.listing_type.unwrap_or(ListingType::All);
      let sort = options.sort.unwrap_or(SortType::Hot);
      let local_user = options.local_user.map(|l| &l.local_user);

      let admin = local_user.map(|l| l.admin).unwrap_or(false);
      let show_nsfw = local_user.map(|l| l.show_nsfw).unwrap_or(false);
      let show_bot_accounts = local_user.map(|l| l.show_bot_accounts).unwrap_or(true);
      let show_read_posts = local_user.map(|l| l.show_read_posts).unwrap_or(true);

      let (limit, page_number_offset) = limit_and_offset(options.page, options.limit)?;
      let previous_page_exclusion_offset = if options.page_after.is_some() {
        // always skip exactly one post because that's the last post of the previous page
        // fixing the where clause is more difficult because we'd have to change only the last order-by-where clause
        // e.g. WHERE (featured_local<=, hot_rank<=, published<=) to WHERE (<=, <=, <)
        1
      } else {
        0
      };
      let offset = page_number_offset + previous_page_exclusion_offset;

      let build_inner_query = |page_before_or_equal: Option<PaginationCursorData>| {
        let mut query = new_query().limit(limit).offset(offset);
        let (mut saved, mut read, mut creator_blocked, mut my_vote) = init_selected_values();

        let is_subscriber = || subscribe().is_not_null();

        query = query
          // hide posts from deleted communities
          .filter(not(community::deleted))
          // only show deleted posts to creator
          .filter(is_creator.or(not(post::deleted)));

        // only show removed posts to admin when viewing user profile
        if !(options.creator_id.is_some() && admin) {
          query = query.filter(not(removed));
        }

        if let Some(community_id) = options.community_id {
          query = query.filter(post_aggregates::community_id.eq(community_id));
        }
        if let Some(creator_id) = options.creator_id {
          query = query.filter(post_aggregates::creator_id.eq(creator_id));
        }
        if let Some(url_search) = &options.url_search {
          query = query.filter(post::url.eq(url_search));
        }
        if let Some(search_term) = &options.search_term {
          let pattern = fuzzy_search(search_term);
          let name_matches = post::name.ilike(pattern.clone());
          let body_matches = post::body.ilike(pattern);
          query = query.filter(name_matches.or(body_matches));
        }

        query = match listing_type {
          ListingType::Subscribed => query.filter(is_subscriber()),
          ListingType::Local => query.filter(community::local),
          ListingType::All => query,
          ListingType::ModeratorView => query.filter(is_some_and(me, |me| {
            exists(community_moderator::table.find((me, post_aggregates::community_id)))
          })),
        };

        // Filters that should not affect which posts can be moderated
        if listing_type != ListingType::ModeratorView {
          // If a user is logged in, then only show posts with a language that the user enabled.
          if let Some(local_user) = local_user {
            query = query.filter(exists(
              local_user_language::table.find((local_user.id, post::language_id)),
            ));
          }

          // Hide posts from blocked instances, communities, and persons
          query = query
            .filter_var_eq(&mut creator_blocked, false)
            .filter(not(is_some_and(me, |me| {
              let community_blocked =
                exists(community_block::table.find((me, post_aggregates::community_id)));
              let instance_blocked =
                exists(instance_block::table.find((me, post_aggregates::instance_id)));
              community_blocked.or(instance_blocked)
            })));

          // This filter hides hidden communities for non-subscribers. For `ListingType::Subscribed`,
          // it is redundant and would cause a duplicated `community_follower` subquery.
          if listing_type != ListingType::Subscribed {
            query = query.filter(is_subscriber().or(not(community::hidden)));
          }
        }

        if !show_nsfw {
          query = query.filter(not(post::nsfw.or(community::nsfw)));
        }
        if !show_bot_accounts {
          query = query.filter(not(person::bot_account));
        }
        if !(show_read_posts || options.saved_only || options.creator_id.is_some()) {
          query = query.filter_var_eq(&mut read, false);
        }
        if options.saved_only {
          query = query.filter_var_eq(&mut saved, true);
        }
        if options.liked_only {
          query = query.filter_var_eq(&mut my_vote, 1);
        }
        if options.disliked_only {
          query = query.filter_var_eq(&mut my_vote, -1);
        }

        let featured_field = if options.community_id.is_some() {
          field!(featured_community)
        } else {
          field!(featured_local)
        };

        let (main_sort, top_sort_interval) = match sort {
          SortType::Active => ((Ord::Desc, field!(hot_rank_active)), None),
          SortType::Hot => ((Ord::Desc, field!(hot_rank)), None),
          SortType::Scaled => ((Ord::Desc, field!(scaled_rank)), None),
          SortType::Controversial => ((Ord::Desc, field!(controversy_rank)), None),
          SortType::New => ((Ord::Desc, field!(published)), None),
          SortType::Old => ((Ord::Asc, field!(published)), None),
          SortType::NewComments => ((Ord::Desc, field!(newest_comment_time)), None),
          SortType::MostComments => ((Ord::Desc, field!(comments)), None),
          SortType::TopAll => ((Ord::Desc, field!(score)), None),
          SortType::TopYear => ((Ord::Desc, field!(score)), Some(1.years())),
          SortType::TopMonth => ((Ord::Desc, field!(score)), Some(1.months())),
          SortType::TopWeek => ((Ord::Desc, field!(score)), Some(1.weeks())),
          SortType::TopDay => ((Ord::Desc, field!(score)), Some(1.days())),
          SortType::TopHour => ((Ord::Desc, field!(score)), Some(1.hours())),
          SortType::TopSixHour => ((Ord::Desc, field!(score)), Some(6.hours())),
          SortType::TopTwelveHour => ((Ord::Desc, field!(score)), Some(12.hours())),
          SortType::TopThreeMonths => ((Ord::Desc, field!(score)), Some(3.months())),
          SortType::TopSixMonths => ((Ord::Desc, field!(score)), Some(6.months())),
          SortType::TopNineMonths => ((Ord::Desc, field!(score)), Some(9.months())),
        };

        if let Some(interval) = top_sort_interval {
          query = query.filter(post_aggregates::published.gt(now() - interval));
        }

        let tie_breaker = match sort {
          // A second time-based sort would not be very useful
          SortType::New | SortType::Old | SortType::NewComments => None,
          _ => Some((Ord::Desc, field!(published))),
        };

        let sorts = [
          Some((Ord::Desc, featured_field)),
          Some(main_sort),
          tie_breaker,
        ];
        let sorts_iter = sorts.iter().flatten();

        // This loop does almost the same thing as sorting by and comparing tuples. If the rows were
        // only sorted by 1 field called `foo` in descending order, then it would be like this:
        //
        // ```
        // query = query.then_order_by(foo.desc());
        // if let Some(first) = &options.page_after {
        //   query = query.filter(foo.le(first.foo));
        // }
        // if let Some(last) = &page_before_or_equal {
        //   query = query.filter(foo.ge(last.foo));
        // }
        // ```
        //
        // If multiple rows have the same value for a sorted field, then they are
        // grouped together, and the rows in that group are sorted by the next fields.
        // When checking if a row is within the range determined by the cursors, a field
        // that's sorted after other fields is only compared if the row and the cursor
        // are in the same group created by the previous sort, which is checked by using
        // `or` to skip the comparison if any previously sorted field is not equal.
        for (i, (order, field)) in sorts_iter.clone().enumerate() {
          // Both cursors are treated as inclusive here. `page_after` is made exclusive
          // by adding `1` to the offset.
          let (then_order_by_field, compare_first, compare_last) = match order {
            Ord::Desc => (field.then_order_by_desc, field.le, field.ge),
            Ord::Asc => (field.then_order_by_asc, field.ge, field.le),
          };

          query = then_order_by_field(query);

          for (cursor_data, compare) in [
            (&options.page_after, compare_first),
            (&page_before_or_equal, compare_last),
          ] {
            let Some(cursor_data) = cursor_data else {
              continue;
            };
            let mut condition: BoxExpr<_, sql_types::Bool> = Box::new(compare(&cursor_data.0));

            // For each field that was sorted before the current one, skip the filter by changing
            // `condition` to `true` if the row's value doesn't equal the cursor's value.
            for (_, other_field) in sorts_iter.clone().take(i) {
              condition = Box::new(condition.or((other_field.ne)(&cursor_data.0)));
            }

            query = query.filter(condition);
          }
        }

        debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));

        (query, (saved, read, creator_blocked, my_vote))
      };

      let page_before_or_equal = if listing_type == ListingType::Subscribed {
        // first get one page for the most popular community to get an upper bound for the the page end for the real query
        //
        // the reason this is needed is that when fetching posts for a single community PostgreSQL can optimize
        // the query to use an index on e.g. (=, >=, >=, >=) and fetch only LIMIT rows
        // but for the followed-communities query it has to query the index on (IN, >=, >=, >=)
        // which it currently can't do at all (as of PG 16). see the discussion here:
        // https://github.com/LemmyNet/lemmy/issues/2877#issuecomment-1673597190
        //
        // the results are correct no matter which community we fetch these for, since it basically covers the "worst case" of the whole page consisting of posts from one community
        // but using the largest community decreases the pagination-frame so make the real query more efficient.

        let largest_subscribed: Option<CommunityId> = community_aggregates::table
          .filter(is_some_and(me, |me| {
            exists(community_follower::table.find((me, community_aggregates::community_id)))
          }))
          .order_by(community_aggregates::users_active_month.desc())
          .select(community_aggregates::community_id)
          .first(&mut *get_conn(pool).await?)
          .await
          .optional()?;

        build_inner_query(None)
          .0
          .filter(
            post_aggregates::community_id
              .nullable()
              .eq(largest_subscribed),
          )
          // If there's at least `limit` rows, then get the last row within the limit. Otherwise,
          // get `None`, which prevents the amount of rows returned by the final query from being
          // incorrectly limited.
          .offset(offset + limit - 1)
          .select(post_aggregates::all_columns)
          .first(&mut *get_conn(pool).await?)
          .await
          .optional()?
          .map(PaginationCursorData)
      } else {
        None
      };

      build_inner_query(page_before_or_equal)
    }
  };

  Ok(final_query.select((
    post::all_columns,
    person::all_columns,
    community::all_columns,
    creator_banned_from_community,
    creator_is_moderator,
    creator_is_admin,
    post_aggregates::all_columns,
    subscribe(),
    saved,
    read,
    creator_blocked,
    my_vote,
    post_aggregates::comments - coalesce(read_comments, 0),
  )))
}

impl PostView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    me: Option<PersonId>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    build_query(
      pool,
      &QueryInput::Read {
        post_id,
        me,
        is_mod_or_admin,
      },
    )
    .await?
    .first(&mut *get_conn(pool).await?)
    .await
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
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub page_after: Option<PaginationCursorData>,
}

impl<'a> PostQuery<'a> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PostView>, Error> {
    build_query(pool, &QueryInput::List { options: self })
      .await?
      .load(&mut *get_conn(pool).await?)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    post_view::{PaginationCursorData, PostQuery, PostView},
    structs::LocalUserView,
  };
  use chrono::Utc;
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::{InstanceId, LanguageId, PersonId},
    source::{
      actor_language::LocalUserLanguage,
      comment::{Comment, CommentInsertForm},
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
    utils::{build_db_pool, DbPool, RANK_DEFAULT},
    SortType,
    SubscribedType,
  };
  use lemmy_utils::error::LemmyResult;
  use serial_test::serial;
  use std::time::Duration;

  const POST_BY_BLOCKED_PERSON: &str = "post by blocked person";
  const POST_BY_BOT: &str = "post by bot";
  const POST: &str = "post";

  fn names(post_views: &[PostView]) -> Vec<&str> {
    post_views.iter().map(|i| i.post.name.as_str()).collect()
  }

  struct Data {
    inserted_instance: Instance,
    local_user_view: LocalUserView,
    blocked_local_user_view: LocalUserView,
    inserted_bot: Person,
    inserted_community: Community,
    inserted_post: Post,
    inserted_bot_post: Post,
  }

  impl Data {
    fn default_post_query(&self) -> PostQuery<'_> {
      PostQuery {
        sort: Some(SortType::New),
        local_user: Some(&self.local_user_view),
        ..Default::default()
      }
    }
  }

  fn default_person_insert_form(instance_id: InstanceId, name: &str) -> PersonInsertForm {
    PersonInsertForm::builder()
      .name(name.to_owned())
      .public_key("pubkey".to_string())
      .instance_id(instance_id)
      .build()
  }

  fn default_local_user_form(person_id: PersonId) -> LocalUserInsertForm {
    LocalUserInsertForm::builder()
      .person_id(person_id)
      .password_encrypted(String::new())
      .build()
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = default_person_insert_form(inserted_instance.id, "tegan");

    let inserted_person = Person::create(pool, &new_person).await?;

    let local_user_form = LocalUserInsertForm {
      admin: Some(true),
      ..default_local_user_form(inserted_person.id)
    };
    let inserted_local_user = LocalUser::create(pool, &local_user_form).await?;

    let new_bot = PersonInsertForm {
      bot_account: Some(true),
      ..default_person_insert_form("mybot")
    },

    let inserted_bot = Person::create(pool, &new_bot).await?;

    let new_community = CommunityInsertForm::builder()
      .name("test_community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await?;

    // Test a person block, make sure the post query doesn't include their post
    let blocked_person = default_person_insert_form("john");

    let inserted_blocked_person = Person::create(pool, &blocked_person).await?;

    let inserted_blocked_local_user =
      LocalUser::create(pool, &local_user_form(inserted_blocked_person.id)).await?;

    let post_from_blocked_person = PostInsertForm::builder()
      .name(POST_BY_BLOCKED_PERSON.to_string())
      .creator_id(inserted_blocked_person.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(1)))
      .build();

    Post::create(pool, &post_from_blocked_person).await?;

    // block that person
    let person_block = PersonBlockForm {
      person_id: inserted_person.id,
      target_id: inserted_blocked_person.id,
    };

    PersonBlock::block(pool, &person_block).await?;

    // A sample post
    let new_post = PostInsertForm::builder()
      .name(POST.to_string())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(47)))
      .build();

    let inserted_post = Post::create(pool, &new_post).await?;

    let new_bot_post = PostInsertForm::builder()
      .name(POST_BY_BOT.to_string())
      .creator_id(inserted_bot.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_bot_post = Post::create(pool, &new_bot_post).await?;
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

    Ok(Data {
      inserted_instance,
      local_user_view,
      blocked_local_user_view,
      inserted_bot,
      inserted_community,
      inserted_post,
      inserted_bot_post,
    })
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_with_person() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let innserted_local_user = LocalUser::update(
      pool,
      data.local_user_view.local_user.id,
      &local_user_form,
    ).await?;
    data.local_user_view.local_user = inserted_local_user;

    let read_post_listing = PostQuery {
      community_id: Some(data.inserted_community.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?;

    let post_listing_single_with_person = PostView::read(
      pool,
      data.inserted_post.id,
      Some(data.local_user_view.person.id),
      false,
    )
    .await?;

    let expected_post_listing_with_user = expected_post_view(&data, pool).await?;

    // Should be only one person, IE the bot post, and blocked should be missing
    assert_eq!(vec![post_listing_single_with_person.clone()], read_post_listing);
    assert_eq!(expected_post_listing_with_user, post_listing_single_with_person);

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let innserted_local_user = LocalUser::update(
      pool,
      data.local_user_view.local_user.id,
      &local_user_form,
    ).await?;
    data.local_user_view.local_user = inserted_local_user;

    let post_listings_with_bots = PostQuery {
      community_id: Some(data.inserted_community.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    // should include bot post which has "undetermined" language
    assert_eq!(vec![POST_BY_BOT, POST], names(&post_listings_with_bots));

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_no_person() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let read_post_listing_multiple_no_person = PostQuery {
      community_id: Some(data.inserted_community.id),
      local_user: None,
      ..data.default_post_query()
    }
    .list(pool)
    .await?;

    let read_post_listing_single_no_person = PostView::read(pool, data.inserted_post.id, None, false).await?;

    let expected_post_listing_no_person = expected_post_view(&data, pool).await?;

    // Should be 2 posts, with the bot post, and the blocked
    assert_eq!(
      vec![POST_BY_BOT, POST, POST_BY_BLOCKED_PERSON],
      names(&read_post_listing_multiple_no_person)
    );

    assert_eq!(Some(expected_post_listing_no_person), read_post_listing_multiple_no_person.get(1));
    assert_eq!(expected_post_listing_no_person, read_post_listing_single_no_person);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_block_community() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let community_block = CommunityBlockForm {
      person_id: data.local_user_view.person.id,
      community_id: data.inserted_community.id,
    };
    CommunityBlock::block(pool, &community_block).await?;

    let read_post_listings_with_person_after_block = PostQuery {
      community_id: Some(data.inserted_community.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    // Should be 0 posts after the community block
    assert_eq!(read_post_listings_with_person_after_block, vec![]);

    CommunityBlock::unblock(pool, &community_block).await?;
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_like() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    let post_like_form = PostLikeForm {
      post_id: data.inserted_post.id,
      person_id: data.local_user_view.person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(pool, &post_like_form).await?;

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
    .await?;

    let mut expected_post_with_upvote = expected_post_view(&data, pool).await?;
    expected_post_with_upvote.my_vote = Some(1);
    expected_post_with_upvote.counts.score = 1;
    expected_post_with_upvote.counts.upvotes = 1;
    assert_eq!(expected_post_with_upvote, post_listing_single_with_person);

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let innserted_local_user = LocalUser::update(
      pool,
      data.local_user_view.local_user.id,
      &local_user_form,
    ).await?;
    data.local_user_view.local_user = inserted_local_user;

    let read_post_listing = PostQuery {
      community_id: Some(data.inserted_community.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    assert_eq!(vec![expected_post_with_upvote], read_post_listing);

    let read_liked_post_listing = PostQuery {
      community_id: Some(data.inserted_community.id),
      liked_only: true,
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    assert_eq!(post_list, read_liked_post_listing);

    let read_disliked_post_listing = PostQuery {
      community_id: Some(data.inserted_community.id),
      disliked_only: true,
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    assert_eq!(read_disliked_post_listing, vec![]);

    let like_removed =
      PostLike::remove(pool, data.local_user_view.person.id, data.inserted_post.id).await?;
    assert_eq!(1, like_removed);
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn creator_info() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Make one of the inserted persons a moderator
    let person_id = data.local_user_view.person.id;
    let community_id = data.inserted_community.id;
    let form = CommunityModeratorForm {
      community_id,
      person_id,
    };
    CommunityModerator::join(pool, &form).await.unwrap();

    let post_listing = PostQuery {
      community_id: Some(data.inserted_community.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?
    .into_iter()
    .map(|p| (p.creator.name, p.creator_is_moderator, p.creator_is_admin))
    .collect::<Vec<_>>();

    let expected_post_listing = vec![
      ("mybot".to_owned(), false, false),
      ("tegan".to_owned(), true, true),
    ];

    assert_eq!(expected_post_listing, post_listing);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_person_language() -> LemmyResult<()> {
    const EL_POSTO: &str = "el posto";

    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let spanish_id = Language::read_id_from_code(pool, Some("es"))
      .await?
      .expect("spanish should exist");

    let french_id = Language::read_id_from_code(pool, Some("fr"))
      .await?
      .expect("french should exist");

    let post_spanish = PostInsertForm::builder()
      .name(EL_POSTO.to_string())
      .creator_id(data.local_user_view.person.id)
      .community_id(data.inserted_community.id)
      .language_id(Some(spanish_id))
      .build();

    Post::create(pool, &post_spanish).await?;

    let post_listings_all = data.default_post_query().list(pool).await?;

    // no language filters specified, all posts should be returned
    assert_eq!(vec![EL_POSTO, POST_BY_BOT, POST], names(&post_listings_all));

    LocalUserLanguage::update(pool, vec![french_id], data.local_user_view.local_user.id).await?;

    let post_listing_french = data.default_post_query().list(pool).await?;

    // only one post in french and one undetermined should be returned
    assert_eq!(vec![POST_BY_BOT, POST], names(&post_list_french));
    assert_eq!(
      Some(french_id),
      post_listing_french.get(1).map(|p| p.post.language_id)
    );

    LocalUserLanguage::update(
      pool,
      vec![french_id, UNDETERMINED_ID],
      data.local_user_view.local_user.id,
    )
    .await?;
    let post_listings_french_und = data
      .default_post_query()
      .list(pool)
      .await?
      .into_iter()
      .map(|p| (p.post.name, p.post.language_id))
      .collect::<Vec<_>>();
    let expected_post_listings_french_und = vec![
      (POST_BY_BOT.to_owned(), UNDETERMINED_ID),
      (POST.to_owned(), french_id),
    ];

    // french post and undetermined language post should be returned
    assert_eq!(expected_post_listings_french_und, post_listings_french_und);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listings_removed() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    // Remove the post
    Post::update(
      pool,
      data.inserted_bot_post.id,
      &PostUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // Make sure you don't see the removed post in the results
    let post_listings_no_admin = data.default_post_query().list(pool).await?;
    assert_eq!(vec![POST_BY_BOT], names(&post_listings_no_admin));

    // Removed bot post is shown to admins on its profile page
    data.local_user_view.local_user.admin = true;
    let post_listings_is_admin = PostQuery {
      creator_id: Some(data.inserted_bot.id),
      ..data.default_post_query()
    }
    .list(pool)
    .await?;
    assert_eq!(vec![POST_BY_BOT], names(&post_listins_is_admin));

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listings_deleted() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Delete the post
    Post::update(
      pool,
      data.inserted_post.id,
      &PostUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    for (local_user, expect_contains_deleted) in [
      (None, false),
      (Some(&data.blocked_local_user_view), false),
      (Some(&data.local_user_view), true),
    ] {
      let contains_deleted = PostQuery {
        local_user,
        ..data.default_post_query()
      }
      .list(pool)
      .await?
      .iter()
      .any(|p| p.post.id == data.inserted_post.id);

      assert_eq!(expect_contains_deleted, contains_deleted);
    }

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn post_listing_instance_block() -> LemmyResult<()> {
    const POST_FROM_BLOCKED_INSTANCE: &str = "post on blocked instance";

    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let blocked_instance = Instance::read_or_create(pool, "another_domain.tld".to_string()).await?;

    let community_form = CommunityInsertForm::builder()
      .name("test_community_4".to_string())
      .title("none".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(blocked_instance.id)
      .build();
    let inserted_community = Community::create(pool, &community_form).await?;

    let post_form = PostInsertForm::builder()
      .name(POST_FROM_BLOCKED_INSTANCE.to_string())
      .creator_id(data.inserted_bot.id)
      .community_id(inserted_community.id)
      .language_id(Some(LanguageId(1)))
      .build();

    let post_from_blocked_instance = Post::create(pool, &post_form).await?;

    // no instance block, should return all posts
    let post_listings_all = data.default_post_query().list(pool).await?;
    assert_eq!(
      vec![POST_FROM_BLOCKED_INSTANCE, POST_BY_BOT, POST],
      names(&post_listings_all)
    );

    // block the instance
    let block_form = InstanceBlockForm {
      person_id: data.local_user_view.person.id,
      instance_id: blocked_instance.id,
    };
    InstanceBlock::block(pool, &block_form).await?;

    // now posts from communities on that instance should be hidden
    let post_listings_blocked = data.default_post_query().list(pool).await?;
    assert_eq!(vec![POST_BY_BOT, POST], names(&post_listings_blocked));
    assert!(post_listings_blocked
      .iter()
      .all(|p| p.post.id != post_from_blocked_instance.id));

    // after unblocking it should return all posts again
    InstanceBlock::unblock(pool, &block_form).await?;
    let post_listings_blocked = data.default_post_query().list(pool).await?;
    assert_eq!(
      vec![POST_FROM_BLOCKED_INSTANCE, POST_BY_BOT, POST],
      names(&post_listings_blocked)
    );

    Instance::delete(pool, blocked_instance.id).await?;
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn pagination_includes_each_post_once() -> LemmyResult<()> {
    let pool = &build_db_pool().await?;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let pre_existing_post_ids = data
      .default_post_query()
      .list(pool)
      .await?
      .into_iter()
      .map(|p| p.post.id)
      .collect::<Vec<_>>();

    let mut expected_post_ids = vec![];
    let mut comment_ids = vec![];

    // Create 15 posts for each amount of comments from 0 to 9
    for comments in 0..10 {
      for _ in 0..15 {
        let post = Post::create(
          pool,
          &PostInsertForm::builder()
            .name("keep Christ in Christmas".to_owned())
            .creator_id(data.local_user_view.person.id)
            .community_id(data.inserted_community.id)
            .featured_local(Some((comments % 2) == 0))
            .featured_community(Some((comments % 2) == 0))
            .published(Some(Utc::now() - Duration::from_secs(comments % 3)))
            .build(),
        )
        .await?;
        expected_post_ids.push(post.id);
        for _ in 0..comments {
          let comment = Comment::create(
            pool,
            &CommentInsertForm::builder()
              .creator_id(data.local_user_view.person.id)
              .post_id(post.id)
              .content("hi".to_owned())
              .build(),
            None,
          )
          .await?;
          comment_ids.push(comment.id);
        }
      }
    }

    let mut post_ids = vec![];
    let mut page_after = None;
    loop {
      let posts = PostQuery {
        sort: Some(SortType::MostComments),
        page_after,
        limit: Some(10),
        ..data.default_post_query()
      }
      .list(pool)
      .await?;

      post_ids.extend(
        posts
          .iter()
          .map(|p| p.post.id)
          .filter(|id| !pre_existing_post_ids.contains(&id))
      );

      if let Some(p) = posts.into_iter().last() {
        page_after = Some(PaginationCursorData(p.counts));
      } else {
        break;
      }
    }

    cleanup(data, pool).await?;

    expected_post_ids.sort_unstable_by_key(|id| id.0);
    post_ids.sort_unstable_by_key(|id| id.0);
    assert_eq!(expected_post_ids, post_ids);

    Ok(())
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    let num_deleted = Post::delete(pool, data.inserted_post.id).await?;
    Community::delete(pool, data.inserted_community.id).await?;
    Person::delete(pool, data.local_user_view.person.id).await?;
    Person::delete(pool, data.inserted_bot.id).await?;
    Person::delete(pool, data.blocked_local_user_view.person.id).await?;
    Instance::delete(pool, data.inserted_instance.id).await?;
    assert_eq!(1, num_deleted);

    Ok(())
  }

  async fn expected_post_view(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<PostView> {
    let (inserted_person, inserted_community, inserted_post) = (
      &data.local_user_view.person,
      &data.inserted_community,
      &data.inserted_post,
    );
    let agg = PostAggregates::read(pool, inserted_post.id).await?;

    Ok(PostView {
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
    })
  }
}
