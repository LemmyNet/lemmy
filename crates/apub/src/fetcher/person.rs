use crate::{
  fetcher::{fetch::fetch_remote_object, is_deleted, should_refetch_actor},
  objects::{person::Person as ApubPerson, FromApub},
};
use anyhow::anyhow;
use diesel::result::Error::NotFound;
use lemmy_api_common::blocking;
use lemmy_db_queries::{source::person::Person_, ApubObject};
use lemmy_db_schema::source::person::Person;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

/// Get a person from its apub ID.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_person(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Person, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let person = blocking(context.pool(), move |conn| {
    Person::read_from_apub_id(conn, &apub_id_owned.into())
  })
  .await?;

  match person {
    // If its older than a day, re-fetch it
    Ok(u) if !u.local && should_refetch_actor(u.last_refreshed_at) => {
      debug!("Fetching and updating from remote person: {}", apub_id);
      let person =
        fetch_remote_object::<ApubPerson>(context.client(), apub_id, recursion_counter).await;

      if is_deleted(&person) {
        // TODO: use Person::update_deleted() once implemented
        blocking(context.pool(), move |conn| {
          Person::delete_account(conn, u.id)
        })
        .await??;
        return Err(anyhow!("Person was deleted by remote instance").into());
      } else if person.is_err() {
        return Ok(u);
      }

      let person = Person::from_apub(&person?, context, apub_id, recursion_counter).await?;

      let person_id = person.id;
      blocking(context.pool(), move |conn| {
        Person::mark_as_updated(conn, person_id)
      })
      .await??;

      Ok(person)
    }
    Ok(u) => Ok(u),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote person: {}", apub_id);
      let person =
        fetch_remote_object::<ApubPerson>(context.client(), apub_id, recursion_counter).await?;

      let person = Person::from_apub(&person, context, apub_id, recursion_counter).await?;

      Ok(person)
    }
    Err(e) => Err(e.into()),
  }
}
