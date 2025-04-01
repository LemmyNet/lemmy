use crate::{
  newtypes::{
    ModAddCommunityId,
    ModAddId,
    ModBanFromCommunityId,
    ModBanId,
    ModChangeCommunityVisibilityId,
    ModFeaturePostId,
    ModLockPostId,
    ModRemoveCommentId,
    ModRemoveCommunityId,
    ModRemovePostId,
    ModTransferCommunityId,
  },
  schema::{
    mod_add,
    mod_add_community,
    mod_ban,
    mod_ban_from_community,
    mod_change_community_visibility,
    mod_feature_post,
    mod_lock_post,
    mod_remove_comment,
    mod_remove_community,
    mod_remove_post,
    mod_transfer_community,
  },
  source::mod_log::moderator::{
    ModAdd,
    ModAddCommunity,
    ModAddCommunityForm,
    ModAddForm,
    ModBan,
    ModBanForm,
    ModBanFromCommunity,
    ModBanFromCommunityForm,
    ModChangeCommunityVisibility,
    ModChangeCommunityVisibilityForm,
    ModFeaturePost,
    ModFeaturePostForm,
    ModLockPost,
    ModLockPostForm,
    ModRemoveComment,
    ModRemoveCommentForm,
    ModRemoveCommunity,
    ModRemoveCommunityForm,
    ModRemovePost,
    ModRemovePostForm,
    ModTransferCommunity,
    ModTransferCommunityForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for ModRemovePost {
  type InsertForm = ModRemovePostForm;
  type UpdateForm = ModRemovePostForm;
  type IdType = ModRemovePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl ModRemovePost {
  pub async fn create_multiple(
    pool: &mut DbPool<'_>,
    forms: &Vec<ModRemovePostForm>,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_post::table)
      .values(forms)
      .execute(conn)
      .await
  }
}

impl Crud for ModLockPost {
  type InsertForm = ModLockPostForm;
  type UpdateForm = ModLockPostForm;
  type IdType = ModLockPostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_lock_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_lock_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModFeaturePost {
  type InsertForm = ModFeaturePostForm;
  type UpdateForm = ModFeaturePostForm;
  type IdType = ModFeaturePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_feature_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_feature_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModRemoveComment {
  type InsertForm = ModRemoveCommentForm;
  type UpdateForm = ModRemoveCommentForm;
  type IdType = ModRemoveCommentId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_comment::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_comment::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl ModRemoveComment {
  pub async fn create_multiple(
    pool: &mut DbPool<'_>,
    forms: &Vec<ModRemoveCommentForm>,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_comment::table)
      .values(forms)
      .execute(conn)
      .await
  }
}

impl Crud for ModRemoveCommunity {
  type InsertForm = ModRemoveCommunityForm;
  type UpdateForm = ModRemoveCommunityForm;
  type IdType = ModRemoveCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModBanFromCommunity {
  type InsertForm = ModBanFromCommunityForm;
  type UpdateForm = ModBanFromCommunityForm;
  type IdType = ModBanFromCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban_from_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban_from_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModBan {
  type InsertForm = ModBanForm;
  type UpdateForm = ModBanForm;
  type IdType = ModBanId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModChangeCommunityVisibility {
  type InsertForm = ModChangeCommunityVisibilityForm;
  type UpdateForm = ModChangeCommunityVisibilityForm;
  type IdType = ModChangeCommunityVisibilityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_change_community_visibility::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_change_community_visibility::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModAddCommunity {
  type InsertForm = ModAddCommunityForm;
  type UpdateForm = ModAddCommunityForm;
  type IdType = ModAddCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModTransferCommunity {
  type InsertForm = ModTransferCommunityForm;
  type UpdateForm = ModTransferCommunityForm;
  type IdType = ModTransferCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_transfer_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_transfer_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for ModAdd {
  type InsertForm = ModAddForm;
  type UpdateForm = ModAddForm;
  type IdType = ModAddId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> Result<(), Error> {
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
      published: inserted_mod_remove_post.published,
    };

    // lock post

    let mod_lock_post_form = ModLockPostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      locked: None,
    };
    let inserted_mod_lock_post = ModLockPost::create(pool, &mod_lock_post_form).await?;
    let read_mod_lock_post = ModLockPost::read(pool, inserted_mod_lock_post.id).await?;
    let expected_mod_lock_post = ModLockPost {
      id: inserted_mod_lock_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      locked: true,
      published: inserted_mod_lock_post.published,
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
      published: inserted_mod_feature_post.published,
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
      published: inserted_mod_remove_comment.published,
    };

    // community

    let mod_remove_community_form = ModRemoveCommunityForm {
      mod_person_id: inserted_mod.id,
      community_id: inserted_community.id,
      reason: None,
      removed: None,
    };
    let inserted_mod_remove_community =
      ModRemoveCommunity::create(pool, &mod_remove_community_form).await?;
    let read_mod_remove_community =
      ModRemoveCommunity::read(pool, inserted_mod_remove_community.id).await?;
    let expected_mod_remove_community = ModRemoveCommunity {
      id: inserted_mod_remove_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      published: inserted_mod_remove_community.published,
    };

    // ban from community

    let mod_ban_from_community_form = ModBanFromCommunityForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      community_id: inserted_community.id,
      reason: None,
      banned: None,
      expires: None,
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
      expires: None,
      published: inserted_mod_ban_from_community.published,
    };

    // ban

    let mod_ban_form = ModBanForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: None,
      expires: None,
      instance_id: inserted_instance.id,
    };
    let inserted_mod_ban = ModBan::create(pool, &mod_ban_form).await?;
    let read_mod_ban = ModBan::read(pool, inserted_mod_ban.id).await?;
    let expected_mod_ban = ModBan {
      id: inserted_mod_ban.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: true,
      expires: None,
      published: inserted_mod_ban.published,
      instance_id: inserted_instance.id,
    };

    // mod add community

    let mod_add_community_form = ModAddCommunityForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      community_id: inserted_community.id,
      removed: None,
    };
    let inserted_mod_add_community = ModAddCommunity::create(pool, &mod_add_community_form).await?;
    let read_mod_add_community = ModAddCommunity::read(pool, inserted_mod_add_community.id).await?;
    let expected_mod_add_community = ModAddCommunity {
      id: inserted_mod_add_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      published: inserted_mod_add_community.published,
    };

    // mod add

    let mod_add_form = ModAddForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: None,
    };
    let inserted_mod_add = ModAdd::create(pool, &mod_add_form).await?;
    let read_mod_add = ModAdd::read(pool, inserted_mod_add.id).await?;
    let expected_mod_add = ModAdd {
      id: inserted_mod_add.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      published: inserted_mod_add.published,
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
    assert_eq!(expected_mod_remove_community, read_mod_remove_community);
    assert_eq!(expected_mod_ban_from_community, read_mod_ban_from_community);
    assert_eq!(expected_mod_ban, read_mod_ban);
    assert_eq!(expected_mod_add_community, read_mod_add_community);
    assert_eq!(expected_mod_add, read_mod_add);

    Ok(())
  }
}
