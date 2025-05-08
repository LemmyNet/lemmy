use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  post::{CreatePostLike, PostResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_bot_account, check_community_user_action, check_local_vote_mode},
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::post::{PostActions, PostLikeForm, PostReadForm},
  traits::{Likeable, Readable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn like_post(
  data: Json<CreatePostLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;

  check_local_vote_mode(
    data.score,
    PostOrCommentId::Post(post_id),
    &local_site,
    local_user_view.person.id,
    &mut context.pool(),
  )
  .await?;
  check_bot_account(&local_user_view.person)?;

  // Check for a community ban
  let post = PostView::read(&mut context.pool(), post_id, None, local_instance_id, false).await?;

  check_community_user_action(&local_user_view, &post.community, &mut context.pool()).await?;

  let mut like_form = PostLikeForm::new(data.post_id, local_user_view.person.id, data.score);

  // Remove any likes first
  let person_id = local_user_view.person.id;

  PostActions::remove_like(&mut context.pool(), person_id, post_id).await?;

  // Only add the like if the score isnt 0
  let do_add =
    like_form.like_score != 0 && (like_form.like_score == 1 || like_form.like_score == -1);
  if do_add {
    like_form = plugin_hook_before("before_post_vote", like_form).await?;
    let like = PostActions::like(&mut context.pool(), &like_form).await?;
    plugin_hook_after("after_post_vote", &like)?;
  }

  // Mark Post Read
  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      object_id: post.post.ap_id,
      actor: local_user_view.person.clone(),
      community: post.community.clone(),
      score: data.score,
    },
    &context,
  )?;

  build_post_response(context.deref(), post.community.id, local_user_view, post_id).await
}
