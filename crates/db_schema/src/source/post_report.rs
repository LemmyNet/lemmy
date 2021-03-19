use crate::{schema::post_report, source::post::Post, DbUrl, PersonId, PostId};
use serde::{Deserialize, Serialize};

#[derive(
  Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug, Clone,
)]
#[belongs_to(Post)]
#[table_name = "post_report"]
pub struct PostReport {
  pub id: i32,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "post_report"]
pub struct PostReportForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
}
