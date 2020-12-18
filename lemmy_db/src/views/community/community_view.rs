use crate::{
  aggregates::community_aggregates::CommunityAggregates,
  functions::hot_rank,
  fuzzy_search,
  limit_and_offset,
  source::{
    category::Category,
    community::{Community, CommunityFollower, CommunitySafe},
  },
  views::ViewToVec,
  MaybeOptional,
  SortType,
  ToSafe,
};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  schema::{category, community, community_aggregates, community_follower, user_},
  source::user::{UserSafe, User_},
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityView {
  pub community: CommunitySafe,
  pub creator: UserSafe,
  pub category: Category,
  pub subscribed: bool,
  pub counts: CommunityAggregates,
}

type CommunityViewTuple = (
  CommunitySafe,
  UserSafe,
  Category,
  CommunityAggregates,
  Option<CommunityFollower>,
);

impl CommunityView {
  pub fn read(
    conn: &PgConnection,
    community_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (community, creator, category, counts, follower) = community::table
      .find(community_id)
      .inner_join(user_::table)
      .inner_join(category::table)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        User_::safe_columns_tuple(),
        category::all_columns,
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
      ))
      .first::<CommunityViewTuple>(conn)?;

    Ok(CommunityView {
      community,
      creator,
      category,
      subscribed: follower.is_some(),
      counts,
    })
  }
}

pub struct CommunityQueryBuilder<'a> {
  conn: &'a PgConnection,
  sort: &'a SortType,
  my_user_id: Option<i32>,
  show_nsfw: bool,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommunityQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    CommunityQueryBuilder {
      conn,
      my_user_id: None,
      sort: &SortType::Hot,
      show_nsfw: true,
      search_term: None,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn show_nsfw(mut self, show_nsfw: bool) -> Self {
    self.show_nsfw = show_nsfw;
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
    self
  }

  pub fn my_user_id<T: MaybeOptional<i32>>(mut self, my_user_id: T) -> Self {
    self.my_user_id = my_user_id.get_optional();
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<CommunityView>, Error> {
    // The left join below will return None in this case
    let user_id_join = self.my_user_id.unwrap_or(-1);

    let mut query = community::table
      .inner_join(user_::table)
      .inner_join(category::table)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        User_::safe_columns_tuple(),
        category::all_columns,
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
      ))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(community::name.ilike(searcher.to_owned()))
        .or_filter(community::title.ilike(searcher.to_owned()))
        .or_filter(community::description.ilike(searcher));
    };

    match self.sort {
      SortType::New => query = query.order_by(community::published.desc()),
      SortType::TopAll => query = query.order_by(community_aggregates::subscribers.desc()),
      // Covers all other sorts, including hot
      _ => {
        query = query
          // TODO do custom sql function for hot_rank, make sure this works
          .order_by(hot_rank(community_aggregates::subscribers, community::published).desc())
          .then_order_by(community_aggregates::subscribers.desc())
      }
    };

    if !self.show_nsfw {
      query = query.filter(community::nsfw.eq(false));
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    let res = query
      .limit(limit)
      .offset(offset)
      .filter(community::removed.eq(false))
      .filter(community::deleted.eq(false))
      .load::<CommunityViewTuple>(self.conn)?;

    Ok(CommunityView::to_vec(res))
  }
}

impl ViewToVec for CommunityView {
  type DbTuple = CommunityViewTuple;
  fn to_vec(communities: Vec<Self::DbTuple>) -> Vec<Self> {
    communities
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        creator: a.1.to_owned(),
        category: a.2.to_owned(),
        counts: a.3.to_owned(),
        subscribed: a.4.is_some(),
      })
      .collect::<Vec<Self>>()
  }
}
