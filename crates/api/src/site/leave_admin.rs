use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetSiteResponse, LeaveAdmin},
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  source::{
    actor_language::SiteLanguage,
    language::Language,
    moderator::{ModAdd, ModAddForm},
    person::{Person, PersonUpdateForm},
    tagline::Tagline,
  },
  traits::Crud,
};
use lemmy_db_views::structs::SiteView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{error::LemmyError, version, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for LeaveAdmin {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &LeaveAdmin = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    // Make sure there isn't just one admin (so if one leaves, there will still be one left)
    let admins = PersonView::admins(context.pool()).await?;
    if admins.len() == 1 {
      return Err(LemmyError::from_message("cannot_leave_admin"));
    }

    let person_id = local_user_view.person.id;
    Person::update(
      context.pool(),
      person_id,
      &PersonUpdateForm::builder().admin(Some(false)).build(),
    )
    .await?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: person_id,
      other_person_id: person_id,
      removed: Some(true),
    };

    ModAdd::create(context.pool(), &form).await?;

    // Reread site and admins
    let site_view = SiteView::read_local(context.pool()).await?;
    let admins = PersonView::admins(context.pool()).await?;

    let all_languages = Language::read_all(context.pool()).await?;
    let discussion_languages = SiteLanguage::read_local(context.pool()).await?;
    let taglines_res = Tagline::get_all(context.pool(), site_view.local_site.id).await?;
    let taglines = taglines_res.is_empty().then_some(taglines_res);

    Ok(GetSiteResponse {
      site_view,
      admins,
      online: 0,
      version: version::VERSION.to_string(),
      my_user: None,
      federated_instances: None,
      all_languages,
      discussion_languages,
      taglines,
    })
  }
}
