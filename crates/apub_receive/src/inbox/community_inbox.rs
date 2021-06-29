use crate::inbox::new_inbox_routing::{Activity, SharedInboxActivities};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

/// Handler for all incoming receive to community inboxes.
pub async fn community_inbox(
  _request: HttpRequest,
  _input: web::Json<Activity<Activity<SharedInboxActivities>>>,
  _path: web::Path<String>,
  _context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  todo!()
}
