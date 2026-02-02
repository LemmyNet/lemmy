#![expect(clippy::indexing_slicing, clippy::expect_used)]

use crate::{PostView, impls::PostQuery};
use chrono::{DateTime, Days, Utc};
use diesel_async::SimpleAsyncConnection;
use diesel_uplete::UpleteCount;
use lemmy_db_schema::{
  impls::actor_language::UNDETERMINED_ID,
  newtypes::{LanguageId, PostId},
  source::{
    actor_language::LocalUserLanguage,
    comment::{Comment, CommentInsertForm},
    community::{
      Community,
      CommunityActions,
      CommunityBlockForm,
      CommunityFollowerForm,
      CommunityInsertForm,
      CommunityModeratorForm,
      CommunityPersonBanForm,
      CommunityUpdateForm,
    },
    community_tag::{CommunityTag, CommunityTagInsertForm, PostCommunityTag},
    instance::{
      Instance,
      InstanceActions,
      InstanceBanForm,
      InstanceCommunitiesBlockForm,
      InstancePersonsBlockForm,
    },
    keyword_block::LocalUserKeywordBlock,
    language::Language,
    local_site::{LocalSite, LocalSiteUpdateForm},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    multi_community::{MultiCommunity, MultiCommunityInsertForm},
    person::{Person, PersonActions, PersonBlockForm, PersonInsertForm, PersonNoteForm},
    post::{Post, PostActions, PostHideForm, PostInsertForm, PostLikeForm, PostUpdateForm},
    site::Site,
  },
  test_data::TestData,
  traits::{Bannable, Blockable, Followable, Likeable},
};
use lemmy_db_schema_file::enums::{
  CommunityFollowerState,
  CommunityVisibility,
  ListingType,
  PostSortType,
  TagColor,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{
  connection::{ActualDbPool, DbPool, build_db_pool, get_conn},
  pagination::PaginationCursor,
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::{
  collections::HashSet,
  time::{Duration, Instant},
};
use test_context::{AsyncTestContext, test_context};
use url::Url;

const POST_BY_BLOCKED_PERSON: &str = "post by blocked person";
const POST_BY_BOT: &str = "post by bot";
const POST: &str = "post";
const POST_WITH_TAGS: &str = "post with tags";
const POST_KEYWORD_BLOCKED: &str = "blocked_keyword";

fn names(post_views: &[PostView]) -> Vec<&str> {
  post_views.iter().map(|i| i.post.name.as_str()).collect()
}

struct Data {
  pool: ActualDbPool,
  instance: Instance,
  tegan: LocalUserView,
  john: LocalUserView,
  bot: LocalUserView,
  community: Community,
  post: Post,
  bot_post: Post,
  post_with_tags: Post,
  tag_1: CommunityTag,
  tag_2: CommunityTag,
  site: Site,
}

impl Data {
  fn pool(&self) -> ActualDbPool {
    self.pool.clone()
  }
  pub fn pool2(&self) -> DbPool<'_> {
    DbPool::Pool(&self.pool)
  }
  fn default_post_query(&self) -> PostQuery<'_> {
    PostQuery {
      sort: Some(PostSortType::New),
      local_user: Some(&self.tegan.local_user),
      ..Default::default()
    }
  }

  async fn setup() -> LemmyResult<Data> {
    let actual_pool = build_db_pool()?;
    let pool = &mut (&actual_pool).into();
    let data = TestData::create(pool).await?;

    let tegan_person_form = PersonInsertForm::test_form(data.instance.id, "tegan");
    let inserted_tegan_person = Person::create(pool, &tegan_person_form).await?;
    let tegan_local_user_form = LocalUserInsertForm {
      admin: Some(true),
      ..LocalUserInsertForm::test_form(inserted_tegan_person.id)
    };
    let inserted_tegan_local_user = LocalUser::create(pool, &tegan_local_user_form, vec![]).await?;

    let bot_person_form = PersonInsertForm {
      bot_account: Some(true),
      ..PersonInsertForm::test_form(data.instance.id, "mybot")
    };
    let inserted_bot_person = Person::create(pool, &bot_person_form).await?;
    let inserted_bot_local_user = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form(inserted_bot_person.id),
      vec![],
    )
    .await?;

    let new_community = CommunityInsertForm::new(
      data.instance.id,
      "test_community_3".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &new_community).await?;

    // Test a person block, make sure the post query doesn't include their post
    let john_person_form = PersonInsertForm::test_form(data.instance.id, "john");
    let inserted_john_person = Person::create(pool, &john_person_form).await?;
    let inserted_john_local_user = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form(inserted_john_person.id),
      vec![],
    )
    .await?;

    let post_from_blocked_person = PostInsertForm {
      language_id: Some(LanguageId(1)),
      ..PostInsertForm::new(
        POST_BY_BLOCKED_PERSON.to_string(),
        inserted_john_person.id,
        community.id,
      )
    };
    Post::create(pool, &post_from_blocked_person).await?;

    // block that person
    let person_block = PersonBlockForm::new(inserted_tegan_person.id, inserted_john_person.id);
    PersonActions::block(pool, &person_block).await?;

    LocalUserKeywordBlock::update(
      pool,
      vec![POST_KEYWORD_BLOCKED.to_string()],
      inserted_tegan_local_user.id,
    )
    .await?;

    // Two community post tags
    let tag_1 = CommunityTag::create(
      pool,
      &CommunityTagInsertForm {
        ap_id: Url::parse(&format!("{}/tags/test_tag1", community.ap_id))?.into(),
        name: "Test Tag 1".into(),
        display_name: None,
        summary: None,
        community_id: community.id,
        deleted: Some(false),
        color: Some(TagColor::Color01),
      },
    )
    .await?;
    let tag_2 = CommunityTag::create(
      pool,
      &CommunityTagInsertForm {
        ap_id: Url::parse(&format!("{}/tags/test_tag2", community.ap_id))?.into(),
        name: "Test Tag 2".into(),
        display_name: None,
        summary: None,
        community_id: community.id,
        deleted: Some(false),
        color: Some(TagColor::Color02),
      },
    )
    .await?;

    // A sample post
    let new_post = PostInsertForm {
      language_id: Some(LanguageId(47)),
      ..PostInsertForm::new(POST.to_string(), inserted_tegan_person.id, community.id)
    };

    let post = Post::create(pool, &new_post).await?;

    let new_bot_post = PostInsertForm::new(
      POST_BY_BOT.to_string(),
      inserted_bot_person.id,
      community.id,
    );
    let bot_post = Post::create(pool, &new_bot_post).await?;

    // A sample post with tags
    let new_post = PostInsertForm {
      language_id: Some(LanguageId(47)),
      ..PostInsertForm::new(
        POST_WITH_TAGS.to_string(),
        inserted_tegan_person.id,
        community.id,
      )
    };

    let post_with_tags = Post::create(pool, &new_post).await?;
    PostCommunityTag::update(pool, &post_with_tags, &[tag_1.id, tag_2.id]).await?;

    let tegan = LocalUserView {
      local_user: inserted_tegan_local_user,
      person: inserted_tegan_person,
      banned: false,
      ban_expires_at: None,
    };
    let john = LocalUserView {
      local_user: inserted_john_local_user,
      person: inserted_john_person,
      banned: false,
      ban_expires_at: None,
    };

    let bot = LocalUserView {
      local_user: inserted_bot_local_user,
      person: inserted_bot_person,
      banned: false,
      ban_expires_at: None,
    };

    Ok(Data {
      pool: actual_pool,
      instance: data.instance,
      tegan,
      john,
      bot,
      community,
      post,
      bot_post,
      post_with_tags,
      tag_1,
      tag_2,
      site: data.site,
    })
  }
  async fn teardown(data: Data) -> LemmyResult<()> {
    let pool = &mut data.pool2();
    let num_deleted = Post::delete(pool, data.post.id).await?;
    Community::delete(pool, data.community.id).await?;
    Person::delete(pool, data.tegan.person.id).await?;
    Person::delete(pool, data.bot.person.id).await?;
    Person::delete(pool, data.john.person.id).await?;
    Site::delete(pool, data.site.id).await?;
    Instance::delete(pool, data.instance.id).await?;
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
impl AsyncTestContext for Data {
  async fn setup() -> Self {
    Data::setup().await.expect("setup failed")
  }
  async fn teardown(self) {
    Data::teardown(self).await.expect("teardown failed")
  }
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_with_person(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let local_user_form = LocalUserUpdateForm {
    show_bot_accounts: Some(false),
    ..Default::default()
  };
  LocalUser::update(pool, data.tegan.local_user.id, &local_user_form).await?;
  data.tegan.local_user.show_bot_accounts = false;

  let mut read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?
  .items;
  // remove tags post
  read_post_listing.remove(0);

  let post_listing_single_with_person = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(
    vec![post_listing_single_with_person.clone()],
    read_post_listing
  );
  assert_eq!(data.post.id, post_listing_single_with_person.post.id);

  let local_user_form = LocalUserUpdateForm {
    show_bot_accounts: Some(true),
    ..Default::default()
  };
  LocalUser::update(pool, data.tegan.local_user.id, &local_user_form).await?;
  data.tegan.local_user.show_bot_accounts = true;

  let post_listings_with_bots = PostQuery {
    community_id: Some(data.community.id),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?;
  // should include bot post which has "undetermined" language
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_with_bots)
  );
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_no_person(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let read_post_listing_multiple_no_person = PostQuery {
    community_id: Some(data.community.id),
    local_user: None,
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?;

  let read_post_listing_single_no_person =
    PostView::read(pool, data.post.id, None, data.instance.id, false).await?;

  // Should be 2 posts, with the bot post, and the blocked
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST, POST_BY_BLOCKED_PERSON],
    names(&read_post_listing_multiple_no_person)
  );

  assert!(
    read_post_listing_multiple_no_person
      .get(2)
      .is_some_and(|x| x.post.id == data.post.id)
  );
  assert_eq!(false, read_post_listing_single_no_person.can_mod);
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_block_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let community_block = CommunityBlockForm::new(data.community.id, data.tegan.person.id);
  CommunityActions::block(pool, &community_block).await?;

  let read_post_listings_with_person_after_block = PostQuery {
    community_id: Some(data.community.id),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?;
  // Should be 0 posts after the community block
  assert_eq!(read_post_listings_with_person_after_block.items, vec![]);

  CommunityActions::unblock(pool, &community_block).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_like(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let post_like_form = PostLikeForm::new(data.post.id, data.tegan.person.id, Some(true));

  let inserted_post_like = PostActions::like(pool, &post_like_form).await?;

  assert_eq!(
    (data.post.id, data.tegan.person.id, Some(true)),
    (
      inserted_post_like.post_id,
      inserted_post_like.person_id,
      inserted_post_like.vote_is_upvote,
    )
  );

  let post_listing_single_with_person = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(
    (true, true, 1, 1, 1),
    (
      post_listing_single_with_person
        .post_actions
        .is_some_and(|t| t.vote_is_upvote == Some(true)),
      // Make sure person actions is none so you don't get a voted_at for your own user
      post_listing_single_with_person.person_actions.is_none(),
      post_listing_single_with_person.post.score,
      post_listing_single_with_person.post.upvotes,
      post_listing_single_with_person.creator.post_score,
    )
  );

  let local_user_form = LocalUserUpdateForm {
    show_bot_accounts: Some(false),
    ..Default::default()
  };
  LocalUser::update(pool, data.tegan.local_user.id, &local_user_form).await?;
  data.tegan.local_user.show_bot_accounts = false;

  let mut read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?
  .items;
  read_post_listing.remove(0);
  assert_eq!(
    post_listing_single_with_person.post.id,
    read_post_listing[0].post.id
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn person_note(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let note_str = "Tegan loves cats.";

  let note_form = PersonNoteForm::new(
    data.john.person.id,
    data.tegan.person.id,
    note_str.to_string(),
  );
  let inserted_note = PersonActions::note(pool, &note_form).await?;
  assert_eq!(Some(note_str.to_string()), inserted_note.note);

  let post_listing = PostView::read(
    pool,
    data.post.id,
    Some(&data.john.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert!(
    post_listing
      .person_actions
      .is_some_and(|t| t.note == Some(note_str.to_string()) && t.noted_at.is_some())
  );

  let note_removed =
    PersonActions::delete_note(pool, data.john.person.id, data.tegan.person.id).await?;

  let post_listing = PostView::read(
    pool,
    data.post.id,
    Some(&data.john.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(UpleteCount::only_deleted(1), note_removed);
  assert!(post_listing.person_actions.is_none());

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_person_vote_totals(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Create a 2nd bot post, to do multiple votes
  let bot_post_2 = PostInsertForm::new(
    "Bot post 2".to_string(),
    data.bot.person.id,
    data.community.id,
  );
  let bot_post_2 = Post::create(pool, &bot_post_2).await?;

  let post_like_form = PostLikeForm::new(data.bot_post.id, data.tegan.person.id, Some(true));
  let inserted_post_like = PostActions::like(pool, &post_like_form).await?;

  assert_eq!(
    (data.bot_post.id, data.tegan.person.id, Some(true)),
    (
      inserted_post_like.post_id,
      inserted_post_like.person_id,
      inserted_post_like.vote_is_upvote,
    )
  );

  let inserted_person_like = PersonActions::like(
    pool,
    data.tegan.person.id,
    data.bot.person.id,
    None,
    Some(true),
  )
  .await?;

  assert_eq!(
    (data.tegan.person.id, data.bot.person.id, Some(1), Some(0),),
    (
      inserted_person_like.person_id,
      inserted_person_like.target_id,
      inserted_person_like.upvotes,
      inserted_person_like.downvotes,
    )
  );

  let post_listing = PostView::read(
    pool,
    data.bot_post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(
    (true, true, true, 1, 1, 1),
    (
      post_listing
        .post_actions
        .is_some_and(|t| t.vote_is_upvote == Some(true)),
      post_listing
        .person_actions
        .as_ref()
        .is_some_and(|t| t.upvotes == Some(1)),
      post_listing
        .person_actions
        .as_ref()
        .is_some_and(|t| t.downvotes == Some(0)),
      post_listing.post.score,
      post_listing.post.upvotes,
      post_listing.creator.post_score,
    )
  );

  // Do a 2nd like to another post
  let post_2_like_form = PostLikeForm::new(bot_post_2.id, data.tegan.person.id, Some(true));
  PostActions::like(pool, &post_2_like_form).await?;

  let inserted_person_like_2 = PersonActions::like(
    pool,
    data.tegan.person.id,
    data.bot.person.id,
    None,
    Some(true),
  )
  .await?;
  assert_eq!(
    (data.tegan.person.id, data.bot.person.id, Some(2), Some(0),),
    (
      inserted_person_like_2.person_id,
      inserted_person_like_2.target_id,
      inserted_person_like_2.upvotes,
      inserted_person_like_2.downvotes,
    )
  );

  // Remove the like
  let form = PostLikeForm::new(data.bot_post.id, data.tegan.person.id, None);
  PostActions::like(pool, &form).await?;

  let person_like_removed = PersonActions::like(
    pool,
    data.tegan.person.id,
    data.bot.person.id,
    Some(true),
    None,
  )
  .await?;
  assert_eq!(
    (data.tegan.person.id, data.bot.person.id, Some(1), Some(0),),
    (
      person_like_removed.person_id,
      person_like_removed.target_id,
      person_like_removed.upvotes,
      person_like_removed.downvotes,
    )
  );

  // Now do a downvote
  let post_like_form = PostLikeForm::new(data.bot_post.id, data.tegan.person.id, Some(false));
  PostActions::like(pool, &post_like_form).await?;
  let inserted_person_dislike = PersonActions::like(
    pool,
    data.tegan.person.id,
    data.bot.person.id,
    None,
    Some(false),
  )
  .await?;
  assert_eq!(
    (data.tegan.person.id, data.bot.person.id, Some(1), Some(1),),
    (
      inserted_person_dislike.person_id,
      inserted_person_dislike.target_id,
      inserted_person_dislike.upvotes,
      inserted_person_dislike.downvotes,
    )
  );

  let post_listing = PostView::read(
    pool,
    data.bot_post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(
    (true, true, true, -1, 1, 0),
    (
      post_listing
        .post_actions
        .is_some_and(|t| t.vote_is_upvote == Some(false)),
      post_listing
        .person_actions
        .as_ref()
        .is_some_and(|t| t.upvotes == Some(1)),
      post_listing
        .person_actions
        .as_ref()
        .is_some_and(|t| t.downvotes == Some(1)),
      post_listing.post.score,
      post_listing.post.downvotes,
      post_listing.creator.post_score,
    )
  );

  let form = PostLikeForm::new(data.bot_post.id, data.tegan.person.id, None);
  PostActions::like(pool, &form).await?;

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_read_only(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Mark the bot post, then the tags post as read
  PostActions::mark_as_read(pool, data.tegan.person.id, &[data.bot_post.id]).await?;

  PostActions::mark_as_read(pool, data.tegan.person.id, &[data.post_with_tags.id]).await?;

  let read_read_post_listing =
    PostView::list_read(pool, &data.tegan.person, None, None, None).await?;

  // This should be ordered from most recently read
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT],
    names(&read_read_post_listing)
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn creator_info(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();
  let community_id = data.community.id;

  let tegan_listings = PostQuery {
    community_id: Some(community_id),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?
  .into_iter()
  .map(|p| (p.creator.name, p.creator_is_moderator, p.can_mod))
  .collect::<Vec<_>>();

  // Tegan is an admin, so can_mod should be always true
  let expected_post_listing = vec![
    ("tegan".to_owned(), false, true),
    ("mybot".to_owned(), false, true),
    ("tegan".to_owned(), false, true),
  ];
  assert_eq!(expected_post_listing, tegan_listings);

  // Have john become a moderator, then the bot
  let john_mod_form = CommunityModeratorForm::new(community_id, data.john.person.id);
  CommunityActions::join(pool, &john_mod_form).await?;

  let bot_mod_form = CommunityModeratorForm::new(community_id, data.bot.person.id);
  CommunityActions::join(pool, &bot_mod_form).await?;

  let john_listings = PostQuery {
    sort: Some(PostSortType::New),
    local_user: Some(&data.john.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?
  .into_iter()
  .map(|p| (p.creator.name, p.creator_is_moderator, p.can_mod))
  .collect::<Vec<_>>();

  // John is a mod, so he can_mod the bots (and his own) posts, but not tegans.
  let expected_post_listing = vec![
    ("tegan".to_owned(), false, false),
    ("mybot".to_owned(), true, true),
    ("tegan".to_owned(), false, false),
    ("john".to_owned(), true, true),
  ];
  assert_eq!(expected_post_listing, john_listings);

  // Bot is also a mod, but was added after john, so can't mod anything
  let bot_listings = PostQuery {
    sort: Some(PostSortType::New),
    local_user: Some(&data.bot.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?
  .into_iter()
  .map(|p| (p.creator.name, p.creator_is_moderator, p.can_mod))
  .collect::<Vec<_>>();

  let expected_post_listing = vec![
    ("tegan".to_owned(), false, false),
    ("mybot".to_owned(), true, true),
    ("tegan".to_owned(), false, false),
    ("john".to_owned(), true, false),
  ];
  assert_eq!(expected_post_listing, bot_listings);

  // Make the bot leave the mod team, and make sure it can_mod is false.
  CommunityActions::leave(pool, &bot_mod_form).await?;

  let bot_listings = PostQuery {
    sort: Some(PostSortType::New),
    local_user: Some(&data.bot.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?
  .into_iter()
  .map(|p| (p.creator.name, p.creator_is_moderator, p.can_mod))
  .collect::<Vec<_>>();

  let expected_post_listing = vec![
    ("tegan".to_owned(), false, false),
    ("mybot".to_owned(), false, false),
    ("tegan".to_owned(), false, false),
    ("john".to_owned(), true, false),
  ];
  assert_eq!(expected_post_listing, bot_listings);

  // Have tegan the administrator become a moderator
  let tegan_mod_form = CommunityModeratorForm::new(community_id, data.tegan.person.id);
  CommunityActions::join(pool, &tegan_mod_form).await?;

  let john_listings = PostQuery {
    sort: Some(PostSortType::New),
    local_user: Some(&data.john.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?
  .into_iter()
  .map(|p| (p.creator.name, p.creator_is_moderator, p.can_mod))
  .collect::<Vec<_>>();

  // John is a mod, so he still can_mod the bots (and his own) posts. Tegan is a lower mod and
  // admin, john can't mod their posts.
  let expected_post_listing = vec![
    ("tegan".to_owned(), true, false),
    ("mybot".to_owned(), false, true),
    ("tegan".to_owned(), true, false),
    ("john".to_owned(), true, true),
  ];
  assert_eq!(expected_post_listing, john_listings);

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_person_language(data: &mut Data) -> LemmyResult<()> {
  const EL_POSTO: &str = "el posto";

  let pool = &data.pool();
  let pool = &mut pool.into();

  let spanish_id = Language::read_id_from_code(pool, "es").await?;

  let french_id = Language::read_id_from_code(pool, "fr").await?;

  let post_spanish = PostInsertForm {
    language_id: Some(spanish_id),
    ..PostInsertForm::new(
      EL_POSTO.to_string(),
      data.tegan.person.id,
      data.community.id,
    )
  };
  Post::create(pool, &post_spanish).await?;

  let post_listings_all = data.default_post_query().list(&data.site, pool).await?;

  // no language filters specified, all posts should be returned
  assert_eq!(
    vec![EL_POSTO, POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_all)
  );

  LocalUserLanguage::update(pool, vec![french_id], data.tegan.local_user.id).await?;

  let post_listing_french = data.default_post_query().list(&data.site, pool).await?;

  // only one post in french and one undetermined should be returned
  assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listing_french));
  assert_eq!(
    Some(french_id),
    post_listing_french.get(1).map(|p| p.post.language_id)
  );

  LocalUserLanguage::update(
    pool,
    vec![french_id, UNDETERMINED_ID],
    data.tegan.local_user.id,
  )
  .await?;
  let post_listings_french_und = data
    .default_post_query()
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|p| (p.post.name, p.post.language_id))
    .collect::<Vec<_>>();
  let expected_post_listings_french_und = vec![
    (POST_WITH_TAGS.to_owned(), french_id),
    (POST_BY_BOT.to_owned(), UNDETERMINED_ID),
    (POST.to_owned(), french_id),
  ];

  // french post and undetermined language post should be returned
  assert_eq!(expected_post_listings_french_und, post_listings_french_und);

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_removed(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Remove the post
  Post::update(
    pool,
    data.bot_post.id,
    &PostUpdateForm {
      removed: Some(true),
      ..Default::default()
    },
  )
  .await?;

  // Make sure you don't see the removed post in the results
  data.tegan.local_user.admin = false;
  let post_listings_no_admin = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listings_no_admin));

  // Removed bot post is shown to admins
  data.tegan.local_user.admin = true;
  let post_listings_is_admin = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_is_admin)
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_deleted(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Delete the post
  Post::update(
    pool,
    data.post.id,
    &PostUpdateForm {
      deleted: Some(true),
      ..Default::default()
    },
  )
  .await?;

  // Deleted post is only shown to creator
  for (local_user, expect_contains_deleted) in [
    (None, false),
    (Some(&data.john.local_user), false),
    (Some(&data.tegan.local_user), true),
  ] {
    let contains_deleted = PostQuery {
      local_user,
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?
    .iter()
    .any(|p| p.post.id == data.post.id);

    assert_eq!(expect_contains_deleted, contains_deleted);
  }

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_hidden_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  Community::update(
    pool,
    data.community.id,
    &CommunityUpdateForm {
      visibility: Some(CommunityVisibility::Unlisted),
      ..Default::default()
    },
  )
  .await?;

  let posts = PostQuery::default().list(&data.site, pool).await?;
  assert!(posts.is_empty());

  let posts = data.default_post_query().list(&data.site, pool).await?;
  assert!(posts.is_empty());

  // Follow the community
  let form = CommunityFollowerForm::new(
    data.community.id,
    data.tegan.person.id,
    CommunityFollowerState::Accepted,
  );
  CommunityActions::follow(pool, &form).await?;

  let posts = data.default_post_query().list(&data.site, pool).await?;
  assert!(!posts.is_empty());

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_instance_block_communities(data: &mut Data) -> LemmyResult<()> {
  const POST_FROM_BLOCKED_INSTANCE_COMMS: &str = "post on blocked instance";
  const HOWARD_POST: &str = "howard post";
  const POST_LISTING_WITH_BLOCKED: [&str; 5] = [
    HOWARD_POST,
    POST_FROM_BLOCKED_INSTANCE_COMMS,
    POST_WITH_TAGS,
    POST_BY_BOT,
    POST,
  ];

  let pool = &data.pool();
  let pool = &mut pool.into();

  let blocked_instance_comms = Instance::read_or_create(pool, "another_domain.tld").await?;

  let community_form = CommunityInsertForm::new(
    blocked_instance_comms.id,
    "test_community_4".to_string(),
    "none".to_owned(),
    "pubkey".to_string(),
  );
  let inserted_community = Community::create(pool, &community_form).await?;

  let post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(
      POST_FROM_BLOCKED_INSTANCE_COMMS.to_string(),
      data.bot.person.id,
      inserted_community.id,
    )
  };
  let post_from_blocked_instance = Post::create(pool, &post_form).await?;

  // Create a person on that comm-blocked instance,
  // have them create a post from a non-instance-comm blocked community.
  // Make sure others can see it.
  let howard_form = PersonInsertForm::test_form(blocked_instance_comms.id, "howard");
  let howard = Person::create(pool, &howard_form).await?;
  let howard_post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(HOWARD_POST.to_string(), howard.id, data.community.id)
  };
  let _post_from_blocked_instance_user = Post::create(pool, &howard_post_form).await?;

  // no instance block, should return all posts
  let post_listings_all = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_all));

  // block the instance communities
  let block_form =
    InstanceCommunitiesBlockForm::new(data.tegan.person.id, blocked_instance_comms.id);
  InstanceActions::block_communities(pool, &block_form).await?;

  // now posts from communities on that instance should be hidden
  let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(
    vec![HOWARD_POST, POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_blocked)
  );
  assert!(
    post_listings_blocked
      .iter()
      .all(|p| p.post.id != post_from_blocked_instance.id)
  );

  // Follow community from the blocked instance to see posts anyway
  let follow_form = CommunityFollowerForm::new(
    inserted_community.id,
    data.tegan.person.id,
    CommunityFollowerState::Accepted,
  );
  CommunityActions::follow(pool, &follow_form).await?;
  let post_listings_bypass = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_bypass));
  CommunityActions::unfollow(pool, data.tegan.person.id, inserted_community.id).await?;

  // after unblocking it should return all posts again
  InstanceActions::unblock_communities(pool, &block_form).await?;
  let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_blocked));

  Instance::delete(pool, blocked_instance_comms.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_instance_block_persons(data: &mut Data) -> LemmyResult<()> {
  const POST_FROM_BLOCKED_INSTANCE_USERS: &str = "post from blocked instance user";
  const POST_TO_UNBLOCKED_COMM: &str = "post to unblocked comm";
  const POST_LISTING_WITH_BLOCKED: [&str; 5] = [
    POST_TO_UNBLOCKED_COMM,
    POST_FROM_BLOCKED_INSTANCE_USERS,
    POST_WITH_TAGS,
    POST_BY_BOT,
    POST,
  ];

  let pool = &data.pool();
  let pool = &mut pool.into();

  let blocked_instance_persons = Instance::read_or_create(pool, "another_domain.tld").await?;

  let howard_form = PersonInsertForm::test_form(blocked_instance_persons.id, "howard");
  let howard = Person::create(pool, &howard_form).await?;

  let community_form = CommunityInsertForm::new(
    blocked_instance_persons.id,
    "test_community_8".to_string(),
    "none".to_owned(),
    "pubkey".to_string(),
  );
  let inserted_community = Community::create(pool, &community_form).await?;

  // Create a post from the blocked user on a safe community
  let blocked_post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(
      POST_FROM_BLOCKED_INSTANCE_USERS.to_string(),
      howard.id,
      data.community.id,
    )
  };
  let post_from_blocked_instance = Post::create(pool, &blocked_post_form).await?;

  // Also create a post from an unblocked user
  let unblocked_post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(
      POST_TO_UNBLOCKED_COMM.to_string(),
      data.bot.person.id,
      inserted_community.id,
    )
  };
  let _post_to_unblocked_comm = Post::create(pool, &unblocked_post_form).await?;

  // no instance block, should return all posts
  let post_listings_all = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_all));

  // block the instance communities
  let block_form = InstancePersonsBlockForm::new(data.tegan.person.id, blocked_instance_persons.id);
  InstanceActions::block_persons(pool, &block_form).await?;

  // now posts from users on that instance should be hidden
  let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(
    vec![POST_TO_UNBLOCKED_COMM, POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_blocked)
  );
  assert!(
    post_listings_blocked
      .iter()
      .all(|p| p.post.id != post_from_blocked_instance.id)
  );

  // after unblocking it should return all posts again
  InstanceActions::unblock_persons(pool, &block_form).await?;
  let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_blocked));

  Instance::delete(pool, blocked_instance_persons.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn pagination_includes_each_post_once(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let community_form = CommunityInsertForm::new(
    data.instance.id,
    "yes".to_string(),
    "yes".to_owned(),
    "pubkey".to_string(),
  );
  let inserted_community = Community::create(pool, &community_form).await?;

  let mut inserted_post_ids = HashSet::new();

  // Create 150 posts with varying non-correlating values for publish date, number of comments,
  // and featured
  for i in 0..45 {
    let post_form = PostInsertForm {
      featured_local: Some((i % 2) == 0),
      featured_community: Some((i % 2) == 0),
      published_at: Some(Utc::now() - Duration::from_secs(i)),
      ..PostInsertForm::new(
        "keep Christ in Christmas".to_owned(),
        data.tegan.person.id,
        inserted_community.id,
      )
    };
    let inserted_post = Post::create(pool, &post_form).await?;
    inserted_post_ids.insert(inserted_post.id);
  }

  let options = PostQuery {
    community_id: Some(inserted_community.id),
    sort: Some(PostSortType::Hot),
    limit: Some(3),
    ..Default::default()
  };

  let mut listed_post_ids_forward = vec![];
  let mut page_cursor = None;
  let mut page_cursor_back = None;
  loop {
    let post_listings = PostQuery {
      page_cursor,
      ..options.clone()
    }
    .list(&data.site, pool)
    .await?;

    listed_post_ids_forward.extend(post_listings.iter().map(|p| p.post.id));

    if post_listings.next_page.is_none() {
      break;
    }
    page_cursor = post_listings.next_page;
    page_cursor_back = post_listings.prev_page;
  }

  // unsorted comparison with hashset
  assert_eq!(
    inserted_post_ids,
    listed_post_ids_forward.iter().cloned().collect()
  );

  // By going backwards from the last page we dont see the last page again, so remove those items
  listed_post_ids_forward.truncate(listed_post_ids_forward.len() - 3);

  // Check that backward pagination matches forward pagination
  loop {
    let post_listings = PostQuery {
      page_cursor: page_cursor_back,
      ..options.clone()
    }
    .list(&data.site, pool)
    .await?;

    let listed_post_ids = post_listings.iter().map(|p| p.post.id).collect::<Vec<_>>();

    let index = listed_post_ids_forward.len() - listed_post_ids.len();

    assert_eq!(
      listed_post_ids_forward.get(index..),
      listed_post_ids.get(..)
    );
    listed_post_ids_forward.truncate(index);

    if let Some(cursor) = post_listings.prev_page {
      page_cursor_back = Some(cursor);
    } else {
      break;
    }
  }

  Community::delete(pool, inserted_community.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
/// Test that last and first partial pages only have one cursor.
async fn pagination_hidden_cursors(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let community_form = CommunityInsertForm::new(
    data.instance.id,
    "yes".to_string(),
    "yes".to_owned(),
    "pubkey".to_string(),
  );
  let inserted_community = Community::create(pool, &community_form).await?;

  let page_size: usize = 5;

  // Create 2 pages with 5 and 4 posts respectively
  for i in 0..9 {
    let post_form = PostInsertForm {
      featured_local: Some((i % 2) == 0),
      featured_community: Some((i % 2) == 0),
      published_at: Some(Utc::now() - Duration::from_secs(i)),
      ..PostInsertForm::new(
        "keep Christ in Christmas".to_owned(),
        data.tegan.person.id,
        inserted_community.id,
      )
    };
    Post::create(pool, &post_form).await?;
  }

  let options = PostQuery {
    community_id: Some(inserted_community.id),
    sort: Some(PostSortType::Hot),
    limit: Some(page_size.try_into()?),
    ..Default::default()
  };

  let mut get_page = async |cursor: &Option<PaginationCursor>| {
    PostQuery {
      page_cursor: cursor.clone(),
      ..options.clone()
    }
    .list(&data.site, pool)
    .await
  };

  let first_page = get_page(&None).await?;
  assert_eq!(first_page.items.len(), page_size);
  assert!(first_page.prev_page.is_none()); // without request cursor, no back cursor
  assert!(first_page.next_page.is_some());

  let last_page = get_page(&first_page.next_page).await?;
  assert_eq!(last_page.items.len(), page_size - 1);
  assert!(last_page.prev_page.is_some());
  assert!(last_page.next_page.is_none());

  // Get first page with both cursors
  let first_page2 = get_page(&last_page.prev_page).await?;
  assert_eq!(first_page2.items.len(), page_size);
  assert!(first_page2.prev_page.is_some());
  assert_eq!(first_page2.next_page, first_page.next_page);

  let pool = &data.pool;
  let pool = &mut pool.into();

  // Mark first post as deleted
  let first_post_view = first_page.items.first().expect("first post");
  let post_update_form = PostUpdateForm {
    deleted: Some(true),
    ..Default::default()
  };
  Post::update(pool, first_post_view.post.id, &post_update_form).await?;

  let partial_first_page = get_page(&last_page.prev_page).await?;
  assert_eq!(partial_first_page.items.len(), page_size - 1);
  assert!(partial_first_page.prev_page.is_none());
  assert!(partial_first_page.next_page.is_some());

  // Cursor works for item marked as deleted
  let removed_item_page = get_page(&first_page2.prev_page).await?;
  assert_eq!(removed_item_page.items.len(), 0);
  assert!(removed_item_page.prev_page.is_none());
  assert!(removed_item_page.next_page.is_some()); // recovery cursor

  let recovered_page = get_page(&removed_item_page.next_page).await?;
  assert_eq!(recovered_page.items.len(), page_size);
  assert!(recovered_page.prev_page.is_some());
  assert!(recovered_page.next_page.is_some());

  // Delete first post from the database
  Post::delete(pool, first_post_view.post.id).await?;

  let partial_first_page = get_page(&last_page.prev_page).await?;
  assert_eq!(partial_first_page.items.len(), page_size - 1);
  assert!(partial_first_page.prev_page.is_none());
  assert!(partial_first_page.next_page.is_some());

  // Cursor doesn't work for item that no longer exists
  let removed_item_page = get_page(&first_page2.prev_page).await;
  if let Err(LemmyError {
    error_type,
    inner: _,
    caller: _,
  }) = removed_item_page
  {
    assert_eq!(error_type, LemmyErrorType::NotFound);
  } else {
    unreachable!();
  }

  Community::delete(pool, inserted_community.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
/// Test paging past the last and first page.
async fn pagination_recovery_cursors(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let community_form = CommunityInsertForm::new(
    data.instance.id,
    "yes".to_string(),
    "yes".to_owned(),
    "pubkey".to_string(),
  );
  let inserted_community = Community::create(pool, &community_form).await?;

  let page_size: usize = 5;

  // Create 2 pages with 5 posts each
  for i in 0..10 {
    let post_form = PostInsertForm {
      featured_local: Some((i % 2) == 0),
      featured_community: Some((i % 2) == 0),
      published_at: Some(Utc::now() - Duration::from_secs(i)),
      ..PostInsertForm::new(
        "keep Christ in Christmas".to_owned(),
        data.tegan.person.id,
        inserted_community.id,
      )
    };
    Post::create(pool, &post_form).await?;
  }

  let options = PostQuery {
    community_id: Some(inserted_community.id),
    sort: Some(PostSortType::Hot),
    limit: Some(page_size.try_into()?),
    ..Default::default()
  };

  let mut get_page = async |cursor: &Option<PaginationCursor>| {
    PostQuery {
      page_cursor: cursor.clone(),
      ..options.clone()
    }
    .list(&data.site, pool)
    .await
  };

  let first_page = get_page(&None).await?;
  assert_eq!(first_page.items.len(), page_size);
  assert!(first_page.prev_page.is_none()); // without request cursor, no back cursor
  assert!(first_page.next_page.is_some());

  let last_page = get_page(&first_page.next_page).await?;
  assert_eq!(last_page.items.len(), page_size);
  assert!(last_page.prev_page.is_some());
  assert!(last_page.next_page.is_some()); // full page, has cursor

  // Get the first page with both cursors
  let first_page2 = get_page(&last_page.prev_page).await?;
  assert_eq!(first_page.items.len(), page_size);
  assert!(first_page2.prev_page.is_some()); // full page, has cursor
  assert!(first_page2.next_page.is_some());
  assert_eq!(first_page2.next_page, first_page.next_page);
  assert_eq!(
    first_page2
      .items
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>(),
    first_page
      .items
      .clone()
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>()
  );

  let beyond_first_page = get_page(&first_page2.prev_page).await?;
  assert_eq!(beyond_first_page.items.len(), 0);
  assert!(beyond_first_page.prev_page.is_none());
  assert!(beyond_first_page.next_page.is_some());

  let recovered_first_page = get_page(&beyond_first_page.next_page).await?;
  assert_eq!(recovered_first_page.items.len(), page_size);
  assert!(recovered_first_page.prev_page.is_some()); // full page, has cursor
  assert!(recovered_first_page.next_page.is_some());
  assert_eq!(recovered_first_page.next_page, first_page2.next_page);
  assert_eq!(recovered_first_page.prev_page, first_page2.prev_page);
  assert_eq!(
    recovered_first_page
      .items
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>(),
    first_page
      .items
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>()
  );

  let beyond_last_page = get_page(&last_page.next_page).await?;
  assert_eq!(beyond_last_page.items.len(), 0);
  assert!(beyond_last_page.prev_page.is_some());
  assert!(beyond_last_page.next_page.is_none());

  let recovered_last_page = get_page(&beyond_last_page.prev_page).await?;
  assert_eq!(recovered_last_page.items.len(), page_size);
  assert!(recovered_last_page.prev_page.is_some());
  assert!(recovered_last_page.next_page.is_some()); // full page, has cursor
  assert_eq!(recovered_last_page.next_page, last_page.next_page);
  assert_eq!(recovered_last_page.prev_page, last_page.prev_page);
  assert_eq!(
    recovered_last_page
      .items
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>(),
    last_page
      .items
      .into_iter()
      .map(|pv| pv.post.id)
      .collect::<Vec<PostId>>()
  );

  Community::delete(pool, inserted_community.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_hide_read(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Make sure local user hides read posts
  let local_user_form = LocalUserUpdateForm {
    show_read_posts: Some(false),
    ..Default::default()
  };
  LocalUser::update(pool, data.tegan.local_user.id, &local_user_form).await?;
  data.tegan.local_user.show_read_posts = false;

  // Mark a post as read
  PostActions::mark_as_read(pool, data.tegan.person.id, &[data.bot_post.id]).await?;

  // Make sure you don't see the read post in the results
  let post_listings_hide_read = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listings_hide_read));

  // Test with the show_read override as true
  let post_listings_show_read_true = PostQuery {
    show_read: Some(true),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_show_read_true)
  );

  // Test with the show_read override as false
  let post_listings_show_read_false = PostQuery {
    show_read: Some(false),
    ..data.default_post_query()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST],
    names(&post_listings_show_read_false)
  );
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_hide_hidden(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Mark a post as hidden
  let hide_form = PostHideForm::new(data.bot_post.id, data.tegan.person.id);
  PostActions::hide(pool, &hide_form).await?;

  // Make sure you don't see the hidden post in the results
  let post_listings_hide_hidden = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST],
    names(&post_listings_hide_hidden)
  );

  // Make sure it does come back with the show_hidden option
  let post_listings_show_hidden = PostQuery {
    sort: Some(PostSortType::New),
    local_user: Some(&data.tegan.local_user),
    show_hidden: Some(true),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_show_hidden)
  );

  // Make sure that hidden field is true.
  assert!(&post_listings_show_hidden.get(1).is_some_and(|p| {
    p.post_actions
      .as_ref()
      .is_some_and(|a| a.hidden_at.is_some())
  }));

  // Make sure only that one comes back for list_hidden
  let list_hidden = PostView::list_hidden(pool, &data.tegan.person, None, None, None).await?;
  assert_eq!(vec![POST_BY_BOT], names(&list_hidden));

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_hide_nsfw(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Mark a post as nsfw
  let update_form = PostUpdateForm {
    nsfw: Some(true),
    ..Default::default()
  };

  Post::update(pool, data.post_with_tags.id, &update_form).await?;

  // Make sure you don't see the nsfw post in the regular results
  let post_listings_hide_nsfw = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(vec![POST_BY_BOT, POST], names(&post_listings_hide_nsfw));

  // Make sure it does come back with the show_nsfw option
  let post_listings_show_nsfw = PostQuery {
    sort: Some(PostSortType::New),
    show_nsfw: Some(true),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT, POST],
    names(&post_listings_show_nsfw)
  );

  // Make sure that nsfw field is true.
  assert!(
    &post_listings_show_nsfw
      .first()
      .ok_or(LemmyErrorType::NotFound)?
      .post
      .nsfw
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn local_only_instance(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  Community::update(
    pool,
    data.community.id,
    &CommunityUpdateForm {
      visibility: Some(CommunityVisibility::LocalOnlyPrivate),
      ..Default::default()
    },
  )
  .await?;

  let unauthenticated_query = PostQuery {
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(0, unauthenticated_query.len());

  let authenticated_query = PostQuery {
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(3, authenticated_query.len());

  let unauthenticated_post =
    PostView::read(pool, data.post.id, None, data.instance.id, false).await;
  assert!(unauthenticated_post.is_err());

  let authenticated_post = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await;
  assert!(authenticated_post.is_ok());

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_local_user_banned_from_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Test that post view shows if local user is blocked from community
  let banned_from_comm_person = PersonInsertForm::test_form(data.instance.id, "jill");

  let inserted_banned_from_comm_person = Person::create(pool, &banned_from_comm_person).await?;

  let inserted_banned_from_comm_local_user = LocalUser::create(
    pool,
    &LocalUserInsertForm::test_form(inserted_banned_from_comm_person.id),
    vec![],
  )
  .await?;

  CommunityActions::ban(
    pool,
    &CommunityPersonBanForm::new(data.community.id, inserted_banned_from_comm_person.id),
  )
  .await?;

  let post_view = PostView::read(
    pool,
    data.post.id,
    Some(&inserted_banned_from_comm_local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert!(
    post_view
      .community_actions
      .is_some_and(|x| x.received_ban_at.is_some())
  );

  Person::delete(pool, inserted_banned_from_comm_person.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_local_user_not_banned_from_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let post_view = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert!(post_view.community_actions.is_none());

  Ok(())
}

/// Use microseconds for date checks
///
/// Necessary because postgres uses micros, but rust uses nanos
fn micros(dt: DateTime<Utc>) -> i64 {
  dt.timestamp_micros()
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_creator_banned(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let banned_person_form = PersonInsertForm::test_form(data.instance.id, "jill");

  let banned_person = Person::create(pool, &banned_person_form).await?;

  let post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(
      "banned person post".to_string(),
      banned_person.id,
      data.community.id,
    )
  };
  let banned_post = Post::create(pool, &post_form).await?;

  let expires_at = Utc::now().checked_add_days(Days::new(1));

  InstanceActions::ban(
    pool,
    &InstanceBanForm::new(banned_person.id, data.instance.id, expires_at),
  )
  .await?;

  // Let john read their post
  let post_view = PostView::read(
    pool,
    banned_post.id,
    Some(&data.john.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert!(post_view.creator_banned);

  // Make sure the expires at is correct
  assert_eq!(
    expires_at.map(micros),
    post_view.creator_ban_expires_at.map(micros)
  );

  Person::delete(pool, banned_person.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_creator_community_banned(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let banned_person_form = PersonInsertForm::test_form(data.instance.id, "jarvis");

  let banned_person = Person::create(pool, &banned_person_form).await?;

  let post_form = PostInsertForm {
    language_id: Some(LanguageId(1)),
    ..PostInsertForm::new(
      "banned jarvis post".to_string(),
      banned_person.id,
      data.community.id,
    )
  };
  let banned_post = Post::create(pool, &post_form).await?;

  let expires_at = Utc::now().checked_add_days(Days::new(1));

  CommunityActions::ban(
    pool,
    &CommunityPersonBanForm {
      ban_expires_at: Some(expires_at),
      ..CommunityPersonBanForm::new(data.community.id, banned_person.id)
    },
  )
  .await?;

  // Let john read their post
  let post_view = PostView::read(
    pool,
    banned_post.id,
    Some(&data.john.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert!(post_view.creator_banned_from_community);
  assert!(!post_view.creator_banned);

  // Make sure the expires at is correct
  assert_eq!(
    expires_at.map(micros),
    post_view.creator_community_ban_expires_at.map(micros)
  );

  Person::delete(pool, banned_person.id).await?;
  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn speed_check(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Make sure the post_view query is less than this time
  let duration_max = Duration::from_millis(120);

  // Create some dummy posts
  let num_posts = 1000;
  for x in 1..num_posts {
    let name = format!("post_{x}");
    let url = Some(Url::parse(&format!("https://google.com/{name}"))?.into());

    let post_form = PostInsertForm {
      url,
      ..PostInsertForm::new(name, data.tegan.person.id, data.community.id)
    };
    Post::create(pool, &post_form).await?;
  }

  // Manually trigger and wait for a statistics update to ensure consistent and high amount of
  // accuracy in the statistics used for query planning
  println!("🧮 updating database statistics");
  let conn = &mut get_conn(pool).await?;
  conn.batch_execute("ANALYZE;").await?;

  // Time how fast the query took
  let now = Instant::now();
  PostQuery {
    sort: Some(PostSortType::Active),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;

  let elapsed = now.elapsed();
  println!("Elapsed: {:.0?}", elapsed);

  assert!(
    elapsed.lt(&duration_max),
    "Query took {:.0?}, longer than the max of {:.0?}",
    elapsed,
    duration_max
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_no_comments_only(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Create a comment for a post
  let comment_form =
    CommentInsertForm::new(data.tegan.person.id, data.post.id, "a comment".to_owned());
  Comment::create(pool, &comment_form, None).await?;

  // Make sure it doesnt come back with the no_comments option
  let post_listings_no_comments = PostQuery {
    sort: Some(PostSortType::New),
    no_comments_only: Some(true),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;

  assert_eq!(
    vec![POST_WITH_TAGS, POST_BY_BOT],
    names(&post_listings_no_comments)
  );

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_private_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Mark community as private
  Community::update(
    pool,
    data.community.id,
    &CommunityUpdateForm {
      visibility: Some(CommunityVisibility::Private),
      ..Default::default()
    },
  )
  .await?;

  // No posts returned without auth
  let read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(0, read_post_listing.len());
  let post_view = PostView::read(pool, data.post.id, None, data.instance.id, false).await;
  assert!(post_view.is_err());

  // No posts returned for non-follower who is not admin
  data.tegan.local_user.admin = false;
  let read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(0, read_post_listing.len());
  let post_view = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await;
  assert!(post_view.is_err());

  // Admin can view content without following
  data.tegan.local_user.admin = true;
  let read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(3, read_post_listing.len());
  let post_view = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    true,
  )
  .await;
  assert!(post_view.is_ok());
  data.tegan.local_user.admin = false;

  // User can view after following
  let follow_form = CommunityFollowerForm::new(
    data.community.id,
    data.tegan.person.id,
    CommunityFollowerState::Accepted,
  );
  CommunityActions::follow(pool, &follow_form).await?;

  let read_post_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(3, read_post_listing.len());
  let post_view = PostView::read(
    pool,
    data.post.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    true,
  )
  .await;
  assert!(post_view.is_ok());

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listings_hide_media(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // Make one post an image post
  Post::update(
    pool,
    data.bot_post.id,
    &PostUpdateForm {
      url_content_type: Some(Some(String::from("image/png"))),
      ..Default::default()
    },
  )
  .await?;

  // Make sure all the posts are returned when `hide_media` is unset
  let hide_media_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(3, hide_media_listing.len());

  // Ensure the `hide_media` user setting is set
  let local_user_form = LocalUserUpdateForm {
    hide_media: Some(true),
    ..Default::default()
  };
  LocalUser::update(pool, data.tegan.local_user.id, &local_user_form).await?;
  data.tegan.local_user.hide_media = true;

  // Ensure you don't see the image post
  let hide_media_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(2, hide_media_listing.len());

  // Make sure the `hide_media` override works
  let hide_media_listing = PostQuery {
    community_id: Some(data.community.id),
    local_user: Some(&data.tegan.local_user),
    hide_media: Some(false),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(3, hide_media_listing.len());

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_with_blocked_keywords(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let name_blocked = format!("post_{POST_KEYWORD_BLOCKED}");
  let name_blocked2 = format!("post2_{POST_KEYWORD_BLOCKED}2");
  let url = Some(Url::parse(&format!("https://google.com/{POST_KEYWORD_BLOCKED}"))?.into());
  let body = format!("post body with {POST_KEYWORD_BLOCKED}");
  let name_not_blocked = "post_with_name_not_blocked".to_string();
  let name_not_blocked2 = "post_with_name_not_blocked2".to_string();

  let post_name_blocked = PostInsertForm::new(
    name_blocked.clone(),
    data.tegan.person.id,
    data.community.id,
  );

  let post_body_blocked = PostInsertForm {
    body: Some(body),
    ..PostInsertForm::new(
      name_not_blocked.clone(),
      data.tegan.person.id,
      data.community.id,
    )
  };

  let post_url_blocked = PostInsertForm {
    url,
    ..PostInsertForm::new(
      name_not_blocked2.clone(),
      data.tegan.person.id,
      data.community.id,
    )
  };

  let post_name_blocked_but_not_body_and_url = PostInsertForm {
    body: Some("Some body".to_string()),
    url: Some(Url::parse("https://google.com")?.into()),
    ..PostInsertForm::new(
      name_blocked2.clone(),
      data.tegan.person.id,
      data.community.id,
    )
  };
  Post::create(pool, &post_name_blocked).await?;
  Post::create(pool, &post_body_blocked).await?;
  Post::create(pool, &post_url_blocked).await?;
  Post::create(pool, &post_name_blocked_but_not_body_and_url).await?;

  let keyword_blocks = Some(LocalUserKeywordBlock::read(pool, data.tegan.local_user.id).await?);

  let post_listings = PostQuery {
    local_user: Some(&data.tegan.local_user),
    keyword_blocks,
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;

  // Should not have any of the posts
  assert!(!names(&post_listings).contains(&name_blocked.as_str()));
  assert!(!names(&post_listings).contains(&name_blocked2.as_str()));
  assert!(!names(&post_listings).contains(&name_not_blocked.as_str()));
  assert!(!names(&post_listings).contains(&name_not_blocked2.as_str()));

  // Should contain not blocked posts
  assert!(names(&post_listings).contains(&POST_BY_BOT));
  assert!(names(&post_listings).contains(&POST));
  Ok(())
}
#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_tags_present(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  let post_view = PostView::read(
    pool,
    data.post_with_tags.id,
    Some(&data.tegan.local_user),
    data.instance.id,
    false,
  )
  .await?;

  assert_eq!(2, post_view.tags.0.len());
  assert_eq!(data.tag_1.name, post_view.tags.0[0].name);
  assert_eq!(data.tag_2.name, post_view.tags.0[1].name);
  assert_eq!(data.tag_1.color, post_view.tags.0[0].color);
  assert_eq!(data.tag_2.color, post_view.tags.0[1].color);

  let all_posts = data.default_post_query().list(&data.site, pool).await?;
  assert_eq!(2, all_posts[0].tags.0.len()); // post with tags
  assert_eq!(0, all_posts[1].tags.0.len()); // bot post
  assert_eq!(0, all_posts[2].tags.0.len()); // normal post

  Ok(())
}

#[test_context(Data)]
#[tokio::test]
#[serial]
async fn post_listing_multi_community(data: &mut Data) -> LemmyResult<()> {
  let pool = &data.pool();
  let pool = &mut pool.into();

  // create two more communities with one post each
  let form = CommunityInsertForm::new(
    data.instance.id,
    "test_community_4".to_string(),
    "nada".to_owned(),
    "pubkey".to_string(),
  );
  let community_1 = Community::create(pool, &form).await?;

  let form = PostInsertForm::new(POST.to_string(), data.tegan.person.id, community_1.id);
  let post_1 = Post::create(pool, &form).await?;

  let form = CommunityInsertForm::new(
    data.instance.id,
    "test_community_5".to_string(),
    "nada".to_owned(),
    "pubkey".to_string(),
  );
  let community_2 = Community::create(pool, &form).await?;

  let form = PostInsertForm::new(POST.to_string(), data.tegan.person.id, community_2.id);
  let post_2 = Post::create(pool, &form).await?;

  let form = MultiCommunityInsertForm::new(
    data.tegan.person.id,
    data.tegan.person.instance_id,
    "test multi".to_string(),
    String::new(),
  );
  let multi = MultiCommunity::create(pool, &form).await?;
  MultiCommunity::update_entries(pool, multi.id, &vec![community_1.id, community_2.id]).await?;

  let listing = PostQuery {
    multi_community_id: Some(multi.id),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;

  let listing_communities = listing
    .iter()
    .map(|l| l.community.id)
    .collect::<HashSet<_>>();
  assert_eq!(
    HashSet::from([community_1.id, community_2.id]),
    listing_communities
  );

  let listing_posts = listing.iter().map(|l| l.post.id).collect::<HashSet<_>>();
  assert_eq!(HashSet::from([post_1.id, post_2.id]), listing_posts);

  let suggested = PostQuery {
    listing_type: Some(ListingType::Suggested),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert!(suggested.is_empty());

  let form = LocalSiteUpdateForm {
    suggested_communities: Some(multi.id),
    ..Default::default()
  };
  LocalSite::update(pool, &form).await?;

  let suggested = PostQuery {
    listing_type: Some(ListingType::Suggested),
    ..Default::default()
  }
  .list(&data.site, pool)
  .await?;
  assert_eq!(listing.items, suggested.items);

  Ok(())
}
