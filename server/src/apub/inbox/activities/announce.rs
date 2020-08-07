use crate::{
  apub::inbox::{
    activities::{
      create::receive_create,
      delete::receive_delete,
      dislike::receive_dislike,
      like::receive_like,
      remove::receive_remove,
      undo::receive_undo,
      update::receive_update,
    },
    shared_inbox::{get_community_id_from_activity, receive_unhandled_activity},
  },
  routes::ChatServerParam,
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::*,
  base::{AnyBase, BaseExt},
  prelude::ExtendsExt,
};
use actix_web::{client::Client, HttpResponse};

pub async fn receive_announce(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let announce = Announce::from_any_base(activity)?.unwrap();

  // ensure that announce and community come from the same instance
  let community = get_community_id_from_activity(&announce)?;
  announce.id(community.domain().unwrap())?;

  let kind = announce.object().as_single_kind_str();
  let object = announce.object();
  let object2 = object.clone().one().unwrap();
  match kind {
    Some("Create") => receive_create(object2, client, pool, chat_server).await,
    Some("Update") => receive_update(object2, client, pool, chat_server).await,
    Some("Like") => receive_like(object2, client, pool, chat_server).await,
    Some("Dislike") => receive_dislike(object2, client, pool, chat_server).await,
    Some("Delete") => receive_delete(object2, client, pool, chat_server).await,
    Some("Remove") => receive_remove(object2, client, pool, chat_server).await,
    Some("Undo") => receive_undo(object2, client, pool, chat_server).await,
    _ => receive_unhandled_activity(announce),
  }
}
