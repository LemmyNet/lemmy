use crate::schema;

impl diesel_uplete::SupportedTable for schema::comment_actions::table {
  type Key = (
    schema::comment_actions::person_id,
    schema::comment_actions::comment_id,
  );
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for schema::community_actions::table {
  type Key = (
    schema::community_actions::person_id,
    schema::community_actions::community_id,
  );
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for schema::instance_actions::table {
  type Key = (
    schema::instance_actions::person_id,
    schema::instance_actions::instance_id,
  );
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for schema::person_actions::table {
  type Key = (
    schema::person_actions::person_id,
    schema::person_actions::target_id,
  );
  type AdditionalIgnoredColumns = ();
}

impl diesel_uplete::SupportedTable for schema::post_actions::table {
  type Key = (
    schema::post_actions::person_id,
    schema::post_actions::post_id,
  );
  type AdditionalIgnoredColumns = ();
}
