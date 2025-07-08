use crate::protocol::collections::empty_outbox::EmptyOutbox;
use activitypub_federation::{
  actix_web::response::create_http_response,
  config::Data,
  traits::Object,
};
use actix_web::HttpResponse;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::instance::ApubSite;
use lemmy_db_schema::source::site::Site;
use lemmy_utils::{error::LemmyResult, FEDERATION_CONTEXT};
use url::Url;

pub(crate) async fn get_apub_site_http(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let site: ApubSite = Site::read_local(&mut context.pool()).await?.into();

  site.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_site_outbox(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let outbox_id = format!(
    "{}/site_outbox",
    context.settings().get_protocol_and_hostname()
  );
  let outbox = EmptyOutbox::new(Url::parse(&outbox_id)?)?;
  Ok(create_http_response(outbox, &FEDERATION_CONTEXT)?)
}
