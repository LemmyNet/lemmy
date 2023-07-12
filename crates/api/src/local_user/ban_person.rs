use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  utils::{is_admin, local_user_view_from_jwt, remove_user_data},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModBan, ModBanForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{time::naive_from_unix, validation::is_valid_body_field},
};

#[async_trait::async_trait(?Send)]
impl Perform for BanPerson {
  type Response = BanPersonResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<BanPersonResponse, LemmyError> {
    let data: &BanPerson = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    is_valid_body_field(&data.reason, false)?;

    let ban = data.ban;
    let banned_person_id = data.person_id;
    let expires = data.expires.map(naive_from_unix);

    let person = Person::update(
      &mut context.pool(),
      banned_person_id,
      &PersonUpdateForm::builder()
        .banned(Some(ban))
        .ban_expires(Some(expires))
        .build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

    // Remove their data if that's desired
    let remove_data = data.remove_data.unwrap_or(false);
    if remove_data {
      remove_user_data(
        person.id,
        &mut context.pool(),
        context.settings(),
        context.client(),
      )
      .await?;
    }

    // Mod tables
    let form = ModBanForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      reason: data.reason.clone(),
      banned: Some(data.ban),
      expires,
    };

    ModBan::create(&mut context.pool(), &form).await?;

    let person_id = data.person_id;
    let person_view = PersonView::read(&mut context.pool(), person_id).await?;

    Ok(BanPersonResponse {
      person_view,
      banned: data.ban,
    })
  }
}
