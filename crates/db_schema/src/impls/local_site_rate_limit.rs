use crate::{
  schema::local_site_rate_limit,
  source::local_site_rate_limit::{
    LocalSiteRateLimit,
    LocalSiteRateLimitInsertForm,
    LocalSiteRateLimitUpdateForm,
  },
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl LocalSiteRateLimit {
  pub async fn read(conn: &mut DbConn) -> Result<Self, Error> {
    local_site_rate_limit::table.first::<Self>(conn).await
  }

  pub async fn create(
    conn: &mut DbConn,
    form: &LocalSiteRateLimitInsertForm,
  ) -> Result<Self, Error> {
    insert_into(local_site_rate_limit::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(conn: &mut DbConn, form: &LocalSiteRateLimitUpdateForm) -> Result<(), Error> {
    // avoid error "There are no changes to save. This query cannot be built"
    if form.is_empty() {
      return Ok(());
    }
    diesel::update(local_site_rate_limit::table)
      .set(form)
      .get_result::<Self>(conn)
      .await?;
    Ok(())
  }
}

impl LocalSiteRateLimitUpdateForm {
  fn is_empty(&self) -> bool {
    self.message.is_none()
      && self.message_per_second.is_none()
      && self.post.is_none()
      && self.post_per_second.is_none()
      && self.register.is_none()
      && self.register_per_second.is_none()
      && self.image.is_none()
      && self.image_per_second.is_none()
      && self.comment.is_none()
      && self.comment_per_second.is_none()
      && self.search.is_none()
      && self.search_per_second.is_none()
      && self.updated.is_none()
  }
}
