use crate::{
  fetcher::post_or_comment::{PageOrNote, PostOrComment},
  objects::community::ApubCommunity,
  protocol::objects::group::Group,
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use either::Either;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use reqwest::Url;
use serde::Deserialize;

/// The types of ActivityPub objects that reports can be created for.
pub(crate) type ReportableObjects = Either<PostOrComment, ApubCommunity>;

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ReportableKinds {
  PageOrNote(PageOrNote),
  Group(Box<Group>),
}
