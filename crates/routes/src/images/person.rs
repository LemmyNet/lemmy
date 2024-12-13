use super::utils::{delete_old_image, do_upload_image, UploadType};
use actix_web::{self, web::*, HttpRequest};
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::{
  source::person::{Person, PersonUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;
use url::Url;

pub async fn upload_avatar(
  req: HttpRequest,
  body: Payload,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let image = do_upload_image(req, body, UploadType::Avatar, &local_user_view, &context).await?;

  delete_old_image(&local_user_view.person.avatar, &context).await?;

  let avatar = format!(
    "{}/api/v4/image/{}",
    context.settings().get_protocol_and_hostname(),
    image.file
  );
  let avatar = Some(Some(Url::parse(&avatar)?.into()));
  let person_form = PersonUpdateForm {
    avatar,
    ..Default::default()
  };

  Person::update(&mut context.pool(), local_user_view.person.id, &person_form).await?;

  Ok(Json(SuccessResponse::default()))
}
