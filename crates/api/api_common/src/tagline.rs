pub use lemmy_db_schema::source::tagline::Tagline;
pub use lemmy_db_schema_file::newtypes::TaglineId;
pub use lemmy_db_views_site::api::{ListTaglines, TaglineResponse};

pub mod administration {
  pub use lemmy_db_views_site::api::{CreateTagline, DeleteTagline, EditTagline};
}
