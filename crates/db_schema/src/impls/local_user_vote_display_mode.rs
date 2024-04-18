use crate::{
  diesel::OptionalExtension,
  newtypes::LocalUserId,
  schema::local_user_vote_display_mode,
  source::local_user_vote_display_mode::{
    LocalUserVoteDisplayMode,
    LocalUserVoteDisplayModeInsertForm,
    LocalUserVoteDisplayModeUpdateForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl LocalUserVoteDisplayMode {
  pub async fn read(pool: &mut DbPool<'_>) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    local_user_vote_display_mode::table
      .first(conn)
      .await
      .optional()
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &LocalUserVoteDisplayModeInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_user_vote_display_mode::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    form: &LocalUserVoteDisplayModeUpdateForm,
  ) -> Result<(), Error> {
    // avoid error "There are no changes to save. This query cannot be built"
    if form.is_empty() {
      return Ok(());
    }
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user_vote_display_mode::table.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
      .await?;
    Ok(())
  }
}

impl LocalUserVoteDisplayModeUpdateForm {
  fn is_empty(&self) -> bool {
    self.score.is_none()
      && self.upvotes.is_none()
      && self.downvotes.is_none()
      && self.upvote_percentage.is_none()
  }
}
