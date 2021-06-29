use crate::inbox::new_inbox_routing::{Activity, SharedInboxActivities};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_apub_lib::{ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

pub async fn shared_inbox(
  _request: HttpRequest,
  input: web::Json<Activity<SharedInboxActivities>>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  activity.inner.verify(&context).await?;
  let request_counter = &mut 0;
  activity.inner.receive(&context, request_counter).await?;
  todo!()
}
