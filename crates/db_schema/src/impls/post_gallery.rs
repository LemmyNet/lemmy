use crate::{
  newtypes::{PostGalleryId, PostId},
  source::post_gallery::{PostGallery, PostGalleryInsertForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::post_gallery;

use crate::source::post_gallery::PostGalleryView;
use diesel::{
  deserialize::FromSql,
  pg::{Pg, PgValue},
  serialize::ToSql,
  sql_types::{self, Nullable},
};


impl Crud for PostGallery {
  type InsertForm = PostGalleryInsertForm;
  type UpdateForm = PostGalleryInsertForm;
  type IdType = PostGalleryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_gallery::table)
      .values(form)
      .on_conflict((post_gallery::post_id, post_gallery::url))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    post_url_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post_gallery::table.find(post_url_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PostGallery {
  pub async fn list_from_post_id(
    post_id: PostId,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    post_gallery::table
      .filter(post_gallery::post_id.eq(post_id))
      .order(post_gallery::page)
      .load::<Self>(conn)
      .await
  }

  pub async fn create_from_vec(
    forms: &Vec<PostGalleryInsertForm>,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_gallery::table)
      .values(forms)
      .get_results::<Self>(conn)
      .await
  }

  pub async fn delete_from_post_id(
    post_id: PostId,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(post_gallery::table)
      .filter(post_gallery::post_id.eq(post_id))
      .get_results::<Self>(conn)
      .await
  }
}

impl FromSql<Nullable<sql_types::Json>, Pg> for PostGalleryView {
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<sql_types::Json, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value::<PostGalleryView>(value)?)
  }

  fn from_nullable_sql(bytes: Option<PgValue>) -> diesel::deserialize::Result<Self> {
    match bytes {
      Some(bytes) => Self::from_sql(bytes),
      None => Ok(Self(vec![])),
    }
  }
}

impl ToSql<Nullable<sql_types::Json>, Pg> for PostGalleryView {
  fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<sql_types::Json, Pg>>::to_sql(&value, &mut out.reborrow())
  }
}
