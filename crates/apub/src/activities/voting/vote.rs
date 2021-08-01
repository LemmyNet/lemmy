use crate::{
  activities::{
    verify_activity,
    verify_person_in_community,
    voting::{vote_comment, vote_post},
  },
  fetcher::{
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  PostOrComment,
};
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VoteType {
  Like,
  Dislike,
}

impl VoteType {
  pub(crate) fn score(&self) -> i16 {
    match self {
      VoteType::Like => 1,
      VoteType::Dislike => -1,
    }
  }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  to: PublicUrl,
  pub(in crate::activities::voting) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: VoteType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc[0], context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor =
      get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
    let object =
      get_or_fetch_and_insert_post_or_comment(&self.object, context, request_counter).await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, p.deref(), context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, c.deref(), context).await,
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
