use crate::{
  newtypes::{
    AdminAllowInstanceId,
    AdminBlockInstanceId,
    AdminPurgeCommentId,
    AdminPurgeCommunityId,
    AdminPurgePersonId,
    AdminPurgePostId,
  },
  schema::{
    admin_allow_instance,
    admin_block_instance,
    admin_purge_comment,
    admin_purge_community,
    admin_purge_person,
    admin_purge_post,
  },
  source::mod_log::admin::{
    AdminAllowInstance,
    AdminAllowInstanceForm,
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
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Crud for AdminPurgePerson {
  type InsertForm = AdminPurgePersonForm;
  type UpdateForm = AdminPurgePersonForm;
  type IdType = AdminPurgePersonId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_person::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for AdminPurgeCommunity {
  type InsertForm = AdminPurgeCommunityForm;
  type UpdateForm = AdminPurgeCommunityForm;
  type IdType = AdminPurgeCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for AdminPurgePost {
  type InsertForm = AdminPurgePostForm;
  type UpdateForm = AdminPurgePostForm;
  type IdType = AdminPurgePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for AdminPurgeComment {
  type InsertForm = AdminPurgeCommentForm;
  type UpdateForm = AdminPurgeCommentForm;
  type IdType = AdminPurgeCommentId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_purge_comment::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_comment::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for AdminAllowInstance {
  type InsertForm = AdminAllowInstanceForm;
  type UpdateForm = AdminAllowInstanceForm;
  type IdType = AdminAllowInstanceId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_allow_instance::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_allow_instance::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl Crud for AdminBlockInstance {
  type InsertForm = AdminBlockInstanceForm;
  type UpdateForm = AdminBlockInstanceForm;
  type IdType = AdminBlockInstanceId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_block_instance::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_block_instance::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
