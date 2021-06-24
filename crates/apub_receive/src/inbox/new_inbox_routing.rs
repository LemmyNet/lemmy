use crate::activities_new::{
  comment::create::CreateComment,
  follow::AcceptFollowCommunity,
  private_message::{
    create::CreatePrivateMessage,
    delete::DeletePrivateMessage,
    undo_delete::UndoDeletePrivateMessage,
    update::UpdatePrivateMessage,
  },
};
use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::ReceiveActivity;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

// TODO: would be nice if we could move this to lemmy_apub_lib crate. doing that gives error:
//       "only traits defined in the current crate can be implemented for arbitrary types"
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity<Kind> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  id: Url,

  /// type-specific fields
  #[serde(flatten)]
  pub inner: Kind,

  // unparsed fields
  // todo: can probably remove this field
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl<Kind> Activity<Kind> {
  pub fn id_unchecked(&self) -> &Url {
    &self.id
  }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum PersonAcceptedActivitiesNew {
  AcceptFollowCommunity(AcceptFollowCommunity),
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  CreateComment(CreateComment),
}

// todo: can probably get rid of this?
#[async_trait::async_trait(?Send)]
impl ReceiveActivity for PersonAcceptedActivitiesNew {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.receive(context, request_counter).await
  }
}
