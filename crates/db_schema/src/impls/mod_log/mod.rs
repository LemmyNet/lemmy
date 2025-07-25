pub mod admin;
pub mod moderator;

#[cfg(test)]
mod tests {

  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      mod_log::{admin::*, moderator::*},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_mod = PersonInsertForm::test_form(inserted_instance.id, "the mod");

    let inserted_mod = Person::create(pool, &new_mod).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "jim2");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "mod_community".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post thweep".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    // Now the actual tests

    // remove post
    let mod_remove_post_form = ModRemovePostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      reason: None,
      removed: None,
    };
    let inserted_mod_remove_post = ModRemovePost::create(pool, &mod_remove_post_form).await?;
    let read_mod_remove_post = ModRemovePost::read(pool, inserted_mod_remove_post.id).await?;
    let expected_mod_remove_post = ModRemovePost {
      id: inserted_mod_remove_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      published_at: inserted_mod_remove_post.published_at,
    };

    // lock post

    let mod_lock_post_form = ModLockPostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      locked: None,
      reason: None,
    };
    let inserted_mod_lock_post = ModLockPost::create(pool, &mod_lock_post_form).await?;
    let read_mod_lock_post = ModLockPost::read(pool, inserted_mod_lock_post.id).await?;
    let expected_mod_lock_post = ModLockPost {
      id: inserted_mod_lock_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      locked: true,
      reason: None,
      published_at: inserted_mod_lock_post.published_at,
    };

    // feature post

    let mod_feature_post_form = ModFeaturePostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      featured: Some(false),
      is_featured_community: Some(true),
    };
    let inserted_mod_feature_post = ModFeaturePost::create(pool, &mod_feature_post_form).await?;
    let read_mod_feature_post = ModFeaturePost::read(pool, inserted_mod_feature_post.id).await?;
    let expected_mod_feature_post = ModFeaturePost {
      id: inserted_mod_feature_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      featured: false,
      is_featured_community: true,
      published_at: inserted_mod_feature_post.published_at,
    };

    // comment

    let mod_remove_comment_form = ModRemoveCommentForm {
      mod_person_id: inserted_mod.id,
      comment_id: inserted_comment.id,
      reason: None,
      removed: None,
    };
    let inserted_mod_remove_comment =
      ModRemoveComment::create(pool, &mod_remove_comment_form).await?;
    let read_mod_remove_comment =
      ModRemoveComment::read(pool, inserted_mod_remove_comment.id).await?;
    let expected_mod_remove_comment = ModRemoveComment {
      id: inserted_mod_remove_comment.id,
      comment_id: inserted_comment.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      published_at: inserted_mod_remove_comment.published_at,
    };

    // community

    let admin_remove_community_form = AdminRemoveCommunityForm {
      mod_person_id: inserted_mod.id,
      community_id: inserted_community.id,
      reason: None,
      removed: None,
    };
    let inserted_admin_remove_community =
      AdminRemoveCommunity::create(pool, &admin_remove_community_form).await?;
    let read_mod_remove_community =
      AdminRemoveCommunity::read(pool, inserted_admin_remove_community.id).await?;
    let expected_admin_remove_community = AdminRemoveCommunity {
      id: inserted_admin_remove_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      published_at: inserted_admin_remove_community.published_at,
    };

    // ban from community

    let mod_ban_from_community_form = ModBanFromCommunityForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      community_id: inserted_community.id,
      reason: None,
      banned: None,
      expires_at: None,
    };
    let inserted_mod_ban_from_community =
      ModBanFromCommunity::create(pool, &mod_ban_from_community_form).await?;
    let read_mod_ban_from_community =
      ModBanFromCommunity::read(pool, inserted_mod_ban_from_community.id).await?;
    let expected_mod_ban_from_community = ModBanFromCommunity {
      id: inserted_mod_ban_from_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: true,
      expires_at: None,
      published_at: inserted_mod_ban_from_community.published_at,
    };

    // ban

    let admin_ban_form = AdminBanForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: None,
      expires_at: None,
      instance_id: inserted_instance.id,
    };
    let inserted_admin_ban = AdminBan::create(pool, &admin_ban_form).await?;
    let read_mod_ban = AdminBan::read(pool, inserted_admin_ban.id).await?;
    let expected_admin_ban = AdminBan {
      id: inserted_admin_ban.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: true,
      expires_at: None,
      published_at: inserted_admin_ban.published_at,
      instance_id: inserted_instance.id,
    };

    // mod add community

    let mod_add_to_community_form = ModAddToCommunityForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      community_id: inserted_community.id,
      removed: None,
    };
    let inserted_mod_add_to_community =
      ModAddToCommunity::create(pool, &mod_add_to_community_form).await?;
    let read_mod_add_to_community =
      ModAddToCommunity::read(pool, inserted_mod_add_to_community.id).await?;
    let expected_mod_add_to_community = ModAddToCommunity {
      id: inserted_mod_add_to_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      published_at: inserted_mod_add_to_community.published_at,
    };

    // admin add

    let admin_add_form = AdminAddForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: None,
    };
    let inserted_admin_add = AdminAdd::create(pool, &admin_add_form).await?;
    let read_mod_add = AdminAdd::read(pool, inserted_admin_add.id).await?;
    let expected_admin_add = AdminAdd {
      id: inserted_admin_add.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      published_at: inserted_admin_add.published_at,
    };

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Person::delete(pool, inserted_mod.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_mod_remove_post, read_mod_remove_post);
    assert_eq!(expected_mod_lock_post, read_mod_lock_post);
    assert_eq!(expected_mod_feature_post, read_mod_feature_post);
    assert_eq!(expected_mod_remove_comment, read_mod_remove_comment);
    assert_eq!(expected_admin_remove_community, read_mod_remove_community);
    assert_eq!(expected_mod_ban_from_community, read_mod_ban_from_community);
    assert_eq!(expected_admin_ban, read_mod_ban);
    assert_eq!(expected_mod_add_to_community, read_mod_add_to_community);
    assert_eq!(expected_admin_add, read_mod_add);

    Ok(())
  }
}
