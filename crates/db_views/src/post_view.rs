use crate::structs::PostView;
use diesel::{dsl::*, pg::Pg, result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::PostAggregates,
  newtypes::{CommunityId, DbUrl, LocalUserId, PersonId, PostId},
  schema::{
    community,
    community_block,
    community_follower,
    community_person_ban,
    local_user_language,
    person,
    person_block,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
  },
  source::{
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    local_user::LocalUser,
    person::{Person, PersonSafe},
    person_block::PersonBlock,
    post::{Post, PostRead, PostSaved},
  },
  traits::{ToSafe, ViewToVec},
  utils::{functions::hot_rank, fuzzy_search, limit_and_offset},
  ListingType,
  SortType,
};
use tracing::debug;
use typed_builder::TypedBuilder;

type PostViewTuple = (
  Post,
  PersonSafe,
  CommunitySafe,
  Option<CommunityPersonBan>,
  PostAggregates,
  Option<CommunityFollower>,
  Option<PostSaved>,
  Option<PostRead>,
  Option<PersonBlock>,
  Option<i16>,
);

impl PostView {
  pub fn read(
    conn: &mut PgConnection,
    post_id: PostId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));
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
    ) = post::table
      .find(post_id)
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
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
      .select((
        post::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        community_person_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        person_block::all_columns.nullable(),
        post_like::score.nullable(),
      ))
      .first::<PostViewTuple>(conn)?;

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
    })
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostQuery<'a> {
  #[builder(!default)]
  conn: &'a mut PgConnection,
  listing_type: Option<ListingType>,
  sort: Option<SortType>,
  creator_id: Option<PersonId>,
  community_id: Option<CommunityId>,
  community_actor_id: Option<DbUrl>,
  local_user: Option<&'a LocalUser>,
  search_term: Option<String>,
  url_search: Option<String>,
  saved_only: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PostQuery<'a> {
  pub fn list(self) -> Result<Vec<PostView>, Error> {
    use diesel::dsl::*;

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
            .and(community_person_ban::person_id.eq(post::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
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
          community::id
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
        local_user_language::table.on(
          post::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select((
        post::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        community_person_ban::all_columns.nullable(),
        post_aggregates::all_columns,
        community_follower::all_columns.nullable(),
        post_saved::all_columns.nullable(),
        post_read::all_columns.nullable(),
        person_block::all_columns.nullable(),
        post_like::score.nullable(),
      ))
      .into_boxed();

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

    if let Some(community_id) = self.community_id {
      query = query
        .filter(post::community_id.eq(community_id))
        .then_order_by(post_aggregates::stickied.desc());
    }

    if let Some(community_actor_id) = self.community_actor_id {
      query = query
        .filter(community::actor_id.eq(community_actor_id))
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

    // If its for a specific person, show the removed / deleted
    if let Some(creator_id) = self.creator_id {
      query = query.filter(post::creator_id.eq(creator_id));
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
      query = query.filter(post_saved::id.is_not_null());
    }
    // Only hide the read posts, if the saved_only is false. Otherwise ppl with the hide_read
    // setting wont be able to see saved posts.
    else if !self.local_user.map(|l| l.show_read_posts).unwrap_or(true) {
      query = query.filter(post_read::id.is_null());
    }

    if self.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::id.is_not_null());

      // Don't show blocked communities or persons
      query = query.filter(community_block::person_id.is_null());
      query = query.filter(person_block::person_id.is_null());
    }

    query = match self.sort.unwrap_or(SortType::Hot) {
      SortType::Active => query
        .then_order_by(
          hot_rank(
            post_aggregates::score,
            post_aggregates::newest_comment_time_necro,
          )
          .desc(),
        )
        .then_order_by(post_aggregates::newest_comment_time_necro.desc()),
      SortType::Hot => query
        .then_order_by(hot_rank(post_aggregates::score, post_aggregates::published).desc())
        .then_order_by(post_aggregates::published.desc()),
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
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

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
      .into_iter()
      .map(|a| Self {
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
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::post_view::{PostQuery, PostView};
  use diesel::PgConnection;
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    newtypes::LanguageId,
    source::{
      community::*,
      community_block::{CommunityBlock, CommunityBlockForm},
      language::Language,
      local_user::{LocalUser, LocalUserForm},
      local_user_language::LocalUserLanguage,
      person::*,
      person_block::{PersonBlock, PersonBlockForm},
      post::*,
    },
    traits::{Blockable, Crud, Likeable},
    utils::establish_unpooled_connection,
    SortType,
    SubscribedType,
  };
  use serial_test::serial;

  struct Data {
    inserted_person: Person,
    inserted_local_user: LocalUser,
    inserted_blocked_person: Person,
    inserted_bot: Person,
    inserted_community: Community,
    inserted_post: Post,
  }

  fn init_data(conn: &mut PgConnection) -> Data {
    let person_name = "tegan".to_string();

    let new_person = PersonForm {
      name: person_name.to_owned(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let local_user_form = LocalUserForm {
      person_id: Some(inserted_person.id),
      password_encrypted: Some("".to_string()),
      ..Default::default()
    };
    let inserted_local_user = LocalUser::create(conn, &local_user_form).unwrap();

    let new_bot = PersonForm {
      name: "mybot".to_string(),
      bot_account: Some(true),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_bot = Person::create(conn, &new_bot).unwrap();

    let new_community = CommunityForm {
      name: "test_community_3".to_string(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(conn, &new_community).unwrap();

    // Test a person block, make sure the post query doesn't include their post
    let blocked_person = PersonForm {
      name: person_name,
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_blocked_person = Person::create(conn, &blocked_person).unwrap();

    let post_from_blocked_person = PostForm {
      name: "blocked_person_post".to_string(),
      creator_id: inserted_blocked_person.id,
      community_id: inserted_community.id,
      language_id: Some(LanguageId(1)),
      ..PostForm::default()
    };

    Post::create(conn, &post_from_blocked_person).unwrap();

    // block that person
    let person_block = PersonBlockForm {
      person_id: inserted_person.id,
      target_id: inserted_blocked_person.id,
    };

    PersonBlock::block(conn, &person_block).unwrap();

    // A sample post
    let new_post = PostForm {
      name: "test post 3".to_string(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      language_id: Some(LanguageId(47)),
      ..PostForm::default()
    };

    let inserted_post = Post::create(conn, &new_post).unwrap();

    let new_bot_post = PostForm {
      name: "test bot post".to_string(),
      creator_id: inserted_bot.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let _inserted_bot_post = Post::create(conn, &new_bot_post).unwrap();

    Data {
      inserted_person,
      inserted_local_user,
      inserted_blocked_person,
      inserted_bot,
      inserted_community,
      inserted_post,
    }
  }

  #[test]
  #[serial]
  fn post_listing_with_person() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let local_user_form = LocalUserForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    let inserted_local_user =
      LocalUser::update(conn, data.inserted_local_user.id, &local_user_form).unwrap();

    let read_post_listing = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&inserted_local_user))
      .build()
      .list()
      .unwrap();

    let post_listing_single_with_person =
      PostView::read(conn, data.inserted_post.id, Some(data.inserted_person.id)).unwrap();

    let mut expected_post_listing_with_user = expected_post_view(&data, conn);

    // Should be only one person, IE the bot post, and blocked should be missing
    assert_eq!(1, read_post_listing.len());

    assert_eq!(expected_post_listing_with_user, read_post_listing[0]);
    expected_post_listing_with_user.my_vote = Some(0);
    assert_eq!(
      expected_post_listing_with_user,
      post_listing_single_with_person
    );

    let local_user_form = LocalUserForm {
      show_bot_accounts: Some(true),
      ..Default::default()
    };
    let inserted_local_user =
      LocalUser::update(conn, data.inserted_local_user.id, &local_user_form).unwrap();

    let post_listings_with_bots = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&inserted_local_user))
      .build()
      .list()
      .unwrap();
    // should include bot post which has "undetermined" language
    assert_eq!(2, post_listings_with_bots.len());

    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn post_listing_no_person() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let read_post_listing_multiple_no_person = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .build()
      .list()
      .unwrap();

    let read_post_listing_single_no_person =
      PostView::read(conn, data.inserted_post.id, None).unwrap();

    let expected_post_listing_no_person = expected_post_view(&data, conn);

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

    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn post_listing_block_community() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let community_block = CommunityBlockForm {
      person_id: data.inserted_person.id,
      community_id: data.inserted_community.id,
    };
    CommunityBlock::block(conn, &community_block).unwrap();

    let read_post_listings_with_person_after_block = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .community_id(Some(data.inserted_community.id))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();
    // Should be 0 posts after the community block
    assert_eq!(0, read_post_listings_with_person_after_block.len());

    CommunityBlock::unblock(conn, &community_block).unwrap();
    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn post_listing_like() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let post_like_form = PostLikeForm {
      post_id: data.inserted_post.id,
      person_id: data.inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: data.inserted_post.id,
      person_id: data.inserted_person.id,
      published: inserted_post_like.published,
      score: 1,
    };
    assert_eq!(expected_post_like, inserted_post_like);

    let like_removed =
      PostLike::remove(conn, data.inserted_person.id, data.inserted_post.id).unwrap();
    assert_eq!(1, like_removed);
    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn post_listing_person_language() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let spanish_id = Language::read_id_from_code(conn, "es").unwrap();
    let post_spanish = PostForm {
      name: "asffgdsc".to_string(),
      creator_id: data.inserted_person.id,
      community_id: data.inserted_community.id,
      language_id: Some(spanish_id),
      ..PostForm::default()
    };

    Post::create(conn, &post_spanish).unwrap();

    let post_listings_all = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();

    // no language filters specified, all posts should be returned
    assert_eq!(3, post_listings_all.len());

    let french_id = Language::read_id_from_code(conn, "fr").unwrap();
    LocalUserLanguage::update_user_languages(
      conn,
      Some(vec![french_id]),
      data.inserted_local_user.id,
    )
    .unwrap();

    let post_listing_french = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();

    // only one french language post should be returned
    assert_eq!(1, post_listing_french.len());
    assert_eq!(french_id, post_listing_french[0].post.language_id);

    let undetermined_id = Language::read_id_from_code(conn, "und").unwrap();
    LocalUserLanguage::update_user_languages(
      conn,
      Some(vec![french_id, undetermined_id]),
      data.inserted_local_user.id,
    )
    .unwrap();
    let post_listings_french_und = PostQuery::builder()
      .conn(conn)
      .sort(Some(SortType::New))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();

    // french post and undetermined language post should be returned
    assert_eq!(2, post_listings_french_und.len());
    assert_eq!(
      undetermined_id,
      post_listings_french_und[0].post.language_id
    );
    assert_eq!(french_id, post_listings_french_und[1].post.language_id);

    cleanup(data, conn);
  }

  fn cleanup(data: Data, conn: &mut PgConnection) {
    let num_deleted = Post::delete(conn, data.inserted_post.id).unwrap();
    Community::delete(conn, data.inserted_community.id).unwrap();
    Person::delete(conn, data.inserted_person.id).unwrap();
    Person::delete(conn, data.inserted_bot.id).unwrap();
    Person::delete(conn, data.inserted_blocked_person.id).unwrap();
    assert_eq!(1, num_deleted);
  }

  fn expected_post_view(data: &Data, conn: &mut PgConnection) -> PostView {
    let (inserted_person, inserted_community, inserted_post) = (
      &data.inserted_person,
      &data.inserted_community,
      &data.inserted_post,
    );
    let agg = PostAggregates::read(conn, inserted_post.id).unwrap();

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
        stickied: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_video_url: None,
        thumbnail_url: None,
        ap_id: inserted_post.ap_id.to_owned(),
        local: true,
        language_id: LanguageId(47),
      },
      my_vote: None,
      creator: PersonSafe {
        id: inserted_person.id,
        name: inserted_person.name.clone(),
        display_name: None,
        published: inserted_person.published,
        avatar: None,
        actor_id: inserted_person.actor_id.to_owned(),
        local: true,
        admin: false,
        bot_account: false,
        banned: false,
        deleted: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_person.inbox_url.to_owned(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
      },
      creator_banned_from_community: false,
      community: CommunitySafe {
        id: inserted_community.id,
        name: inserted_community.name.clone(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.to_owned(),
        local: true,
        title: "nada".to_owned(),
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: inserted_community.published,
      },
      counts: PostAggregates {
        id: agg.id,
        post_id: inserted_post.id,
        comments: 0,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        stickied: false,
        published: agg.published,
        newest_comment_time_necro: inserted_post.published,
        newest_comment_time: inserted_post.published,
      },
      subscribed: SubscribedType::NotSubscribed,
      read: false,
      saved: false,
      creator_blocked: false,
    }
  }
}
