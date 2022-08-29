use doku::Document;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct Settings {
  /// settings related to the postgresql database
  #[default(Default::default())]
  pub database: DatabaseConfig,
  /// rate limits for various user actions, by user ip
  #[default(Some(Default::default()))]
  pub rate_limit: Option<RateLimitConfig>,
  /// Settings related to activitypub federation
  #[default(Default::default())]
  pub federation: FederationConfig,
  /// Pictrs image server configuration.
  #[default(Some(Default::default()))]
  pub(crate) pictrs: Option<PictrsConfig>,
  #[default(Default::default())]
  pub captcha: CaptchaConfig,
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
  #[default(None)]
  #[doku(example = "(\\bThis\\b)|(\\bis\\b)|(\\bsample\\b)")]
  /// A regex list of slurs to block / hide
  pub slur_filter: Option<String>,
  /// Maximum length of local community and user names
  #[default(20)]
  pub actor_name_max_length: usize,

  /// Set the URL for opentelemetry exports. If you do not have an opentelemetry collector, do not set this option
  #[default(None)]
  #[doku(skip)]
  pub opentelemetry_url: Option<Url>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct PictrsConfig {
  /// Address where pictrs is available (for image hosting)
  #[default(Url::parse("http://pictrs:8080").expect("parse pictrs url"))]
  #[doku(example = "http://pictrs:8080")]
  pub url: Url,

  /// Set a custom pictrs API key. ( Required for deleting images )
  #[default(None)]
  pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct CaptchaConfig {
  /// Whether captcha is required for signup
  #[default(false)]
  pub enabled: bool,
  /// Can be easy, medium, or hard
  #[default("medium")]
  pub difficulty: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct DatabaseConfig {
  /// Username to connect to postgres
  #[default("lemmy")]
  pub(super) user: String,
  /// Password to connect to postgres
  #[default("password")]
  pub password: String,
  #[default("localhost")]
  /// Host where postgres is running
  pub host: String,
  /// Port where postgres can be accessed
  #[default(5432)]
  pub(super) port: i32,
  /// Name of the postgres database for lemmy
  #[default("lemmy")]
  pub(super) database: String,
  /// Maximum number of active sql connections
  #[default(5)]
  pub pool_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Document, SmartDefault)]
pub struct EmailConfig {
  /// Hostname and port of the smtp server
  #[doku(example = "localhost:25")]
  pub smtp_server: String,
  /// Login name for smtp server
  pub smtp_login: Option<String>,
  /// Password to login to the smtp server
  pub smtp_password: Option<String>,
  #[doku(example = "noreply@example.com")]
  /// Address to send emails from, eg "noreply@your-instance.com"
  pub smtp_from_address: String,
  /// Whether or not smtp connections should use tls. Can be none, tls, or starttls
  #[default("none")]
  #[doku(example = "none")]
  pub tls_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct FederationConfig {
  /// Whether to enable activitypub federation.
  #[default(false)]
  pub enabled: bool,
  /// Allows and blocks are described here:
  /// https://join-lemmy.org/docs/en/administration/federation_getting_started.html
  ///
  /// list of instances with which federation is allowed
  #[default(None)]
  #[doku(example = "instance1.tld")]
  #[doku(example = "instance2.tld")]
  pub allowed_instances: Option<Vec<String>>,
  /// Instances which we never federate anything with (but previously federated objects are unaffected)
  #[default(None)]
  pub blocked_instances: Option<Vec<String>>,
  /// If true, only federate with instances on the allowlist and block everything else. If false,
  /// use allowlist only for remote communities, and posts/comments in local communities
  /// (meaning remote communities will show content from arbitrary instances).
  #[default(true)]
  pub strict_allowlist: bool,
  /// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object fetch through the search).
  #[default(25)]
  pub http_fetch_retry_limit: i32,
  /// Number of workers for sending outgoing activities. Search logs for "Activity queue stats" to
  /// see information. If "running" number is consistently close to the worker_count, you should
  /// increase it.
  #[default(64)]
  pub worker_count: u64,
  /// Use federation debug mode. Allows connecting to http and localhost urls. Also sends outgoing
  /// activities synchronously for easier testing. Do not use in production.
  #[default(false)]
  pub debug: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
#[serde(default)]
pub struct RateLimitConfig {
  /// Maximum number of messages created in interval
  #[default(180)]
  pub message: i32,
  /// Interval length for message limit, in seconds
  #[default(60)]
  pub message_per_second: i32,
  /// Maximum number of posts created in interval
  #[default(6)]
  pub post: i32,
  /// Interval length for post limit, in seconds
  #[default(600)]
  pub post_per_second: i32,
  /// Maximum number of registrations in interval
  #[default(3)]
  pub register: i32,
  /// Interval length for registration limit, in seconds
  #[default(3600)]
  pub register_per_second: i32,
  /// Maximum number of image uploads in interval
  #[default(6)]
  pub image: i32,
  /// Interval length for image uploads, in seconds
  #[default(3600)]
  pub image_per_second: i32,
  /// Maximum number of comments created in interval
  #[default(6)]
  pub comment: i32,
  /// Interval length for comment limit, in seconds
  #[default(600)]
  pub comment_per_second: i32,
  #[default(60)]
  pub search: i32,
  /// Interval length for search limit, in seconds
  #[default(600)]
  pub search_per_second: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, Document)]
pub struct SetupConfig {
  /// Username for the admin user
  #[doku(example = "admin")]
  pub admin_username: String,
  /// Password for the admin user. It must be at least 10 characters.
  #[doku(example = "tf6HHDS4RolWfFhk4Rq9")]
  pub admin_password: String,
  /// Name of the site (can be changed later)
  #[doku(example = "My Lemmy Instance")]
  pub site_name: String,
  /// Email for the admin user (optional, can be omitted and set later through the website)
  #[doku(example = "user@example.com")]
  #[default(None)]
  pub admin_email: Option<String>,
}
