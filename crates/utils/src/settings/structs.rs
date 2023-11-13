use doku::Document;
use serde::{Deserialize, Serialize};
use std::{
  env,
  net::{IpAddr, Ipv4Addr},
};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct Settings {
  /// settings related to the postgresql database
  #[default(Default::default())]
  pub database: DatabaseConfig,
  /// Settings related to activitypub federation
  /// Pictrs image server configuration.
  #[default(Some(Default::default()))]
  pub(crate) pictrs: Option<PictrsConfig>,
  /// Email sending configuration. All options except login/password are mandatory
  #[default(None)]
  #[doku(example = "Some(Default::default())")]
  pub email: Option<EmailConfig>,
  /// Parameters for automatic configuration of new instance (only used at first start)
  #[default(None)]
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
  /// Set the URL for opentelemetry exports. If you do not have an opentelemetry collector, do not set this option
  #[default(None)]
  #[doku(skip)]
  pub opentelemetry_url: Option<Url>,
  /// The number of activitypub federation workers that can be in-flight concurrently
  #[default(0)]
  pub worker_count: usize,
  /// The number of activitypub federation retry workers that can be in-flight concurrently
  #[default(0)]
  pub retry_count: usize,
  // Prometheus configuration.
  #[default(None)]
  #[doku(example = "Some(Default::default())")]
  pub prometheus: Option<PrometheusConfig>,
  /// Sets a response Access-Control-Allow-Origin CORS header
  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin
  #[default(None)]
  #[doku(example = "*")]
  cors_origin: Option<String>,
}

impl Settings {
  pub fn cors_origin(&self) -> Option<String> {
    env::var("LEMMY_CORS_ORIGIN")
      .ok()
      .or(self.cors_origin.clone())
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PictrsConfig {
  /// Address where pictrs is available (for image hosting)
  #[default(Url::parse("http://localhost:8080").expect("parse pictrs url"))]
  #[doku(example = "http://localhost:8080")]
  pub url: Url,

  /// Set a custom pictrs API key. ( Required for deleting images )
  #[default(None)]
  pub api_key: Option<String>,

  /// By default the thumbnails for external links are stored in pict-rs. This ensures that they
  /// can be reliably retrieved and can be resized using pict-rs APIs. However it also increases
  /// storage usage. In case this is disabled, the Opengraph image is directly returned as
  /// thumbnail.
  ///
  /// In some countries it is forbidden to copy preview images from newspaper articles and only
  /// hotlinking is allowed. If that is the case for your instance, make sure that this setting is
  /// disabled.
  #[default(true)]
  pub cache_external_link_previews: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct DatabaseConfig {
  #[serde(flatten, default)]
  pub(crate) connection: DatabaseConnection,

  /// Maximum number of active sql connections
  #[default(95)]
  pub pool_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(untagged)]
pub enum DatabaseConnection {
  /// Configure the database by specifying a URI
  ///
  /// This is the preferred method to specify database connection details since
  /// it is the most flexible.
  Uri {
    /// Connection URI pointing to a postgres instance
    ///
    /// This example uses peer authentication to obviate the need for creating,
    /// configuring, and managing passwords.
    ///
    /// For an explanation of how to use connection URIs, see [here][0] in
    /// PostgreSQL's documentation.
    ///
    /// [0]: https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6
    #[doku(example = "postgresql:///lemmy?user=lemmy&host=/var/run/postgresql")]
    uri: String,
  },

  /// Configure the database by specifying parts of a URI
  ///
  /// Note that specifying the `uri` field should be preferred since it provides
  /// greater control over how the connection is made. This merely exists for
  /// backwards-compatibility.
  #[default]
  Parts(DatabaseConnectionParts),
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct DatabaseConnectionParts {
  /// Username to connect to postgres
  #[default("lemmy")]
  pub(super) user: String,
  /// Password to connect to postgres
  #[default("password")]
  pub(super) password: String,
  #[default("localhost")]
  /// Host where postgres is running
  pub(super) host: String,
  /// Port where postgres can be accessed
  #[default(5432)]
  pub(super) port: i32,
  /// Name of the postgres database for lemmy
  #[default("lemmy")]
  pub(super) database: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
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
  #[default(None)]
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
