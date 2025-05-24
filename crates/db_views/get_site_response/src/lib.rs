use lemmy_db_schema::{
  newtypes::LanguageId,
  source::{
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    oauth_provider::{OAuthProvider, PublicOAuthProvider},
    tagline::Tagline,
  },
};
use lemmy_db_views_my_user_info::MyUserInfo;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_plugin_metadata::PluginMetadata;
use lemmy_db_views_site::SiteView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  #[cfg_attr(feature = "full", ts(skip))]
  pub my_user: Option<MyUserInfo>,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  #[cfg_attr(feature = "full", ts(optional))]
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  pub oauth_providers: Vec<PublicOAuthProvider>,
  pub admin_oauth_providers: Vec<OAuthProvider>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  // If true then uploads for post images or markdown images are disabled. Only avatars, icons and
  // banners can be set.
  pub image_upload_disabled: bool,
  pub active_plugins: Vec<PluginMetadata>,
}
