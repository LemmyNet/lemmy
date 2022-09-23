use crate::structs::CommentView;
use diesel::{dsl::*, result::Error, *};
use diesel_ltree::{nlevel, subpath, Ltree, LtreeExtensions};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregates,
  newtypes::{CommentId, CommunityId, DbUrl, LocalUserId, PersonId, PostId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_block,
    community_follower,
    community_person_ban,
    local_user_language,
    person,
    person_block,
    post,
  },
  source::{
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    local_user::LocalUser,
    person::{Person, PersonSafe},
    person_block::PersonBlock,
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::{functions::hot_rank, fuzzy_search, limit_and_offset_unlimited},
  CommentSortType,
  ListingType,
};
use typed_builder::TypedBuilder;

type CommentViewTuple = (
  Comment,
  PersonSafe,
  Post,
  CommunitySafe,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl CommentView {
  pub fn read(
    conn: &mut PgConnection,
    comment_id: CommentId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (
      comment,
      creator,
      post,
      community,
      counts,
      creator_banned_from_community,
      follower,
      saved,
      creator_blocked,
      comment_like,
    ) = comment::table
      .find(comment_id)
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<CommentViewTuple>(conn)?;

    // If a person is given, then my_vote, if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    let my_vote = if my_person_id.is_some() && comment_like.is_none() {
      Some(0)
    } else {
      comment_like
    };

    Ok(CommentView {
      comment,
      post,
      creator,
      community,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: CommunityFollower::to_subscribed_type(&follower),
      saved: saved.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
    })
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentQuery<'a> {
  #[builder(!default)]
  conn: &'a mut PgConnection,
  listing_type: Option<ListingType>,
  sort: Option<CommentSortType>,
  community_id: Option<CommunityId>,
  community_actor_id: Option<DbUrl>,
  post_id: Option<PostId>,
  parent_path: Option<Ltree>,
  creator_id: Option<PersonId>,
  local_user: Option<&'a LocalUser>,
  search_term: Option<String>,
  saved_only: Option<bool>,
  show_deleted_and_removed: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
  max_depth: Option<i32>,
}

impl<'a> CommentQuery<'a> {
  pub fn list(self) -> Result<Vec<CommentView>, Error> {
    use diesel::dsl::*;

    // The left join below will return None in this case
    let person_id_join = self.local_user.map(|l| l.person_id).unwrap_or(PersonId(-1));
    let local_user_id_join = self.local_user.map(|l| l.id).unwrap_or(LocalUserId(-1));

    let mut query = comment::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
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
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        local_user_language::table.on(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select((
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    if let Some(creator_id) = self.creator_id {
      query = query.filter(comment::creator_id.eq(creator_id));
    };

    if let Some(post_id) = self.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = self.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };

    if let Some(search_term) = self.search_term {
      query = query.filter(comment::content.ilike(fuzzy_search(&search_term)));
    };

    if let Some(listing_type) = self.listing_type {
      match listing_type {
        ListingType::Subscribed => {
          query = query.filter(community_follower::person_id.is_not_null())
        } // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
        ListingType::Local => {
          query = query.filter(community::local.eq(true)).filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          )
        }
        ListingType::All => {
          query = query.filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          )
        }
      }
    };

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if let Some(community_actor_id) = self.community_actor_id {
      query = query.filter(community::actor_id.eq(community_actor_id))
    }

    if self.saved_only.unwrap_or(false) {
      query = query.filter(comment_saved::id.is_not_null());
    }

    if !self.show_deleted_and_removed.unwrap_or(true) {
      query = query.filter(comment::deleted.eq(false));
      query = query.filter(comment::removed.eq(false));
    }

    if !self.local_user.map(|l| l.show_bot_accounts).unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    if self.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::id.is_not_null());

      // Don't show blocked communities or persons
      query = query.filter(community_block::person_id.is_null());
      query = query.filter(person_block::person_id.is_null());
    }

    // A Max depth given means its a tree fetch
    let (limit, offset) = if let Some(max_depth) = self.max_depth {
      let depth_limit = if let Some(parent_path) = self.parent_path.as_ref() {
        parent_path.0.split('.').count() as i32 + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // Always order by the parent path first
      query = query.order_by(subpath(comment::path, 0, -1));

      // TODO limit question. Limiting does not work for comment threads ATM, only max_depth
      // For now, don't do any limiting for tree fetches
      // https://stackoverflow.com/questions/72983614/postgres-ltree-how-to-limit-the-max-number-of-children-at-any-given-level

      // Don't use the regular error-checking one, many more comments must ofter be fetched.
      // This does not work for comment trees, and the limit should be manually set to a high number
      //
      // If a max depth is given, then you know its a tree fetch, and limits should be ignored
      (i64::MAX, 0)
    } else {
      limit_and_offset_unlimited(self.page, self.limit)
    };

    query = match self.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    // Note: deleted and removed comments are done on the front side
    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentViewTuple>(self.conn)?;

    Ok(CommentView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommentView {
  type DbTuple = CommentViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        comment: a.0,
        creator: a.1,
        post: a.2,
        community: a.3,
        counts: a.4,
        creator_banned_from_community: a.5.is_some(),
        subscribed: CommunityFollower::to_subscribed_type(&a.6),
        saved: a.7.is_some(),
        creator_blocked: a.8.is_some(),
        my_vote: a.9,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::comment_view::*;
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    newtypes::LanguageId,
    source::{
      comment::*,
      community::*,
      language::Language,
      local_user::LocalUserForm,
      local_user_language::LocalUserLanguage,
      person::*,
      person_block::PersonBlockForm,
      post::*,
    },
    traits::{Blockable, Crud, Likeable},
    utils::establish_unpooled_connection,
    SubscribedType,
  };
  use serial_test::serial;

  struct Data {
    inserted_comment_0: Comment,
    inserted_comment_1: Comment,
    inserted_comment_2: Comment,
    inserted_post: Post,
    inserted_person: Person,
    inserted_local_user: LocalUser,
    inserted_person_2: Person,
    inserted_community: Community,
  }

  fn init_data(conn: &mut PgConnection) -> Data {
    let new_person = PersonForm {
      name: "timmy".into(),
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

    let new_person_2 = PersonForm {
      name: "sara".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };
    let inserted_person_2 = Person::create(conn, &new_person_2).unwrap();

    let new_community = CommunityForm {
      name: "test community 5".to_string(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post 2".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(conn, &new_post).unwrap();

    // Create a comment tree with this hierarchy
    //       0
    //     \     \
    //    1      2
    //    \
    //  3  4
    //     \
    //     5
    let comment_form_0 = CommentForm {
      content: "Comment 0".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment_0 = Comment::create(conn, &comment_form_0, None).unwrap();

    let comment_form_1 = CommentForm {
      content: "Comment 1, A test blocked comment".into(),
      creator_id: inserted_person_2.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment_1 =
      Comment::create(conn, &comment_form_1, Some(&inserted_comment_0.path)).unwrap();

    let finnish_id = Language::read_id_from_code(conn, "fi").unwrap();
    let comment_form_2 = CommentForm {
      content: "Comment 2".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      language_id: Some(finnish_id),
      ..CommentForm::default()
    };

    let inserted_comment_2 =
      Comment::create(conn, &comment_form_2, Some(&inserted_comment_0.path)).unwrap();

    let comment_form_3 = CommentForm {
      content: "Comment 3".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let _inserted_comment_3 =
      Comment::create(conn, &comment_form_3, Some(&inserted_comment_1.path)).unwrap();

    let polish_id = Language::read_id_from_code(conn, "pl").unwrap();
    let comment_form_4 = CommentForm {
      content: "Comment 4".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      language_id: Some(polish_id),
      ..CommentForm::default()
    };

    let inserted_comment_4 =
      Comment::create(conn, &comment_form_4, Some(&inserted_comment_1.path)).unwrap();

    let comment_form_5 = CommentForm {
      content: "Comment 5".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let _inserted_comment_5 =
      Comment::create(conn, &comment_form_5, Some(&inserted_comment_4.path)).unwrap();

    let timmy_blocks_sara_form = PersonBlockForm {
      person_id: inserted_person.id,
      target_id: inserted_person_2.id,
    };

    let inserted_block = PersonBlock::block(conn, &timmy_blocks_sara_form).unwrap();

    let expected_block = PersonBlock {
      id: inserted_block.id,
      person_id: inserted_person.id,
      target_id: inserted_person_2.id,
      published: inserted_block.published,
    };
    assert_eq!(expected_block, inserted_block);

    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment_0.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(conn, &comment_like_form).unwrap();

    Data {
      inserted_comment_0,
      inserted_comment_1,
      inserted_comment_2,
      inserted_post,
      inserted_person,
      inserted_local_user,
      inserted_person_2,
      inserted_community,
    }
  }

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let expected_comment_view_no_person = expected_comment_view(&data, conn);

    let mut expected_comment_view_with_person = expected_comment_view_no_person.to_owned();
    expected_comment_view_with_person.my_vote = Some(1);

    let read_comment_views_no_person = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .build()
      .list()
      .unwrap();

    assert_eq!(
      expected_comment_view_no_person,
      read_comment_views_no_person[0]
    );

    let read_comment_views_with_person = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();

    assert_eq!(
      expected_comment_view_with_person,
      read_comment_views_with_person[0]
    );

    // Make sure its 1, not showing the blocked comment
    assert_eq!(5, read_comment_views_with_person.len());

    let read_comment_from_blocked_person = CommentView::read(
      conn,
      data.inserted_comment_1.id,
      Some(data.inserted_person.id),
    )
    .unwrap();

    // Make sure block set the creator blocked
    assert!(read_comment_from_blocked_person.creator_blocked);

    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn test_comment_tree() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    let top_path = data.inserted_comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(top_path))
      .build()
      .list()
      .unwrap();

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(child_path))
      .build()
      .list()
      .unwrap();

    // Make sure the comment parent-limited fetch is correct
    assert_eq!(6, read_comment_views_top_path.len());
    assert_eq!(4, read_comment_views_child_path.len());

    // Make sure it contains the parent, but not the comment from the other tree
    let child_comments = read_comment_views_child_path
      .into_iter()
      .map(|c| c.comment)
      .collect::<Vec<Comment>>();
    assert!(child_comments.contains(&data.inserted_comment_1));
    assert!(!child_comments.contains(&data.inserted_comment_2));

    let read_comment_views_top_max_depth = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .max_depth(Some(1))
      .build()
      .list()
      .unwrap();

    // Make sure a depth limited one only has the top comment
    assert_eq!(
      expected_comment_view(&data, conn),
      read_comment_views_top_max_depth[0]
    );
    assert_eq!(1, read_comment_views_top_max_depth.len());

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_parent_max_depth = CommentQuery::builder()
      .conn(conn)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(child_path))
      .max_depth(Some(1))
      .sort(Some(CommentSortType::New))
      .build()
      .list()
      .unwrap();

    // Make sure a depth limited one, and given child comment 1, has 3
    assert!(read_comment_views_parent_max_depth[2]
      .comment
      .content
      .eq("Comment 3"));
    assert_eq!(3, read_comment_views_parent_max_depth.len());

    cleanup(data, conn);
  }

  #[test]
  #[serial]
  fn test_languages() {
    let conn = &mut establish_unpooled_connection();
    let data = init_data(conn);

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery::builder()
      .conn(conn)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();
    assert_eq!(5, all_languages.len());

    // change user lang to finnish, should only show single finnish comment
    let finnish_id = Language::read_id_from_code(conn, "fi").unwrap();
    LocalUserLanguage::update_user_languages(
      conn,
      Some(vec![finnish_id]),
      data.inserted_local_user.id,
    )
    .unwrap();
    let finnish_comment = CommentQuery::builder()
      .conn(conn)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();
    assert_eq!(1, finnish_comment.len());
    assert_eq!(
      data.inserted_comment_2.content,
      finnish_comment[0].comment.content
    );
    assert_eq!(finnish_id, finnish_comment[0].comment.language_id);

    // now show all comments with undetermined language (which is the default value)
    let undetermined_id = Language::read_id_from_code(conn, "und").unwrap();
    LocalUserLanguage::update_user_languages(
      conn,
      Some(vec![undetermined_id]),
      data.inserted_local_user.id,
    )
    .unwrap();
    let undetermined_comment = CommentQuery::builder()
      .conn(conn)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .unwrap();
    assert_eq!(3, undetermined_comment.len());

    cleanup(data, conn);
  }

  fn cleanup(data: Data, conn: &mut PgConnection) {
    CommentLike::remove(conn, data.inserted_person.id, data.inserted_comment_0.id).unwrap();
    Comment::delete(conn, data.inserted_comment_0.id).unwrap();
    Comment::delete(conn, data.inserted_comment_1.id).unwrap();
    Post::delete(conn, data.inserted_post.id).unwrap();
    Community::delete(conn, data.inserted_community.id).unwrap();
    Person::delete(conn, data.inserted_person.id).unwrap();
    Person::delete(conn, data.inserted_person_2.id).unwrap();
  }

  fn expected_comment_view(data: &Data, conn: &mut PgConnection) -> CommentView {
    let agg = CommentAggregates::read(conn, data.inserted_comment_0.id).unwrap();
    CommentView {
      creator_banned_from_community: false,
      my_vote: None,
      subscribed: SubscribedType::NotSubscribed,
      saved: false,
      creator_blocked: false,
      comment: Comment {
        id: data.inserted_comment_0.id,
        content: "Comment 0".into(),
        creator_id: data.inserted_person.id,
        post_id: data.inserted_post.id,
        removed: false,
        deleted: false,
        published: data.inserted_comment_0.published,
        ap_id: data.inserted_comment_0.ap_id.clone(),
        updated: None,
        local: true,
        distinguished: false,
        path: data.inserted_comment_0.to_owned().path,
        language_id: LanguageId(0),
      },
      creator: PersonSafe {
        id: data.inserted_person.id,
        name: "timmy".into(),
        display_name: None,
        published: data.inserted_person.published,
        avatar: None,
        actor_id: data.inserted_person.actor_id.to_owned(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: data.inserted_person.inbox_url.to_owned(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
      },
      post: Post {
        id: data.inserted_post.id,
        name: data.inserted_post.name.to_owned(),
        creator_id: data.inserted_person.id,
        url: None,
        body: None,
        published: data.inserted_post.published,
        updated: None,
        community_id: data.inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        stickied: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_video_url: None,
        thumbnail_url: None,
        ap_id: data.inserted_post.ap_id.to_owned(),
        local: true,
        language_id: Default::default(),
      },
      community: CommunitySafe {
        id: data.inserted_community.id,
        name: "test community 5".to_string(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: data.inserted_community.actor_id.to_owned(),
        local: true,
        title: "nada".to_owned(),
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: data.inserted_community.published,
      },
      counts: CommentAggregates {
        id: agg.id,
        comment_id: data.inserted_comment_0.id,
        score: 1,
        upvotes: 1,
        downvotes: 0,
        published: agg.published,
        child_count: 5,
      },
    }
  }
}
