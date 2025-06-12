pub use lemmy_db_schema::{newtypes::TaglineId, source::tagline::Tagline};
pub use lemmy_db_views_tagline::{ListTaglines, ListTaglinesResponse, TaglineResponse};

pub mod aministration {
  pub use lemmy_db_views_tagline::{CreateTagline, DeleteTagline, UpdateTagline};
}
