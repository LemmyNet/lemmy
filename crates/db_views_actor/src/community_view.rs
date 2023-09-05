use crate::structs::{CommunityModeratorView, CommunityView, PersonView};
use diesel::{
    pg::Pg, result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, PgTextExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
    newtypes::{CommunityId, PersonId},
    schema::{community, community_aggregates, community_block, community_follower, local_user},
    source::{community::CommunityFollower, local_user::LocalUser},
    utils::{fuzzy_search, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
    ListingType, SortType,
};

fn queries<'a>() -> Queries<
    impl ReadFn<'a, CommunityView, (CommunityId, Option<PersonId>, bool)>,
    impl ListFn<'a, CommunityView, CommunityQuery<'a>>,
> {
    let all_joins = |query: community::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
        // The left join below will return None in this case
        let person_id_join = my_person_id.unwrap_or(PersonId(-1));

        query
            .inner_join(community_aggregates::table)
            .left_join(
                community_follower::table.on(community::id
                    .eq(community_follower::community_id)
                    .and(community_follower::person_id.eq(person_id_join))),
            )
            .left_join(
                community_block::table.on(community::id
                    .eq(community_block::community_id)
                    .and(community_block::person_id.eq(person_id_join))),
            )
    };

    let selection = (
        community::all_columns,
        CommunityFollower::select_subscribed_type(),
        community_block::id.nullable().is_not_null(),
        community_aggregates::all_columns,
    );

    let not_removed_or_deleted = community::removed
        .eq(false)
        .and(community::deleted.eq(false));

    let read = move |mut conn: DbConn<'a>,
                     (community_id, my_person_id, is_mod_or_admin): (
        CommunityId,
        Option<PersonId>,
        bool,
    )| async move {
        let mut query = all_joins(
            community::table.find(community_id).into_boxed(),
            my_person_id,
        )
        .select(selection);

        // Hide deleted and removed for non-admins or mods
        if !is_mod_or_admin {
            query = query.filter(not_removed_or_deleted);
        }

        query.first::<CommunityView>(&mut conn).await
    };

    let list = move |mut conn: DbConn<'a>, options: CommunityQuery<'a>| async move {
        use SortType::*;

        let my_person_id = options.local_user.map(|l| l.person_id);

        // The left join below will return None in this case
        let person_id_join = my_person_id.unwrap_or(PersonId(-1));

        let mut query = all_joins(community::table.into_boxed(), my_person_id)
            .left_join(local_user::table.on(local_user::person_id.eq(person_id_join)))
            .select(selection);

        if let Some(search_term) = options.search_term {
            let searcher = fuzzy_search(&search_term);
            query = query
                .filter(community::name.ilike(searcher.clone()))
                .or_filter(community::title.ilike(searcher))
        }

        // Hide deleted and removed for non-admins or mods
        if !options.is_mod_or_admin {
            query = query.filter(not_removed_or_deleted).filter(
                community::hidden
                    .eq(false)
                    .or(community_follower::person_id.eq(person_id_join)),
            );
        }

        match options.sort.unwrap_or(Hot) {
            Hot | Active => query = query.order_by(community_aggregates::hot_rank.desc()),
            NewComments | TopDay | TopTwelveHour | TopSixHour | TopHour => {
                query = query.order_by(community_aggregates::users_active_day.desc())
            }
            New => query = query.order_by(community::published.desc()),
            Old => query = query.order_by(community::published.asc()),
            // Controversial is temporary until a CommentSortType is created
            MostComments | Controversial => {
                query = query.order_by(community_aggregates::comments.desc())
            }
            TopAll | TopYear | TopNineMonths => {
                query = query.order_by(community_aggregates::subscribers.desc())
            }
            TopSixMonths | TopThreeMonths => {
                query = query.order_by(community_aggregates::users_active_half_year.desc())
            }
            TopMonth => query = query.order_by(community_aggregates::users_active_month.desc()),
            TopWeek => query = query.order_by(community_aggregates::users_active_week.desc()),
        };

        if let Some(listing_type) = options.listing_type {
            query = match listing_type {
                ListingType::Subscribed => query.filter(community_follower::pending.is_not_null()), // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
                ListingType::Local => query.filter(community::local.eq(true)),
                _ => query,
            };
        }

        // Don't show blocked communities or nsfw communities if not enabled in profile
        if options.local_user.is_some() {
            query = query.filter(community_block::person_id.is_null());
            query = query.filter(community::nsfw.eq(false).or(local_user::show_nsfw.eq(true)));
        } else {
            // No person in request, only show nsfw communities if show_nsfw is passed into request
            if !options.show_nsfw {
                query = query.filter(community::nsfw.eq(false));
            }
        }

        let (limit, offset) = limit_and_offset(options.page, options.limit)?;
        query
            .limit(limit)
            .offset(offset)
            .load::<CommunityView>(&mut conn)
            .await
    };

    Queries::new(read, list)
}

impl CommunityView {
    pub async fn read(
        pool: &mut DbPool<'_>,
        community_id: CommunityId,
        my_person_id: Option<PersonId>,
        is_mod_or_admin: bool,
    ) -> Result<Self, Error> {
        queries()
            .read(pool, (community_id, my_person_id, is_mod_or_admin))
            .await
    }

    pub async fn is_mod_or_admin(
        pool: &mut DbPool<'_>,
        person_id: PersonId,
        community_id: CommunityId,
    ) -> Result<bool, Error> {
        let is_mod =
            CommunityModeratorView::is_community_moderator(pool, community_id, person_id).await?;
        if is_mod {
            return Ok(true);
        }

        PersonView::is_admin(pool, person_id).await
    }
}

#[derive(Default)]
pub struct CommunityQuery<'a> {
    pub listing_type: Option<ListingType>,
    pub sort: Option<SortType>,
    pub local_user: Option<&'a LocalUser>,
    pub search_term: Option<String>,
    pub is_mod_or_admin: bool,
    pub show_nsfw: bool,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl<'a> CommunityQuery<'a> {
    pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<CommunityView>, Error> {
        queries().list(pool, self).await
    }
}
