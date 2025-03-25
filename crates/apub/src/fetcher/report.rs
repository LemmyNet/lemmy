use crate::{
  fetcher::post_or_comment::{PageOrNote, PostOrComment},
  objects::{comment::ApubComment, community::ApubCommunity, post::ApubPost},
  protocol::objects::group::Group,
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use reqwest::Url;
use serde::Deserialize;

/// The types of ActivityPub objects that reports can be created for.
pub(crate) type ReportableObjects = Either<PostOrComment, ApubCommunity>;
