#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::{
  federation_allowlist::FederationAllowList,
  federation_blocklist::FederationBlockList,
  federation_queue_state::FederationQueueState,
  instance::Instance,
  local_site::LocalSite,
  local_site_rate_limit::LocalSiteRateLimit,
  site::Site,
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::{CommunityView, MultiCommunityView};
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A site view.
pub struct SiteView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub site: Site,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site: LocalSite,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site_rate_limit: LocalSiteRateLimit,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Instance,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederatedInstanceView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Instance,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub site: Option<Site>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub queue_state: Option<FederationQueueState>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub blocked: Option<FederationBlockList>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub allowed: Option<FederationAllowList>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum ResolveObjectView {
  Post(PostView),
  Comment(CommentView),
  Person(PersonView),
  Community(CommunityView),
  MultiCommunity(MultiCommunityView),
}
