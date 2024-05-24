use crate::{
  schema::{local_site, local_user, site},
  ListingType,
  PostListingMode,
  SortType,
};

pub struct Viewer {
  logged_in_viewer: Option<LoggedInViewer>,
  local_site_fields: LocalSiteFields,
  site_fields: SiteFields,
};

struct LoggedInViewer {
  local_user_fields: LocalUserFields,
}

#[derive(Queryable)]
#[diesel(table_name = local_site)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LocalUserFields {
  pub id: LocalUserId,
  pub person_id: PersonId,
  pub show_nsfw: bool,
  pub default_sort_type: SortType,
  pub default_listing_type: ListingType,
  pub show_bot_accounts: bool,
  pub show_read_posts: bool,
  pub admin: bool,
  // TODO: remove if not needed
  pub email: Option<SensitiveString>,
  pub interface_language: String,
  pub send_notifications_to_email: bool,
}

#[derive(Queryable)]
#[diesel(table_name = local_site)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LocalSiteFilds {
  default_post_listing_type: ListingType,
  default_post_listing_mode: PostListingMode,
  default_sort_type: SortType,
}

#[derive(Queryable)]
#[diesel(table_name = local_site)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SiteFields {
  content_warning: Option<String>,
}
