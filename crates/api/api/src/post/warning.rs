use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  notify::notify_mod_action,
  utils::check_community_mod_action,
};
use lemmy_db_schema::source::modlog::{Modlog, ModlogInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  PostView,
  api::{CreatePostWarning, PostResponse},
};
use lemmy_utils::error::LemmyResult;

/// Creates a warning against a post and notifies the user
pub async fn create_post_warning(
  Json(data): Json<CreatePostWarning>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let local_instance_id = local_user_view.person.instance_id;

  let orig_post = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  check_community_mod_action(
    &local_user_view,
    &orig_post.community,
    false,
    &mut context.pool(),
  )
  .await?;

  // Mod tables
  let form = ModlogInsertForm::mod_create_post_warning(
    local_user_view.person.id,
    &orig_post.post,
    &data.reason,
  );
  let action = Modlog::create(&mut context.pool(), &[form]).await?;
  notify_mod_action(action, &context);

  // TODO federate activity

  build_post_response(&context, orig_post.community.id, local_user_view, post_id).await
}
