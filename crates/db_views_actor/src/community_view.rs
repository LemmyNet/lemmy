use crate::structs::{CommunityModeratorView, CommunityView, PersonViewSafe};
use diesel::{result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::CommunityAggregates,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_aggregates, community_block, community_follower, local_user},
  source::{
    community::{Community, CommunityFollower, CommunitySafe},
    community_block::CommunityBlock,
  },
  traits::{MaybeOptional, ToSafe, ViewToVec},
  utils::{functions::hot_rank, fuzzy_search, limit_and_offset},
  ListingType, SortType,
};

type CommunityViewTuple = (
  CommunitySafe,
  CommunityAggregates,
  Option<CommunityFollower>,
  Option<CommunityBlock>,
);

impl CommunityView {
  pub fn read(
    conn: &mut PgConnection,
    community_id: CommunityId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (community, counts, follower, blocked) = community::table
      .find(community_id)
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_block::table.on(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        community_block::all_columns.nullable(),
      ))
      .first::<CommunityViewTuple>(conn)?;

    Ok(CommunityView {
      community,
      subscribed: follower.is_some(),
      blocked: blocked.is_some(),
      counts,
    })
  }

  pub fn is_mod_or_admin(
    conn: &mut PgConnection,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> bool {
    let is_mod = CommunityModeratorView::for_community(conn, community_id)
      .map(|v| {
        v.into_iter()
          .map(|m| m.moderator.id)
          .collect::<Vec<PersonId>>()
      })
      .unwrap_or_default()
      .contains(&person_id);
    if is_mod {
      return true;
    }

    PersonViewSafe::admins(conn)
      .map(|v| {
        v.into_iter()
          .map(|a| a.person.id)
          .collect::<Vec<PersonId>>()
      })
      .unwrap_or_default()
      .contains(&person_id)
  }
}

pub struct CommunityQueryBuilder<'a> {
  conn: &'a PgConnection,
  listing_type: Option<ListingType>,
  sort: Option<SortType>,
  my_person_id: Option<PersonId>,
  show_nsfw: Option<bool>,
  search_term: Option<String>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> CommunityQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    CommunityQueryBuilder {
      conn,
      my_person_id: None,
      listing_type: None,
      sort: None,
      show_nsfw: None,
      search_term: None,
      page: None,
      limit: None,
    }
  }

  pub fn listing_type<T: MaybeOptional<ListingType>>(mut self, listing_type: T) -> Self {
    self.listing_type = listing_type.get_optional();
    self
  }

  pub fn sort<T: MaybeOptional<SortType>>(mut self, sort: T) -> Self {
    self.sort = sort.get_optional();
    self
  }

  pub fn show_nsfw<T: MaybeOptional<bool>>(mut self, show_nsfw: T) -> Self {
    self.show_nsfw = show_nsfw.get_optional();
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
      .left_join(local_user::table.on(local_user::person_id.eq(person_id_join)))
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_block::table.on(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .select((
        Community::safe_columns_tuple(),
        community_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        community_block::all_columns.nullable(),
      ))
      .into_boxed();

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(community::name.ilike(searcher.to_owned()))
        .or_filter(community::title.ilike(searcher.to_owned()))
        .or_filter(community::description.ilike(searcher));
    };

    match self.sort.unwrap_or(SortType::Hot) {
      SortType::New => query = query.order_by(community::published.desc()),
      SortType::TopAll => query = query.order_by(community_aggregates::subscribers.desc()),
      SortType::TopMonth => query = query.order_by(community_aggregates::users_active_month.desc()),
      SortType::Hot => {
        query = query
          .order_by(
            hot_rank(
              community_aggregates::subscribers,
              community_aggregates::published,
            )
            .desc(),
          )
          .then_order_by(community_aggregates::published.desc());
        // Don't show hidden communities in Hot (trending)
        query = query.filter(
          community::hidden
            .eq(false)
            .or(community_follower::person_id.eq(person_id_join)),
        );
      }
      // Covers all other sorts
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

    if let Some(listing_type) = self.listing_type {
      query = match listing_type {
        ListingType::Subscribed => query.filter(community_follower::person_id.is_not_null()), // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
        ListingType::Local => query.filter(community::local.eq(true)),
        _ => query,
      };
    }

    // Don't show blocked communities or nsfw communities if not enabled in profile
    if self.my_person_id.is_some() {
      query = query.filter(community_block::person_id.is_null());
      query = query.filter(community::nsfw.eq(false).or(local_user::show_nsfw.eq(true)));
    } else {
      // No person in request, only show nsfw communities if show_nsfw passed into request
      if !self.show_nsfw.unwrap_or(false) {
        query = query.filter(community::nsfw.eq(false));
      }
    }

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
        blocked: a.3.is_some(),
      })
      .collect::<Vec<Self>>()
  }
}
