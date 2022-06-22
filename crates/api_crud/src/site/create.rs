use crate::PerformCrud;
use activitypub_federation::core::signatures::generate_actor_keypair;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{CreateSite, SiteResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin, site_description_length_check},
};
use lemmy_apub::generate_site_inbox_url;
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::site::{Site, SiteForm},
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt},
  ConnectionId,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateSite {
  type Response = SiteResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<SiteResponse, LemmyError> {
    let data: &CreateSite = self;

    let read_site = Site::read_local_site;
    if blocking(context.pool(), read_site).await?.is_ok() {
      return Err(LemmyError::from_message("site_already_exists"));
    };

    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let sidebar = diesel_option_overwrite(&data.sidebar);
    let description = diesel_option_overwrite(&data.description);
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    check_slurs(&data.name, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    if let Some(Some(desc)) = &description {
      site_description_length_check(desc)?;
    }

    let actor_id: DbUrl = Url::parse(&context.settings().get_protocol_and_hostname())?.into();
    let inbox_url = Some(generate_site_inbox_url(&actor_id)?);
    let keypair = generate_actor_keypair()?;
    let site_form = SiteForm {
      name: data.name.to_owned(),
      sidebar,
      description,
      icon,
      banner,
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
      community_creation_admin_only: data.community_creation_admin_only,
      actor_id: Some(actor_id),
      last_refreshed_at: Some(naive_now()),
      inbox_url,
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      default_theme: data.default_theme.clone(),
      default_post_listing_type: data.default_post_listing_type.clone(),
      ..SiteForm::default()
    };

    let create_site = move |conn: &'_ _| Site::create(conn, &site_form);
    blocking(context.pool(), create_site)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "site_already_exists"))?;

    let site_view = blocking(context.pool(), SiteView::read_local).await??;

    Ok(SiteResponse { site_view })
  }
}
