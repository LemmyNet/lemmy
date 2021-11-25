use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  get_local_user_view_from_jwt,
  is_admin,
  site::*,
  site_description_length_check,
};
use lemmy_db_schema::{
  diesel_option_overwrite,
  diesel_option_overwrite_to_url,
  source::site::{Site, SiteForm},
  traits::Crud,
};
use lemmy_db_views::site_view::SiteView;
use lemmy_utils::{
  utils::{check_slurs, check_slurs_opt},
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::LemmyContext;

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

    let read_site = Site::read_simple;
    if blocking(context.pool(), read_site).await?.is_ok() {
      return Err(LemmyError::from_message("site_already_exists".into()));
    };

    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    check_slurs(&data.name, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let sidebar = diesel_option_overwrite(&data.sidebar);
    let description = diesel_option_overwrite(&data.description);
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    if let Some(Some(desc)) = &description {
      site_description_length_check(desc)?;
    }

    let site_form = SiteForm {
      name: data.name.to_owned(),
      sidebar,
      description,
      icon,
      banner,
      creator_id: local_user_view.person.id,
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
      updated: None,
      community_creation_admin_only: data.community_creation_admin_only,
    };

    let create_site = move |conn: &'_ _| Site::create(conn, &site_form);
    if blocking(context.pool(), create_site).await?.is_err() {
      return Err(LemmyError::from_message("site_already_exists".into()));
    }

    let site_view = blocking(context.pool(), SiteView::read).await??;

    Ok(SiteResponse { site_view })
  }
}
