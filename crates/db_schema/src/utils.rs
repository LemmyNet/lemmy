pub mod queries;

use chrono::TimeDelta;
use diesel::{
  Expression,
  IntoSql,
  dsl,
  helper_types::AsExprOf,
  pg::{Pg, data_types::PgInterval},
  query_builder::{Query, QueryFragment},
  query_dsl::methods::LimitDsl,
  result::{
    ConnectionError,
    ConnectionResult,
    Error::{self as DieselError, QueryBuilderError},
  },
  sql_types::{self, Timestamptz},
};
use diesel_async::{
  AsyncConnection,
  pg::AsyncPgConnection,
  pooled_connection::{
    AsyncDieselConnectionManager,
    ManagerConfig,
    deadpool::{Hook, HookError, Object as PooledConnection, Pool},
  },
  scoped_futures::ScopedBoxFuture,
};
use futures_util::{FutureExt, future::BoxFuture};
use i_love_jesus::{CursorKey, PaginatedQueryBuilder, SortDirection};
use lemmy_diesel_utils::dburl::DbUrl;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::{SETTINGS, structs::Settings},
  utils::validation::clean_url,
};
use rustls::{
  ClientConfig,
  DigitallySignedStruct,
  SignatureScheme,
  client::danger::{
    DangerousClientConfigBuilder,
    HandshakeSignatureValid,
    ServerCertVerified,
    ServerCertVerifier,
  },
  crypto::{self, verify_tls12_signature, verify_tls13_signature},
  pki_types::{CertificateDer, ServerName, UnixTime},
};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::Duration,
};
use tracing::error;
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 20;
pub const FETCH_LIMIT_MAX: usize = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: TimeDelta = TimeDelta::days(31);
pub const RANK_DEFAULT: f32 = 0.0001;
pub const DELETED_REPLACEMENT_TEXT: &str = "*Permanently Deleted*";

pub fn limit_fetch(limit: Option<i64>) -> LemmyResult<i64> {
  Ok(match limit {
    Some(limit) => limit_fetch_check(limit)?,
    None => FETCH_LIMIT_DEFAULT,
  })
}

pub fn limit_fetch_check(limit: i64) -> LemmyResult<i64> {
  if !(1..=FETCH_LIMIT_MAX.try_into()?).contains(&limit) {
    Err(LemmyErrorType::InvalidFetchLimit.into())
  } else {
    Ok(limit)
  }
}

pub(crate) fn format_actor_url(
  name: &str,
  domain: &str,
  prefix: char,
  settings: &Settings,
) -> LemmyResult<Url> {
  let local_protocol_and_hostname = settings.get_protocol_and_hostname();
  let local_hostname = &settings.hostname;
  let url = if domain != local_hostname {
    format!("{local_protocol_and_hostname}/{prefix}/{name}@{domain}",)
  } else {
    format!("{local_protocol_and_hostname}/{prefix}/{name}")
  };
  Ok(Url::parse(&url)?)
}
