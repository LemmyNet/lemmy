use std::{any::Any, cell::{Cell, RefCell}, future::Future, ops::ControlFlow, pin::Pin, sync::{Arc, Mutex}, task::Poll};

use crate::{
  objects::{
    comment::ApubComment,
    community::ApubCommunity,
    person::ApubPerson,
    post::ApubPost,
    PostOrComment,
  },
  protocol::page::Attachment,
  utils::{
    mentions::MentionOrValue,
    protocol::{InCommunity, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::object::NoteType,
  protocol::{
    helpers::{deserialize_one_or_many, deserialize_skip_error},
    values::MediaTypeMarkdownOrHtml,
  },
};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  MAX_COMMENT_DEPTH_LIMIT,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  pub(crate) r#type: NoteType,
  pub id: ObjectId<ApubComment>,
  pub attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub cc: Vec<Url>,
  pub(crate) content: String,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,

  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
  #[serde(default)]
  pub tag: Vec<MentionOrValue>,
  // lemmy extension
  pub distinguished: Option<bool>,
  pub(crate) language: Option<LanguageTag>,
  #[serde(default)]
  pub(crate) attachment: Vec<Attachment>,
}

impl Note {
  pub async fn get_parents(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPost, Option<ApubComment>)> {
    // We use recursion here to fetch the entire comment chain up to the top-level parent. This is
    // necessary because we need to know the post and parent comment in order to insert a new
    // comment. However it can also lead to stack overflow when fetching many comments recursively.
    // To avoid this we check the request count against max comment depth, which based on testing
    // can be handled without risking stack overflow. This is not a perfect solution, because in
    // some cases we have to fetch user profiles too, and reach the limit after only 25 comments
    // or so.
    // A cleaner solution would be converting the recursion into a loop, but that is tricky.
    if context.request_count() > MAX_COMMENT_DEPTH_LIMIT.try_into()? {
      Err(LemmyErrorType::MaxCommentDepthReached)?;
    }
    let parent = self.in_reply_to.dereference(context).await?;
    match parent {
      PostOrComment::Left(p) => Ok((p.clone(), None)),
      PostOrComment::Right(c) => {
        let post_id = c.post_id;
        let post = Post::read(&mut context.pool(), post_id).await?;
        Ok((post.into(), Some(c.clone())))
      }
    }
  }
}

impl InCommunity for Note {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let (post, _) = self.get_parents(context).await?;
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    Ok(community.into())
  }
}

struct Iterative<T: Future> {
  parent: Option<Box<Iterative<T>>>,
  non_current_children: Vec<Iterative<T>>,
  state: ControlFlow<Poll<T::Output>, Pin<Box<T>>>,
}

thread_local! {
  // One for each `T` in `Iterative<T>`
  static CURRENT_ITERATIVES: RefCell<Vec<Box<dyn Any>>> = const {
    RefCell::new(Vec::new())
  };
}

impl<T: Future + 'static> Future for Iterative<T> where T::Output: Unpin + 'static {
  type Output = T::Output;
  fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
    match &mut self.get_mut().state {
      ControlFlow::Break(output) => std::mem::replace(output, Poll::Pending),
      ControlFlow::Continue(future) => {
        let mut current_iteratives = CURRENT_ITERATIVES.();
        if let Some(iterative) = current_iteratives.iter_mut().find_map(|i| i.downcast_mut::<Self>()) {
          // todo
        }

        CURRENT_ITERATIVES.set(current_iteratives);
        Poll::Pending
      }
    }
  }
}

/*

x = poll future a1 {
  y = poll future b {
    z = poll future a2 {
      ready 9
    }
    ready
  }
  ready 8
}

x = poll future a1 {
  poll future a1 {
    y = poll future b {
      z = poll future a2 {
        enqueue future a2
        pending
      }
      pending
    }
    pending
  }
  poll future a2 {
    result a2 = 9
  }
  pending
}
x = poll future a1 {
  poll future a1 {
    y = poll future b {
      z = 9
      ready
    }
    ready 8
  }
}

[a] b c
a [b] c
a [b 1] c

*/
