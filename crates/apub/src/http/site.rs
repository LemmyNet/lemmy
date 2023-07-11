use crate::{
  activity_lists::SiteInboxActivities,
  http::create_apub_response,
  objects::{instance::ApubSite, person::ApubPerson},
  protocol::collections::empty_outbox::EmptyOutbox,
};
use activitypub_federation::{
  actix_web::inbox::receive_activity,
  config::Data,
  protocol::context::WithContext,
  traits::Object,
};
use actix_web::{web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_views::structs::SiteView;
use lemmy_utils::error::LemmyError;
use url::Url;

pub(crate) async fn get_apub_site_http(
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let site: ApubSite = SiteView::read_local(&mut context.pool()).await?.site.into();

  let apub = site.into_json(&context).await?;
  create_apub_response(&apub)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_site_outbox(
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let outbox_id = format!(
    "{}/site_outbox",
    context.settings().get_protocol_and_hostname()
  );
  let outbox = EmptyOutbox::new(Url::parse(&outbox_id)?)?;
  create_apub_response(&outbox)
}

#[tracing::instrument(skip_all)]
pub async fn get_apub_site_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_activity::<WithContext<SiteInboxActivities>, ApubPerson, LemmyContext>(
    request, body, &data,
  )
  .await
}
