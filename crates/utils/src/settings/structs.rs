use super::pictrs_placeholder_url;
use doku::Document;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{
  collections::BTreeMap,
  env,
  net::{IpAddr, Ipv4Addr},
};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
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
  /// Print logs in JSON format. You can also disable ANSI colors in logs with env var `NO_COLOR`.
  pub json_logging: bool,
  /// Data for loading Lemmy plugins
  pub plugins: Vec<PluginSettings>,
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
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct DatabaseConfig {
  /// Configure the database by specifying URI pointing to a postgres instance. This parameter can
  /// also be set by environment variable `LEMMY_DATABASE_URL`.
  ///
  /// For an explanation of how to use connection URIs, see PostgreSQL's documentation:
  /// https://www.postgresql.org/docs/current/libpq-connect.html#id-1.7.3.8.3.6
  #[default("postgres://lemmy:password@localhost:5432/lemmy")]
  #[doku(example = "postgresql:///lemmy?user=lemmy&host=/var/run/postgresql")]
  pub(crate) connection: String,

  /// Maximum number of active sql connections
  ///
  /// A high value here can result in errors "could not resize shared memory segment". In this case
  /// it is necessary to increase shared memory size in Docker: https://stackoverflow.com/a/56754077
  #[default(30)]
  pub pool_size: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Document, SmartDefault)]
#[serde(default, deny_unknown_fields)]
pub struct EmailConfig {
  /// https://docs.rs/lettre/0.11.14/lettre/transport/smtp/struct.AsyncSmtpTransport.html#method.from_url
  #[default("smtp://localhost:25")]
  #[doku(example = "smtps://user:pass@hostname:port")]
  pub connection: String,
  /// Address to send emails from, eg "noreply@your-instance.com"
  #[doku(example = "noreply@example.com")]
  pub smtp_from_address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
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
  /// On first start Lemmy fetches the 50 most active communities from one of these instances,
  /// to provide some initial data. It tries the first list entry, and if it fails uses subsequent
  /// instances as fallback.
  /// Leave this empty to disable community bootstrap.
  /// TODO: remove voyager.lemmy.ml from defaults once Lemmy 1.0 is deployed to production
  /// instances.
  #[default(vec!["lemmy.ml".to_string(),"lemmy.world".to_string(),"lemmy.zip".to_string(),"voyager.lemmy.ml".to_string()])]
  pub bootstrap_instances: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PrometheusConfig {
  // Address that the Prometheus metrics will be served on.
  #[default(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
  #[doku(example = "127.0.0.1")]
  pub bind: IpAddr,
  // Port that the Prometheus metrics will be served on.
  #[default(10002)]
  #[doku(example = "10002")]
  pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
// named federation"worker"config to disambiguate from the activitypub library configuration
pub struct FederationWorkerConfig {
  /// Limit to the number of concurrent outgoing federation requests per target instance.
  /// Set this to a higher value than 1 (e.g. 6) only if you have a huge instance (>10 activities
  /// per second) and if a receiving instance is not keeping up.
  #[default(1)]
  pub concurrent_sends_per_instance: i8,
}

/// See the extism docs for more details: https://extism.org/docs/concepts/manifest
#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default, deny_unknown_fields)]
pub struct PluginSettings {
  /// Where to load the .wasm file from, can be a local file path or URL
  #[doku(
    example = "https://github.com/LemmyNet/lemmy-plugins/releases/download/0.1.1/go_replace_words.wasm"
  )]
  pub file: String,
  /// SHA256 hash of the .wasm file
  #[doku(example = "37cdc01a3ff26eef578b668c6cc57fc06649deddb3a92cb6bae8e79b4e60fe12")]
  pub hash: Option<String>,
  /// Which websites the plugin may connect to
  #[serde(default)]
  #[doku(example = "lemmy.ml")]
  pub allowed_hosts: Option<Vec<String>>,
  /// Configuration options for the plugin
  #[serde(default)]
  pub config: BTreeMap<String, String>,
}
