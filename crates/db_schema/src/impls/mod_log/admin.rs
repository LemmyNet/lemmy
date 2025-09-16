use crate::{
  newtypes::{
    AdminAddId,
    AdminAllowInstanceId,
    AdminBanId,
    AdminBlockInstanceId,
    AdminPurgeCommentId,
    AdminPurgeCommunityId,
    AdminPurgePersonId,
    AdminPurgePostId,
    AdminRemoveCommunityId,
    PersonId,
  },
  source::{
    mod_log::admin::{
      AdminAdd,
      AdminAddForm,
      AdminAllowInstance,
      AdminAllowInstanceForm,
      AdminBan,
      AdminBanForm,
      AdminBlockInstance,
      AdminBlockInstanceForm,
      AdminPurgeComment,
      AdminPurgeCommentForm,
      AdminPurgeCommunity,
      AdminPurgeCommunityForm,
      AdminPurgePerson,
      AdminPurgePersonForm,
      AdminPurgePost,
      AdminPurgePostForm,
      AdminRemoveCommunity,
      AdminRemoveCommunityForm,
    },
    notification::NotificationInsertForm,
  },
  traits::{Crud, ModActionNotify},
  utils::{get_conn, DbPool},
  ModlogActionType,
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{
  enums::NotificationType,
  schema::{
    admin_add,
    admin_allow_instance,
    admin_ban,
    admin_block_instance,
    admin_purge_comment,
    admin_purge_community,
    admin_purge_person,
    admin_purge_post,
    admin_remove_community,
  },
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for AdminPurgePerson {
  type InsertForm = AdminPurgePersonForm;
  type UpdateForm = AdminPurgePersonForm;
  type IdType = AdminPurgePersonId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_person::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminPurgeCommunity {
  type InsertForm = AdminPurgeCommunityForm;
  type UpdateForm = AdminPurgeCommunityForm;
  type IdType = AdminPurgeCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminPurgePost {
  type InsertForm = AdminPurgePostForm;
  type UpdateForm = AdminPurgePostForm;
  type IdType = AdminPurgePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminPurgeComment {
  type InsertForm = AdminPurgeCommentForm;
  type UpdateForm = AdminPurgeCommentForm;
  type IdType = AdminPurgeCommentId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_comment::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_comment::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminAllowInstance {
  type InsertForm = AdminAllowInstanceForm;
  type UpdateForm = AdminAllowInstanceForm;
  type IdType = AdminAllowInstanceId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_allow_instance::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_allow_instance::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminBlockInstance {
  type InsertForm = AdminBlockInstanceForm;
  type UpdateForm = AdminBlockInstanceForm;
  type IdType = AdminBlockInstanceId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_block_instance::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_block_instance::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminRemoveCommunity {
  type InsertForm = AdminRemoveCommunityForm;
  type UpdateForm = AdminRemoveCommunityForm;
  type IdType = AdminRemoveCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_remove_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_remove_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminBan {
  type InsertForm = AdminBanForm;
  type UpdateForm = AdminBanForm;
  type IdType = AdminBanId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_ban::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_ban::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Crud for AdminAdd {
  type InsertForm = AdminAddForm;
  type UpdateForm = AdminAddForm;
  type IdType = AdminAddId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_add::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_add::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl ModActionNotify for AdminRemoveCommunity {
  fn insert_form(&self, recipient_id: PersonId) -> NotificationInsertForm {
    NotificationInsertForm {
      admin_remove_community_id: Some(self.id),
      ..NotificationInsertForm::new(recipient_id, NotificationType::ModAction)
    }
  }
  fn kind(&self) -> ModlogActionType {
    ModlogActionType::AdminRemoveCommunity
  }
  fn is_revert(&self) -> bool {
    self.removed
  }
  fn reason(&self) -> Option<&str> {
    Some(&self.reason)
  }
}

impl ModActionNotify for AdminAdd {
  fn insert_form(&self, recipient_id: PersonId) -> NotificationInsertForm {
    NotificationInsertForm {
      admin_add_id: Some(self.id),
      ..NotificationInsertForm::new(recipient_id, NotificationType::ModAction)
    }
  }
  fn kind(&self) -> ModlogActionType {
    ModlogActionType::AdminAdd
  }
  fn is_revert(&self) -> bool {
    self.removed
  }
  fn reason(&self) -> Option<&str> {
    None
  }
}

impl ModActionNotify for AdminBan {
  fn insert_form(&self, recipient_id: PersonId) -> NotificationInsertForm {
    NotificationInsertForm {
      admin_ban_id: Some(self.id),
      ..NotificationInsertForm::new(recipient_id, NotificationType::ModAction)
    }
  }
  fn kind(&self) -> ModlogActionType {
    ModlogActionType::AdminBan
  }
  fn is_revert(&self) -> bool {
    self.banned
  }
  fn reason(&self) -> Option<&str> {
    Some(&self.reason)
  }
}
