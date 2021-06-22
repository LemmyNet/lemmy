use activitystreams::base::AnyBase;
use anyhow::Context;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use std::{collections::HashMap, str::FromStr};
use strum_macros::EnumString;

// for now, limit it to activity routing only, no http sigs, parsing or any of that
// need to route in this order:
// 1. recipient actor
// 2. activity type
// 3. inner object (recursively until object is empty or an url)

// library part

/// macro shorthand to create hashmap
/// usage: `let counts = hashmap!['A' => 0, 'C' => 0, 'G' => 0, 'T' => 0];`
/// from https://stackoverflow.com/questions/28392008/more-concise-hashmap-initialization
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

#[derive(Hash, Eq, PartialEq, EnumString)]
enum ActivityTypes {
  Follow,
  Announce,
  Create,
}

#[derive(Eq, PartialEq)]
enum ObjectTypes {
  Page,
  Note,
  Url,  // we dont dereference urls in object field, so we dont know what exactly it refers to
  None, // object field doesnt exist
}

struct InboxConfig {
  actors: Vec<ActorConfig>,
}

impl InboxConfig {
  fn shared_inbox_handler() {
    todo!()
  }
}

type AcceptedTypes = HashMap<ActivityTypes, InnerType>;

// TODO: need to provide a handler function for each value
enum InnerType {
  Simple(ObjectTypes),
  Nested(AcceptedTypes),
}

struct ActorConfig {
  accepted_types: AcceptedTypes,
}

impl ActorConfig {
  pub(crate) fn actor_inbox_handler(
    self,
    activity: AnyBase,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    // TODO: probably better to define our own struct with the fields we need + unparsed, and later
    //       convert to activity (needs id_unchecked(), kind, object)
    let kind = ActivityTypes::from_str(activity.kind_str().context(location_info!())?)?;
    use InnerType::*;
    match self.accepted_types.get(&kind).context(location_info!())? {
      Simple(o) => {}
      Nested(a) => {}
    }
    // TODO: correctly route the activity to handle_follow, receive_create_comment or receive_create_post
    todo!()
  }
}

// application part

pub(crate) fn receive_activity(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  use ActivityTypes::*;
  use InnerType::*;
  use ObjectTypes::*;

  let accepted_types = hashmap![Follow => Simple(Url),
      Announce =>
       Nested(hashmap![Create => Simple(Note), Create => Simple(Page)])];
  let community_inbox_config = ActorConfig { accepted_types };
  let inbox_config = InboxConfig { actors: vec![] };
  community_inbox_config.actor_inbox_handler(activity, context)?;
  Ok(())
}
