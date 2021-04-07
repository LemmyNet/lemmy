use crate::{community_moderator_view::CommunityModeratorView, person_view::PersonViewSafe};
use diesel::{result::Error, *};
use lemmy_db_queries::{
  aggregates::community_aggregates::CommunityAggregates,
  functions::hot_rank,
  fuzzy_search,
  limit_and_offset,
  ListingType,
  MaybeOptional,
  SortType,
  ToSafe,
  ViewToVec,
};
use lemmy_db_schema::{
  schema::{community, community_aggregates, community_follower},
  source::community::{Community, CommunityFollower, CommunitySafe},
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityView {
  pub community: CommunitySafe,
  pub subscribed: bool,
  pub counts: CommunityAggregates,
}

type CommunityViewTuple = (
  CommunitySafe,
  CommunityAggregates,
  Option<CommunityFollower>,
);

impl CommunityView {
  pub fn read(
    conn: &PgConnection,
    community_id: CommunityId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (community, counts, follower) = community::table
      .find(community_id)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
      ))
      .first::<CommunityViewTuple>(conn)?;

    Ok(CommunityView {
      community,
      subscribed: follower.is_some(),
      counts,
    })
  }

  // TODO: this function is only used by is_mod_or_admin() below, can probably be merged
  fn community_mods_and_admins(
    conn: &PgConnection,
    community_id: CommunityId,
  ) -> Result<Vec<PersonId>, Error> {
    let mut mods_and_admins: Vec<PersonId> = Vec::new();
    mods_and_admins.append(
      &mut CommunityModeratorView::for_community(conn, community_id)
        .map(|v| v.into_iter().map(|m| m.moderator.id).collect())?,
    );
    mods_and_admins.append(
      &mut PersonViewSafe::admins(conn).map(|v| v.into_iter().map(|a| a.person.id).collect())?,
    );
    Ok(mods_and_admins)
  }

  pub fn is_mod_or_admin(
    conn: &PgConnection,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> bool {
    Self::community_mods_and_admins(conn, community_id)
      .unwrap_or_default()
      .contains(&person_id)
  }
}

pub struct CommunityQueryBuilder<'a> {
  conn: &'a PgConnection,
  listing_type: &'a ListingType,
  sort: &'a SortType,
  my_person_id: Option<PersonId>,
  show_nsfw: bool,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommunityQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    CommunityQueryBuilder {
      conn,
      my_person_id: None,
      listing_type: &ListingType::All,
      sort: &SortType::Hot,
      show_nsfw: true,
      search_term: None,
      page: None,
      limit: None,
    }
  }

  pub fn listing_type(mut self, listing_type: &'a ListingType) -> Self {
    self.listing_type = listing_type;
    self
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

  pub fn my_person_id<T: MaybeOptional<PersonId>>(mut self, my_person_id: T) -> Self {
    self.my_person_id = my_person_id.get_optional();
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
    let person_id_join = self.my_person_id.unwrap_or(PersonId(-1));

    let mut query = community::table
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
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
      SortType::TopMonth => query = query.order_by(community_aggregates::users_active_month.desc()),
      // Covers all other sorts, including hot
      _ => {
        query = query
          .order_by(
            hot_rank(
              community_aggregates::subscribers,
              community_aggregates::published,
            )
            .desc(),
          )
          .then_order_by(community_aggregates::published.desc())
      }
    };

    if !self.show_nsfw {
      query = query.filter(community::nsfw.eq(false));
    };

    query = match self.listing_type {
      ListingType::Subscribed => query.filter(community_follower::person_id.is_not_null()), // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
      ListingType::Local => query.filter(community::local.eq(true)),
      _ => query,
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    let res = query
      .limit(limit)
      .offset(offset)
      .filter(community::removed.eq(false))
      .filter(community::deleted.eq(false))
      .load::<CommunityViewTuple>(self.conn)?;

    Ok(CommunityView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityView {
  type DbTuple = CommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        counts: a.1.to_owned(),
        subscribed: a.2.is_some(),
      })
      .collect::<Vec<Self>>()
  }
}
