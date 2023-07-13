use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetSiteResponse, LeaveAdmin},
  utils::{is_admin, local_user_view_from_jwt},
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
use lemmy_db_views::structs::{CustomEmojiView, SiteView};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType},
  version,
};

#[async_trait::async_trait(?Send)]
impl Perform for LeaveAdmin {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<GetSiteResponse, LemmyError> {
    let data: &LeaveAdmin = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    is_admin(&local_user_view)?;

    // Make sure there isn't just one admin (so if one leaves, there will still be one left)
    let admins = PersonView::admins(&mut context.pool()).await?;
    if admins.len() == 1 {
      return Err(LemmyErrorType::CannotLeaveAdmin)?;
    }

    let person_id = local_user_view.person.id;
    Person::update(
      &mut context.pool(),
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

    ModAdd::create(&mut context.pool(), &form).await?;

    // Reread site and admins
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let admins = PersonView::admins(&mut context.pool()).await?;

    let all_languages = Language::read_all(&mut context.pool()).await?;
    let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
    let taglines = Tagline::get_all(&mut context.pool(), site_view.local_site.id).await?;
    let custom_emojis =
      CustomEmojiView::get_all(&mut context.pool(), site_view.local_site.id).await?;

    Ok(GetSiteResponse {
      site_view,
      admins,
      version: version::VERSION.to_string(),
      my_user: None,
      all_languages,
      discussion_languages,
      taglines,
      custom_emojis,
    })
  }
}
