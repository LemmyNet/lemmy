use crate::{
  newtypes::CommunityId,
  source::{
    comment::Comment,
    modlog::{Modlog, ModlogInsertForm},
    person::Person,
    post::Post,
  },
};
use chrono::{DateTime, Utc};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::modlog;
use lemmy_db_schema_file::{InstanceId, PersonId, enums::ModlogKind};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Modlog {
  pub async fn create<'a>(
    pool: &mut DbPool<'_>,
    form: &[ModlogInsertForm<'a>],
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    insert_into(modlog::table)
      .values(form)
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }
}

impl<'a> ModlogInsertForm<'a> {
  pub fn admin_ban(
    mod_person: &Person,
    target_person_id: PersonId,
    banned: bool,
    expires_at: Option<DateTime<Utc>>,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      expires_at,
      target_person_id: Some(target_person_id),
      target_instance_id: Some(mod_person.instance_id),
      ..ModlogInsertForm::new(ModlogKind::AdminBan, !banned, mod_person.id)
    }
  }
  pub fn admin_add(mod_person: &Person, target_person_id: PersonId, added: bool) -> Self {
    Self {
      target_person_id: Some(target_person_id),
      ..ModlogInsertForm::new(ModlogKind::AdminAdd, !added, mod_person.id)
    }
  }
  pub fn mod_remove_post(
    mod_person_id: PersonId,
    post: &Post,
    removed: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_post_id: Some(post.id),
      target_person_id: Some(post.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModRemovePost, !removed, mod_person_id)
    }
  }
  pub fn mod_remove_comment(
    mod_person_id: PersonId,
    comment: &Comment,
    removed: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_comment_id: Some(comment.id),
      target_post_id: Some(comment.post_id),
      target_person_id: Some(comment.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModRemoveComment, !removed, mod_person_id)
    }
  }
  pub fn mod_lock_comment(
    mod_person_id: PersonId,
    comment: &Comment,
    removed: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_comment_id: Some(comment.id),
      target_person_id: Some(comment.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModLockComment, !removed, mod_person_id)
    }
  }
  pub fn mod_lock_post(
    mod_person_id: PersonId,
    post: &Post,
    locked: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_post_id: Some(post.id),
      target_community_id: Some(post.community_id),
      target_person_id: Some(post.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModLockPost, !locked, mod_person_id)
    }
  }
  pub fn mod_create_comment_warning(
    mod_person_id: PersonId,
    comment: &Comment,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_comment_id: Some(comment.id),
      target_person_id: Some(comment.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModWarnComment, false, mod_person_id)
    }
  }
  pub fn mod_create_post_warning(mod_person_id: PersonId, post: &Post, reason: &'a str) -> Self {
    Self {
      reason: Some(reason),
      target_post_id: Some(post.id),
      target_community_id: Some(post.community_id),
      target_person_id: Some(post.creator_id),
      ..ModlogInsertForm::new(ModlogKind::ModWarnPost, false, mod_person_id)
    }
  }
  pub fn admin_remove_community(
    mod_person_id: PersonId,
    community_id: CommunityId,
    community_owner_id: Option<PersonId>,
    removed: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_community_id: Some(community_id),
      target_person_id: community_owner_id,
      ..ModlogInsertForm::new(ModlogKind::AdminRemoveCommunity, !removed, mod_person_id)
    }
  }

  pub fn mod_change_community_visibility(
    mod_person_id: PersonId,
    community_id: CommunityId,
  ) -> Self {
    Self {
      target_community_id: Some(community_id),
      ..ModlogInsertForm::new(
        ModlogKind::ModChangeCommunityVisibility,
        false,
        mod_person_id,
      )
    }
  }
  pub fn mod_ban_from_community(
    mod_person_id: PersonId,
    community_id: CommunityId,
    target_person_id: PersonId,
    removed: bool,
    expires_at: Option<DateTime<Utc>>,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      expires_at,
      target_community_id: Some(community_id),
      target_person_id: Some(target_person_id),
      ..ModlogInsertForm::new(ModlogKind::ModBanFromCommunity, !removed, mod_person_id)
    }
  }
  pub fn mod_add_to_community(
    mod_person_id: PersonId,
    community_id: CommunityId,
    target_person_id: PersonId,
    added: bool,
  ) -> Self {
    Self {
      target_community_id: Some(community_id),
      target_person_id: Some(target_person_id),
      ..ModlogInsertForm::new(ModlogKind::ModAddToCommunity, !added, mod_person_id)
    }
  }
  pub fn mod_transfer_community(
    mod_person_id: PersonId,
    community_id: CommunityId,
    target_person_id: PersonId,
  ) -> Self {
    Self {
      target_community_id: Some(community_id),
      target_person_id: Some(target_person_id),
      ..ModlogInsertForm::new(ModlogKind::ModTransferCommunity, false, mod_person_id)
    }
  }
  pub fn admin_allow_instance(
    mod_person_id: PersonId,
    instance_id: InstanceId,
    allow: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_instance_id: Some(instance_id),
      ..ModlogInsertForm::new(ModlogKind::AdminAllowInstance, !allow, mod_person_id)
    }
  }
  pub fn admin_block_instance(
    mod_person_id: PersonId,
    instance_id: InstanceId,
    block: bool,
    reason: &'a str,
  ) -> Self {
    Self {
      reason: Some(reason),
      target_instance_id: Some(instance_id),
      ..ModlogInsertForm::new(ModlogKind::AdminBlockInstance, !block, mod_person_id)
    }
  }
  pub fn admin_purge_comment(
    mod_person_id: PersonId,
    comment: &Comment,
    community_id: CommunityId,
    reason: &'a str,
  ) -> Self {
    Self {
      target_post_id: Some(comment.post_id),
      target_person_id: Some(comment.creator_id),
      target_community_id: Some(community_id),
      reason: Some(reason),
      ..ModlogInsertForm::new(ModlogKind::AdminPurgeComment, false, mod_person_id)
    }
  }
  pub fn admin_purge_post(
    mod_person_id: PersonId,
    community_id: CommunityId,
    reason: &'a str,
  ) -> Self {
    Self {
      target_community_id: Some(community_id),
      reason: Some(reason),
      ..ModlogInsertForm::new(ModlogKind::AdminPurgePost, false, mod_person_id)
    }
  }
  pub fn admin_purge_community(mod_person_id: PersonId, reason: &'a str) -> Self {
    Self {
      reason: Some(reason),
      ..ModlogInsertForm::new(ModlogKind::AdminPurgeCommunity, false, mod_person_id)
    }
  }
  pub fn admin_purge_person(mod_person_id: PersonId, reason: &'a str) -> Self {
    Self {
      reason: Some(reason),
      ..ModlogInsertForm::new(ModlogKind::AdminPurgePerson, false, mod_person_id)
    }
  }
  pub fn mod_feature_post_community(mod_person_id: PersonId, post: &Post, featured: bool) -> Self {
    Self {
      target_post_id: Some(post.id),
      target_community_id: Some(post.community_id),
      ..ModlogInsertForm::new(
        ModlogKind::ModFeaturePostCommunity,
        !featured,
        mod_person_id,
      )
    }
  }
  pub fn admin_feature_post_site(mod_person_id: PersonId, post: &Post, featured: bool) -> Self {
    Self {
      target_post_id: Some(post.id),
      ..ModlogInsertForm::new(ModlogKind::AdminFeaturePostSite, !featured, mod_person_id)
    }
  }
}
