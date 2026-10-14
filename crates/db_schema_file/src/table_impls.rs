use crate::schema::{
  comment_actions,
  community_actions,
  instance_actions,
  person_actions,
  post_actions,
};

impl diesel_uplete::SupportedTable for comment_actions::table {
  type Key = (comment_actions::person_id, comment_actions::comment_id);
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for community_actions::table {
  type Key = (
    community_actions::person_id,
    community_actions::community_id,
  );
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for instance_actions::table {
  type Key = (instance_actions::person_id, instance_actions::instance_id);
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for person_actions::table {
  type Key = (person_actions::person_id, person_actions::target_id);
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for post_actions::table {
  type Key = (post_actions::person_id, post_actions::post_id);
  type AdditionalIgnoredColumns = ();
}
