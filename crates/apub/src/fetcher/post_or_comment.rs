use crate::{
  objects::{comment::ApubComment, post::ApubPost},
  protocol::objects::{note::Note, page::Page},
};
use activitypub_federation::traits::ApubObject;
use chrono::NaiveDateTime;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum PostOrComment {
  Post(Box<ApubPost>),
  Comment(Box<ApubComment>),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PageOrNote {
  Page(Box<Page>),
  Note(Note),
}

#[async_trait::async_trait(?Send)]
impl ApubObject for PostOrComment {
  type DataType = LemmyContext;
  type ApubType = PageOrNote;
  type DbType = ();
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  // TODO: this can probably be implemented using a single sql query
  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    let post = ApubPost::read_from_apub_id(object_id.clone(), data).await?;
    Ok(match post {
      Some(o) => Some(PostOrComment::Post(Box::new(o))),
      None => ApubComment::read_from_apub_id(object_id, data)
        .await?
        .map(|c| PostOrComment::Comment(Box::new(c))),
    })
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    match self {
      PostOrComment::Post(p) => p.delete(data).await,
      PostOrComment::Comment(c) => c.delete(data).await,
    }
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match apub {
      PageOrNote::Page(a) => ApubPost::verify(a, expected_domain, data, request_counter).await,
      PageOrNote::Note(a) => ApubComment::verify(a, expected_domain, data, request_counter).await,
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: PageOrNote,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    Ok(match apub {
      PageOrNote::Page(p) => PostOrComment::Post(Box::new(
        ApubPost::from_apub(*p, context, request_counter).await?,
      )),
      PageOrNote::Note(n) => PostOrComment::Comment(Box::new(
        ApubComment::from_apub(n, context, request_counter).await?,
      )),
    })
  }
}

impl PostOrComment {
  pub(crate) fn ap_id(&self) -> Url {
    match self {
      PostOrComment::Post(p) => p.ap_id.clone(),
      PostOrComment::Comment(c) => c.ap_id.clone(),
    }
    .into()
  }
}
