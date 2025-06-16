pub use lemmy_db_schema::{newtypes::TaglineId, source::tagline::Tagline};
pub use lemmy_db_views_site::api::{ListTaglines, ListTaglinesResponse, TaglineResponse};

pub mod aministration {
  pub use lemmy_db_views_site::api::{CreateTagline, DeleteTagline, UpdateTagline};
}
