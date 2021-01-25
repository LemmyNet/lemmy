use crate::{
  fetcher::{fetch::fetch_remote_object, is_deleted, should_refetch_actor},
  objects::FromApub,
  PersonExt,
};
use anyhow::anyhow;
use diesel::result::Error::NotFound;
use lemmy_db_queries::{source::user::User, ApubObject};
use lemmy_db_schema::source::user::User_;
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

/// Get a user from its apub ID.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_user(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<User_, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_apub_id(conn, &apub_id_owned.into())
  })
  .await?;

  match user {
    // If its older than a day, re-fetch it
    Ok(u) if !u.local && should_refetch_actor(u.last_refreshed_at) => {
      debug!("Fetching and updating from remote user: {}", apub_id);
      let person =
        fetch_remote_object::<PersonExt>(context.client(), apub_id, recursion_counter).await;

      if is_deleted(&person) {
        // TODO: use User_::update_deleted() once implemented
        blocking(context.pool(), move |conn| {
          User_::delete_account(conn, u.id)
        })
        .await??;
        return Err(anyhow!("User was deleted by remote instance").into());
      } else if person.is_err() {
        return Ok(u);
      }

      let user = User_::from_apub(&person?, context, apub_id.to_owned(), recursion_counter).await?;

      let user_id = user.id;
      blocking(context.pool(), move |conn| {
        User_::mark_as_updated(conn, user_id)
      })
      .await??;

      Ok(user)
    }
    Ok(u) => Ok(u),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote user: {}", apub_id);
      let person =
        fetch_remote_object::<PersonExt>(context.client(), apub_id, recursion_counter).await?;

      let user = User_::from_apub(&person, context, apub_id.to_owned(), recursion_counter).await?;

      Ok(user)
    }
    Err(e) => Err(e.into()),
  }
}
