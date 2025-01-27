use super::pictrs_placeholder_url;
use doku::Document;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{
  env,
  net::{IpAddr, Ipv4Addr},
};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct Settings {
  /// settings related to the postgresql database
  pub database: DatabaseConfig,
  /// Pictrs image server configuration.
  #[default(Some(Default::default()))]
  pub(crate) pictrs: Option<PictrsConfig>,
  /// Email sending configuration. All options except login/password are mandatory
  #[doku(example = "Some(Default::default())")]
  pub email: Option<EmailConfig>,
  /// Parameters for automatic configuration of new instance (only used at first start)
  #[doku(example = "Some(Default::default())")]
  pub setup: Option<SetupConfig>,
  /// the domain name of your instance (mandatory)
  #[default("unset")]
  #[doku(example = "example.com")]
  pub hostname: String,
  /// Address where lemmy should listen for incoming requests
  #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
  #[doku(as = "String")]
  pub bind: IpAddr,
  /// Port where lemmy should listen for incoming requests
  #[default(8536)]
  pub port: u16,
  /// Whether the site is available over TLS. Needs to be true for federation to work.
  #[default(true)]
  pub tls_enabled: bool,
  /// Set the URL for opentelemetry exports. If you do not have an opentelemetry collector, do not
  /// set this option
  #[doku(skip)]
  pub opentelemetry_url: Option<Url>,
  pub federation: FederationWorkerConfig,
  // Prometheus configuration.
  #[doku(example = "Some(Default::default())")]
  pub prometheus: Option<PrometheusConfig>,
  /// Sets a response Access-Control-Allow-Origin CORS header. Can also be set via environment:
  /// `LEMMY_CORS_ORIGIN=example.org,site.com`
  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin
  #[doku(example = "lemmy.tld")]
  cors_origin: Vec<String>,
}

impl Settings {
  pub fn cors_origin(&self) -> Vec<String> {
    env::var("LEMMY_CORS_ORIGIN")
      .ok()
      .map(|e| e.split(',').map(ToString::to_string).collect())
      .unwrap_or(self.cors_origin.clone())
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PictrsConfig {
  /// Address where pictrs is available (for image hosting)
  #[default(pictrs_placeholder_url())]
  #[doku(example = "http://localhost:8080")]
  pub url: Url,

  /// Set a custom pictrs API key. ( Required for deleting images )
  pub api_key: Option<String>,

  /// Specifies how to handle remote images, so that users don't have to connect directly to remote
  /// servers.
  #[default(PictrsImageMode::ProxyAllImages)]
  pub image_mode: PictrsImageMode,

  /// Allows bypassing proxy for specific image hosts when using ProxyAllImages.
  ///
  /// imgur.com is bypassed by default to avoid rate limit errors. When specifying any bypass
  /// in the config, this default is ignored and you need to list imgur explicitly. To proxy imgur
  /// requests, specify a noop bypass list, eg `proxy_bypass_domains ["example.org"]`.
  #[default(vec!["i.imgur.com".to_string()])]
  #[doku(example = "i.imgur.com")]
  pub proxy_bypass_domains: Vec<String>,

  /// Timeout for uploading images to pictrs (in seconds)
  #[default(30)]
  pub upload_timeout: u64,

  /// Resize post thumbnails to this maximum width/height.
  #[default(512)]
  pub max_thumbnail_size: u32,

  /// Maximum size for user avatar, community icon and site icon.
  #[default(512)]
  pub max_avatar_size: u32,

  /// Maximum size for user, community and site banner. Larger images are downscaled to fit
  /// into a square of this size.
  #[default(1024)]
  pub max_banner_size: u32,

  /// Prevent users from uploading images for posts or embedding in markdown. Avatars, icons and
  /// banners can still be uploaded.
  #[default(false)]
  pub image_upload_disabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, Document, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum PictrsImageMode {
  /// Leave images unchanged, don't generate any local thumbnails for post urls. Instead the
  /// Opengraph image is directly returned as thumbnail
  None,
  /// Generate thumbnails for external post urls and store them persistently in pict-rs. This
  /// ensures that they can be reliably retrieved and can be resized using pict-rs APIs. However
  /// it also increases storage usage.
  ///
  /// This behaviour matches Lemmy 0.18.
  StoreLinkPreviews,
  /// If enabled, all images from remote domains are rewritten to pass through
  /// `/api/v4/image/proxy`, including embedded images in markdown. Images are stored temporarily
  /// in pict-rs for caching. This improves privacy as users don't expose their IP to untrusted
  /// servers, and decreases load on other servers. However it increases bandwidth use for the
  /// local server.
  ///
  /// Requires pict-rs 0.5
  #[default]
  ProxyAllImages,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct DatabaseConfig {
  /// Configure the database by specifying URI pointing to a postgres instance
  ///
  /// This example uses peer authentication to obviate the need for creating,
  /// configuring, and managing passwords.
  ///
  /// For an explanation of how to use connection URIs, see [here][0] in
  /// PostgreSQL's documentation.
  ///
  /// [0]: https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6
  #[default("postgres://lemmy:password@localhost:5432/lemmy")]
  #[doku(example = "postgresql:///lemmy?user=lemmy&host=/var/run/postgresql")]
  pub(crate) connection: String,

  /// Maximum number of active sql connections
  #[default(30)]
  pub pool_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Document, SmartDefault)]
#[serde(deny_unknown_fields)]
pub struct EmailConfig {
  /// Hostname and port of the smtp server
  #[doku(example = "localhost:25")]
  pub smtp_server: String,
  /// Login name for smtp server
  pub smtp_login: Option<String>,
  /// Password to login to the smtp server
  smtp_password: Option<String>,
  #[doku(example = "noreply@example.com")]
  /// Address to send emails from, eg "noreply@your-instance.com"
  pub smtp_from_address: String,
  /// Whether or not smtp connections should use tls. Can be none, tls, or starttls
  #[default("none")]
  #[doku(example = "none")]
  pub tls_type: String,
}

impl EmailConfig {
  pub fn smtp_password(&self) -> Option<String> {
    std::env::var("LEMMY_SMTP_PASSWORD")
      .ok()
      .or(self.smtp_password.clone())
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, Document)]
#[serde(deny_unknown_fields)]
pub struct SetupConfig {
  /// Username for the admin user
  #[doku(example = "admin")]
  pub admin_username: String,
  /// Password for the admin user. It must be between 10 and 60 characters.
  #[doku(example = "tf6HHDS4RolWfFhk4Rq9")]
  pub admin_password: String,
  /// Name of the site, can be changed later. Maximum 20 characters.
  #[doku(example = "My Lemmy Instance")]
  pub site_name: String,
  /// Email for the admin user (optional, can be omitted and set later through the website)
  #[doku(example = "user@example.com")]
  pub admin_email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(deny_unknown_fields)]
pub struct PrometheusConfig {
  // Address that the Prometheus metrics will be served on.
  #[default(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
  #[doku(example = "127.0.0.1")]
  pub bind: IpAddr,
  // Port that the Prometheus metrics will be served on.
  #[default(10002)]
  #[doku(example = "10002")]
  pub port: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
// named federation"worker"config to disambiguate from the activitypub library configuration
pub struct FederationWorkerConfig {
  /// Limit to the number of concurrent outgoing federation requests per target instance.
  /// Set this to a higher value than 1 (e.g. 6) only if you have a huge instance (>10 activities
  /// per second) and if a receiving instance is not keeping up.
  #[default(1)]
  pub concurrent_sends_per_instance: i8,
}
