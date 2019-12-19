use crate::apub::make_apub_endpoint;
use crate::db::community::Community;
use crate::to_datetime_utc;
use activitypub::{actor::Group, context};

impl Community {
  pub fn as_group(&self) -> Group {
    let base_url = make_apub_endpoint("community", &self.name);

    let mut group = Group::default();

    group.object_props.set_context_object(context()).ok();
    group.object_props.set_id_string(base_url.to_string()).ok();
    group
      .object_props
      .set_name_string(self.name.to_owned())
      .ok();
    group
      .object_props
      .set_published_utctime(to_datetime_utc(self.published))
      .ok();
    if let Some(updated) = self.updated {
      group
        .object_props
        .set_updated_utctime(to_datetime_utc(updated))
        .ok();
    }

    if let Some(description) = &self.description {
      group
        .object_props
        .set_summary_string(description.to_string())
        .ok();
    }

    group
      .ap_actor_props
      .set_inbox_string(format!("{}/inbox", &base_url))
      .ok();
    group
      .ap_actor_props
      .set_outbox_string(format!("{}/outbox", &base_url))
      .ok();
    group
      .ap_actor_props
      .set_followers_string(format!("{}/followers", &base_url))
      .ok();

    group
  }
}
