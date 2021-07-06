use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod comment;
pub mod community;
pub mod following;
pub mod post;
pub mod private_message;

// TODO: remove this
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LemmyActivity<Kind> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  id: Url,
  pub(crate) actor: Url,

  /// type-specific fields
  #[serde(flatten)]
  pub inner: Kind,

  // unparsed fields
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl<Kind> LemmyActivity<Kind> {
  pub fn id_unchecked(&self) -> &Url {
    &self.id
  }
}

async fn verify_mod_action(
  actor_id: Url,
  activity_cc: Url,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &activity_cc.into())
  })
  .await??;

  if community.local {
    let actor = blocking(context.pool(), move |conn| {
      Person::read_from_apub_id(conn, &actor_id.into())
    })
    .await??;

    // Note: this will also return true for admins in addition to mods, but as we dont know about
    //       remote admins, it doesnt make any difference.
    let community_id = community.id;
    let actor_id = actor.id;
    let is_mod_or_admin = blocking(context.pool(), move |conn| {
      CommunityView::is_mod_or_admin(conn, actor_id, community_id)
    })
    .await?;
    if !is_mod_or_admin {
      return Err(anyhow!("Not a mod").into());
    }
  }
  Ok(())
}
