use diesel::{pg::Pg, result::Error, *};
use lemmy_db_queries::{
  aggregates::post_aggregates::PostAggregates,
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
  schema::{
    community,
    community_follower,
    community_user_ban,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
    user_,
  },
  source::{
    community::{Community, CommunityFollower, CommunitySafe, CommunityUserBan},
    post::{Post, PostRead, PostSaved},
    user::{UserSafe, User_},
  },
};
use log::debug;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PostView {
  pub post: Post,
  pub creator: UserSafe,
  pub community: CommunitySafe,
  pub creator_banned_from_community: bool, // Left Join to CommunityUserBan
  pub counts: PostAggregates,
  pub subscribed: bool,     // Left join to CommunityFollower
  pub saved: bool,          // Left join to PostSaved
  pub read: bool,           // Left join to PostRead
  pub my_vote: Option<i16>, // Left join to PostLike
}

type PostViewTuple = (
  Post,
  UserSafe,
  CommunitySafe,
  Option<CommunityUserBan>,
  PostAggregates,
  Option<CommunityFollower>,
  Option<PostSaved>,
  Option<PostRead>,
  Option<i16>,
);

impl PostView {
  pub fn read(conn: &PgConnection, post_id: i32, my_user_id: Option<i32>) -> Result<Self, Error> {
    // The left join below will return None in this case
    let user_id_join = my_user_id.unwrap_or(-1);

    let (
      post,
      creator,
      community,
      creator_banned_from_community,
      counts,
      follower,
      saved,
      read,
      my_vote,
    ) = post::table
      .find(post_id)
      .inner_join(user_::table)
      .inner_join(community::table)
      .left_join(
        community_user_ban::table.on(
          post::community_id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(post::creator_id)),
        ),
      )
      .inner_join(post_aggregates::table)
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post::id
            .eq(post_saved::post_id)
            .and(post_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post::id
            .eq(post_read::post_id)
            .and(post_read::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        post::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        community_user_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        post_like::score.nullable(),
      ))
      .first::<PostViewTuple>(conn)?;

    Ok(PostView {
      post,
      creator,
      community,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      counts,
      subscribed: follower.is_some(),
      saved: saved.is_some(),
      read: read.is_some(),
      my_vote,
    })
  }
}

pub struct PostQueryBuilder<'a> {
  conn: &'a PgConnection,
  listing_type: &'a ListingType,
  sort: &'a SortType,
  creator_id: Option<i32>,
  community_id: Option<i32>,
  community_name: Option<String>,
  my_user_id: Option<i32>,
  search_term: Option<String>,
  url_search: Option<String>,
  show_nsfw: bool,
  saved_only: bool,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PostQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    PostQueryBuilder {
      conn,
      listing_type: &ListingType::All,
      sort: &SortType::Hot,
      creator_id: None,
      community_id: None,
      community_name: None,
      my_user_id: None,
      search_term: None,
      url_search: None,
      show_nsfw: true,
      saved_only: false,
      unread_only: false,
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

  pub fn community_id<T: MaybeOptional<i32>>(mut self, community_id: T) -> Self {
    self.community_id = community_id.get_optional();
    self
  }

  pub fn my_user_id<T: MaybeOptional<i32>>(mut self, my_user_id: T) -> Self {
    self.my_user_id = my_user_id.get_optional();
    self
  }

  pub fn community_name<T: MaybeOptional<String>>(mut self, community_name: T) -> Self {
    self.community_name = community_name.get_optional();
    self
  }

  pub fn creator_id<T: MaybeOptional<i32>>(mut self, creator_id: T) -> Self {
    self.creator_id = creator_id.get_optional();
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    self.search_term = search_term.get_optional();
    self
  }

  pub fn url_search<T: MaybeOptional<String>>(mut self, url_search: T) -> Self {
    self.url_search = url_search.get_optional();
    self
  }

  pub fn show_nsfw(mut self, show_nsfw: bool) -> Self {
    self.show_nsfw = show_nsfw;
    self
  }

  pub fn saved_only(mut self, saved_only: bool) -> Self {
    self.saved_only = saved_only;
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

  pub fn list(self) -> Result<Vec<PostView>, Error> {
    use diesel::dsl::*;

    // The left join below will return None in this case
    let user_id_join = self.my_user_id.unwrap_or(-1);

    let mut query = post::table
      .inner_join(user_::table)
      .inner_join(community::table)
      .left_join(
        community_user_ban::table.on(
          post::community_id
            .eq(community_user_ban::community_id)
            .and(community_user_ban::user_id.eq(community::creator_id)),
        ),
      )
      .inner_join(post_aggregates::table)
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_saved::table.on(
          post::id
            .eq(post_saved::post_id)
            .and(post_saved::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_read::table.on(
          post::id
            .eq(post_read::post_id)
            .and(post_read::user_id.eq(user_id_join)),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::user_id.eq(user_id_join)),
        ),
      )
      .select((
        post::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        community_user_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        post_like::score.nullable(),
      ))
      .into_boxed();

    query = match self.listing_type {
      ListingType::Subscribed => query.filter(community_follower::user_id.is_not_null()), // TODO could be this: and(community_follower::user_id.eq(user_id_join)),
      ListingType::Local => query.filter(community::local.eq(true)),
      _ => query,
    };

    if let Some(community_id) = self.community_id {
      query = query
        .filter(post::community_id.eq(community_id))
        .then_order_by(post_aggregates::stickied.desc());
    }

    if let Some(community_name) = self.community_name {
      query = query
        .filter(community::name.eq(community_name))
        .filter(community::local.eq(true))
        .then_order_by(post_aggregates::stickied.desc());
    }

    if let Some(url_search) = self.url_search {
      query = query.filter(post::url.eq(url_search));
    }

    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query.filter(
        post::name
          .ilike(searcher.to_owned())
          .or(post::body.ilike(searcher)),
      );
    }

    // If its for a specific user, show the removed / deleted
    if let Some(creator_id) = self.creator_id {
      query = query.filter(post::creator_id.eq(creator_id));
    }

    if !self.show_nsfw {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    // TODO  These two might be wrong
    if self.saved_only {
      query = query.filter(post_saved::id.is_not_null());
    };

    if self.unread_only {
      query = query.filter(post_read::id.is_not_null());
    };

    query = match self.sort {
      SortType::Active => query
        .then_order_by(
          hot_rank(post_aggregates::score, post_aggregates::newest_comment_time).desc(),
        )
        .then_order_by(post_aggregates::newest_comment_time.desc()),
      SortType::Hot => query
        .then_order_by(hot_rank(post_aggregates::score, post_aggregates::published).desc())
        .then_order_by(post_aggregates::published.desc()),
      SortType::New => query.then_order_by(post_aggregates::published.desc()),
      SortType::TopAll => query.then_order_by(post_aggregates::score.desc()),
      SortType::TopYear => query
        .filter(post::published.gt(now - 1.years()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopMonth => query
        .filter(post::published.gt(now - 1.months()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopWeek => query
        .filter(post::published.gt(now - 1.weeks()))
        .then_order_by(post_aggregates::score.desc()),
      SortType::TopDay => query
        .filter(post::published.gt(now - 1.days()))
        .then_order_by(post_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query = query
      .limit(limit)
      .offset(offset)
      .filter(post::removed.eq(false))
      .filter(post::deleted.eq(false))
      .filter(community::removed.eq(false))
      .filter(community::deleted.eq(false));

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));

    let res = query.load::<PostViewTuple>(self.conn)?;

    Ok(PostView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PostView {
  type DbTuple = PostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        post: a.0.to_owned(),
        creator: a.1.to_owned(),
        community: a.2.to_owned(),
        creator_banned_from_community: a.3.is_some(),
        counts: a.4.to_owned(),
        subscribed: a.5.is_some(),
        saved: a.6.is_some(),
        read: a.7.is_some(),
        my_vote: a.8,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::post_view::{PostQueryBuilder, PostView};
  use lemmy_db_queries::{
    aggregates::post_aggregates::PostAggregates,
    establish_unpooled_connection,
    Crud,
    Likeable,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{community::*, post::*, user::*};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let user_name = "tegan".to_string();
    let community_name = "test_community_3".to_string();
    let post_name = "test post 3".to_string();

    let new_user = UserForm {
      name: user_name.to_owned(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      published: None,
      updated: None,
      admin: false,
      banned: Some(false),
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: community_name.to_owned(),
      title: "nada".to_owned(),
      description: None,
      creator_id: inserted_user.id,
      category_id: 1,
      removed: None,
      deleted: None,
      updated: None,
      nsfw: false,
      actor_id: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
      icon: None,
      banner: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: post_name.to_owned(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      stickied: None,
      updated: None,
      nsfw: false,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(&conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_like.published,
      score: 1,
    };

    let read_post_listings_with_user = PostQueryBuilder::create(&conn)
      .listing_type(&ListingType::Community)
      .sort(&SortType::New)
      .community_id(inserted_community.id)
      .my_user_id(inserted_user.id)
      .list()
      .unwrap();

    let read_post_listings_no_user = PostQueryBuilder::create(&conn)
      .listing_type(&ListingType::Community)
      .sort(&SortType::New)
      .community_id(inserted_community.id)
      .list()
      .unwrap();

    let read_post_listing_no_user = PostView::read(&conn, inserted_post.id, None).unwrap();
    let read_post_listing_with_user =
      PostView::read(&conn, inserted_post.id, Some(inserted_user.id)).unwrap();

    let agg = PostAggregates::read(&conn, inserted_post.id).unwrap();

    // the non user version
    let expected_post_listing_no_user = PostView {
      post: Post {
        id: inserted_post.id,
        name: post_name,
        creator_id: inserted_user.id,
        url: None,
        body: None,
        published: inserted_post.published,
        updated: None,
        community_id: inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        stickied: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_html: None,
        thumbnail_url: None,
        ap_id: inserted_post.ap_id.to_owned(),
        local: true,
      },
      my_vote: None,
      creator: UserSafe {
        id: inserted_user.id,
        name: user_name,
        preferred_username: None,
        published: inserted_user.published,
        avatar: None,
        actor_id: inserted_user.actor_id.to_owned(),
        local: true,
        banned: false,
        deleted: false,
        bio: None,
        banner: None,
        admin: false,
        updated: None,
        matrix_user_id: None,
      },
      creator_banned_from_community: false,
      community: CommunitySafe {
        id: inserted_community.id,
        name: community_name,
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.to_owned(),
        local: true,
        title: "nada".to_owned(),
        description: None,
        creator_id: inserted_user.id,
        category_id: 1,
        updated: None,
        banner: None,
        published: inserted_community.published,
      },
      counts: PostAggregates {
        id: agg.id,
        post_id: inserted_post.id,
        comments: 0,
        score: 1,
        upvotes: 1,
        downvotes: 0,
        stickied: false,
        published: agg.published,
        newest_comment_time: inserted_post.published,
      },
      subscribed: false,
      read: false,
      saved: false,
    };

    // TODO More needs to be added here
    let mut expected_post_listing_with_user = expected_post_listing_no_user.to_owned();
    expected_post_listing_with_user.my_vote = Some(1);

    let like_removed = PostLike::remove(&conn, inserted_user.id, inserted_post.id).unwrap();
    let num_deleted = Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    // The with user
    assert_eq!(
      expected_post_listing_with_user,
      read_post_listings_with_user[0]
    );
    assert_eq!(expected_post_listing_with_user, read_post_listing_with_user);
    assert_eq!(1, read_post_listings_with_user.len());

    // Without the user
    assert_eq!(expected_post_listing_no_user, read_post_listings_no_user[0]);
    assert_eq!(expected_post_listing_no_user, read_post_listing_no_user);
    assert_eq!(1, read_post_listings_no_user.len());

    // assert_eq!(expected_post, inserted_post);
    // assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);
  }
}
