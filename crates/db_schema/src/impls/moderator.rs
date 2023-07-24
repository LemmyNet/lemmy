use crate::{
  source::moderator::{
    AdminPurgeComment,
    AdminPurgeCommentForm,
    AdminPurgeCommunity,
    AdminPurgeCommunityForm,
    AdminPurgePerson,
    AdminPurgePersonForm,
    AdminPurgePost,
    AdminPurgePostForm,
    ModAdd,
    ModAddCommunity,
    ModAddCommunityForm,
    ModAddForm,
    ModBan,
    ModBanForm,
    ModBanFromCommunity,
    ModBanFromCommunityForm,
    ModFeaturePost,
    ModFeaturePostForm,
    ModHideCommunity,
    ModHideCommunityForm,
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

#[async_trait]
impl Crud for ModRemovePost {
  type InsertForm = ModRemovePostForm;
  type UpdateForm = ModRemovePostForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModRemovePostForm) -> Result<Self, Error> {
    use crate::schema::mod_remove_post::dsl::mod_remove_post;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_post)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModRemovePostForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_remove_post::dsl::mod_remove_post;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_post.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModLockPost {
  type InsertForm = ModLockPostForm;
  type UpdateForm = ModLockPostForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModLockPostForm) -> Result<Self, Error> {
    use crate::schema::mod_lock_post::dsl::mod_lock_post;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_lock_post)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModLockPostForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_lock_post::dsl::mod_lock_post;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_lock_post.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModFeaturePost {
  type InsertForm = ModFeaturePostForm;
  type UpdateForm = ModFeaturePostForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModFeaturePostForm) -> Result<Self, Error> {
    use crate::schema::mod_feature_post::dsl::mod_feature_post;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_feature_post)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModFeaturePostForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_feature_post::dsl::mod_feature_post;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_feature_post.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModRemoveComment {
  type InsertForm = ModRemoveCommentForm;
  type UpdateForm = ModRemoveCommentForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModRemoveCommentForm) -> Result<Self, Error> {
    use crate::schema::mod_remove_comment::dsl::mod_remove_comment;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_comment)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModRemoveCommentForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_remove_comment::dsl::mod_remove_comment;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_comment.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModRemoveCommunity {
  type InsertForm = ModRemoveCommunityForm;
  type UpdateForm = ModRemoveCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModRemoveCommunityForm) -> Result<Self, Error> {
    use crate::schema::mod_remove_community::dsl::mod_remove_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModRemoveCommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_remove_community::dsl::mod_remove_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModBanFromCommunity {
  type InsertForm = ModBanFromCommunityForm;
  type UpdateForm = ModBanFromCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModBanFromCommunityForm) -> Result<Self, Error> {
    use crate::schema::mod_ban_from_community::dsl::mod_ban_from_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban_from_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModBanFromCommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_ban_from_community::dsl::mod_ban_from_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban_from_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModBan {
  type InsertForm = ModBanForm;
  type UpdateForm = ModBanForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModBanForm) -> Result<Self, Error> {
    use crate::schema::mod_ban::dsl::mod_ban;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(pool: &mut DbPool<'_>, from_id: i32, form: ModBanForm) -> Result<Self, Error> {
    use crate::schema::mod_ban::dsl::mod_ban;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModHideCommunity {
  type InsertForm = ModHideCommunityForm;
  type UpdateForm = ModHideCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModHideCommunityForm) -> Result<Self, Error> {
    use crate::schema::mod_hide_community::dsl::mod_hide_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_hide_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModHideCommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_hide_community::dsl::mod_hide_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_hide_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModAddCommunity {
  type InsertForm = ModAddCommunityForm;
  type UpdateForm = ModAddCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModAddCommunityForm) -> Result<Self, Error> {
    use crate::schema::mod_add_community::dsl::mod_add_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModAddCommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_add_community::dsl::mod_add_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModTransferCommunity {
  type InsertForm = ModTransferCommunityForm;
  type UpdateForm = ModTransferCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModTransferCommunityForm) -> Result<Self, Error> {
    use crate::schema::mod_transfer_community::dsl::mod_transfer_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_transfer_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: ModTransferCommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::mod_transfer_community::dsl::mod_transfer_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_transfer_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for ModAdd {
  type InsertForm = ModAddForm;
  type UpdateForm = ModAddForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: ModAddForm) -> Result<Self, Error> {
    use crate::schema::mod_add::dsl::mod_add;
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(pool: &mut DbPool<'_>, from_id: i32, form: ModAddForm) -> Result<Self, Error> {
    use crate::schema::mod_add::dsl::mod_add;
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for AdminPurgePerson {
  type InsertForm = AdminPurgePersonForm;
  type UpdateForm = AdminPurgePersonForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::admin_purge_person::dsl::admin_purge_person;
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_person)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: Self::InsertForm,
  ) -> Result<Self, Error> {
    use crate::schema::admin_purge_person::dsl::admin_purge_person;
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_person.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for AdminPurgeCommunity {
  type InsertForm = AdminPurgeCommunityForm;
  type UpdateForm = AdminPurgeCommunityForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::admin_purge_community::dsl::admin_purge_community;
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_community)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: Self::InsertForm,
  ) -> Result<Self, Error> {
    use crate::schema::admin_purge_community::dsl::admin_purge_community;
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_community.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for AdminPurgePost {
  type InsertForm = AdminPurgePostForm;
  type UpdateForm = AdminPurgePostForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::admin_purge_post::dsl::admin_purge_post;
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_post)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: Self::InsertForm,
  ) -> Result<Self, Error> {
    use crate::schema::admin_purge_post::dsl::admin_purge_post;
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_post.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Crud for AdminPurgeComment {
  type InsertForm = AdminPurgeCommentForm;
  type UpdateForm = AdminPurgeCommentForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::admin_purge_comment::dsl::admin_purge_comment;
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_comment)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: i32,
    form: Self::InsertForm,
  ) -> Result<Self, Error> {
    use crate::schema::admin_purge_comment::dsl::admin_purge_comment;
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_comment.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      moderator::{
        ModAdd,
        ModAddCommunity,
        ModAddCommunityForm,
        ModAddForm,
        ModBan,
        ModBanForm,
        ModBanFromCommunity,
        ModBanFromCommunityForm,
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
      },
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_mod = PersonInsertForm::builder()
      .name("the mod".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_mod = Person::create(pool, &new_mod).await.unwrap();

    let new_person = PersonInsertForm::builder()
      .name("jim2".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("mod_community".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post thweep".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    // Now the actual tests

    // remove post
    let mod_remove_post_form = ModRemovePostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      reason: None,
      removed: None,
    };
    let inserted_mod_remove_post = ModRemovePost::create(pool, &mod_remove_post_form)
      .await
      .unwrap();
    let read_mod_remove_post = ModRemovePost::read(pool, inserted_mod_remove_post.id)
      .await
      .unwrap();
    let expected_mod_remove_post = ModRemovePost {
      id: inserted_mod_remove_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      when_: inserted_mod_remove_post.when_,
    };

    // lock post

    let mod_lock_post_form = ModLockPostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      locked: None,
    };
    let inserted_mod_lock_post = ModLockPost::create(pool, &mod_lock_post_form)
      .await
      .unwrap();
    let read_mod_lock_post = ModLockPost::read(pool, inserted_mod_lock_post.id)
      .await
      .unwrap();
    let expected_mod_lock_post = ModLockPost {
      id: inserted_mod_lock_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      locked: true,
      when_: inserted_mod_lock_post.when_,
    };

    // feature post

    let mod_feature_post_form = ModFeaturePostForm {
      mod_person_id: inserted_mod.id,
      post_id: inserted_post.id,
      featured: false,
      is_featured_community: true,
    };
    let inserted_mod_feature_post = ModFeaturePost::create(pool, &mod_feature_post_form)
      .await
      .unwrap();
    let read_mod_feature_post = ModFeaturePost::read(pool, inserted_mod_feature_post.id)
      .await
      .unwrap();
    let expected_mod_feature_post = ModFeaturePost {
      id: inserted_mod_feature_post.id,
      post_id: inserted_post.id,
      mod_person_id: inserted_mod.id,
      featured: false,
      is_featured_community: true,
      when_: inserted_mod_feature_post.when_,
    };

    // comment

    let mod_remove_comment_form = ModRemoveCommentForm {
      mod_person_id: inserted_mod.id,
      comment_id: inserted_comment.id,
      reason: None,
      removed: None,
    };
    let inserted_mod_remove_comment = ModRemoveComment::create(pool, &mod_remove_comment_form)
      .await
      .unwrap();
    let read_mod_remove_comment = ModRemoveComment::read(pool, inserted_mod_remove_comment.id)
      .await
      .unwrap();
    let expected_mod_remove_comment = ModRemoveComment {
      id: inserted_mod_remove_comment.id,
      comment_id: inserted_comment.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      when_: inserted_mod_remove_comment.when_,
    };

    // community

    let mod_remove_community_form = ModRemoveCommunityForm {
      mod_person_id: inserted_mod.id,
      community_id: inserted_community.id,
      reason: None,
      removed: None,
      expires: None,
    };
    let inserted_mod_remove_community =
      ModRemoveCommunity::create(pool, &mod_remove_community_form)
        .await
        .unwrap();
    let read_mod_remove_community =
      ModRemoveCommunity::read(pool, inserted_mod_remove_community.id)
        .await
        .unwrap();
    let expected_mod_remove_community = ModRemoveCommunity {
      id: inserted_mod_remove_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      reason: None,
      removed: true,
      expires: None,
      when_: inserted_mod_remove_community.when_,
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
      ModBanFromCommunity::create(pool, &mod_ban_from_community_form)
        .await
        .unwrap();
    let read_mod_ban_from_community =
      ModBanFromCommunity::read(pool, inserted_mod_ban_from_community.id)
        .await
        .unwrap();
    let expected_mod_ban_from_community = ModBanFromCommunity {
      id: inserted_mod_ban_from_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: true,
      expires: None,
      when_: inserted_mod_ban_from_community.when_,
    };

    // ban

    let mod_ban_form = ModBanForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: None,
      expires: None,
    };
    let inserted_mod_ban = ModBan::create(pool, &mod_ban_form).await.unwrap();
    let read_mod_ban = ModBan::read(pool, inserted_mod_ban.id).await.unwrap();
    let expected_mod_ban = ModBan {
      id: inserted_mod_ban.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      reason: None,
      banned: true,
      expires: None,
      when_: inserted_mod_ban.when_,
    };

    // mod add community

    let mod_add_community_form = ModAddCommunityForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      community_id: inserted_community.id,
      removed: None,
    };
    let inserted_mod_add_community = ModAddCommunity::create(pool, &mod_add_community_form)
      .await
      .unwrap();
    let read_mod_add_community = ModAddCommunity::read(pool, inserted_mod_add_community.id)
      .await
      .unwrap();
    let expected_mod_add_community = ModAddCommunity {
      id: inserted_mod_add_community.id,
      community_id: inserted_community.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      when_: inserted_mod_add_community.when_,
    };

    // mod add

    let mod_add_form = ModAddForm {
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: None,
    };
    let inserted_mod_add = ModAdd::create(pool, &mod_add_form).await.unwrap();
    let read_mod_add = ModAdd::read(pool, inserted_mod_add.id).await.unwrap();
    let expected_mod_add = ModAdd {
      id: inserted_mod_add.id,
      mod_person_id: inserted_mod.id,
      other_person_id: inserted_person.id,
      removed: false,
      when_: inserted_mod_add.when_,
    };

    Comment::delete(pool, inserted_comment.id).await.unwrap();
    Post::delete(pool, inserted_post.id).await.unwrap();
    Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_person.id).await.unwrap();
    Person::delete(pool, inserted_mod.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_mod_remove_post, read_mod_remove_post);
    assert_eq!(expected_mod_lock_post, read_mod_lock_post);
    assert_eq!(expected_mod_feature_post, read_mod_feature_post);
    assert_eq!(expected_mod_remove_comment, read_mod_remove_comment);
    assert_eq!(expected_mod_remove_community, read_mod_remove_community);
    assert_eq!(expected_mod_ban_from_community, read_mod_ban_from_community);
    assert_eq!(expected_mod_ban, read_mod_ban);
    assert_eq!(expected_mod_add_community, read_mod_add_community);
    assert_eq!(expected_mod_add, read_mod_add);
  }
}
