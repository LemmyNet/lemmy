use crate::{
  source::mod_log::admin::{
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

#[async_trait]
impl Crud for AdminPurgePerson {
  type InsertForm = AdminPurgePersonForm;
  type UpdateForm = AdminPurgePersonForm;
  type IdType = i32;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
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
    form: &Self::InsertForm,
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

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
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
    form: &Self::InsertForm,
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

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
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
    form: &Self::InsertForm,
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

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
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
    form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    use crate::schema::admin_purge_comment::dsl::admin_purge_comment;
    let conn = &mut get_conn(pool).await?;
    diesel::update(admin_purge_comment.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
