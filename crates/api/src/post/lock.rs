use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
    build_response::build_post_response,
    context::LemmyContext,
    post::{LockPost, PostResponse},
    send_activity::{ActivityChannel, SendActivityData},
    utils::{
        check_community_ban, check_community_deleted_or_removed, is_mod_or_admin,
        local_user_view_from_jwt,
    },
};
use lemmy_db_schema::{
    source::{
        moderator::{ModLockPost, ModLockPostForm},
        post::{Post, PostUpdateForm},
    },
    traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn lock_post(
    data: Json<LockPost>,
    context: Data<LemmyContext>,
) -> Result<Json<PostResponse>, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&mut context.pool(), post_id).await?;

    check_community_ban(
        local_user_view.person.id,
        orig_post.community_id,
        &mut context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, &mut context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(
        &mut context.pool(),
        local_user_view.person.id,
        orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    let post = Post::update(
        &mut context.pool(),
        post_id,
        &PostUpdateForm {
            locked: Some(locked),
            ..Default::default()
        },
    )
    .await?;

    // Mod tables
    let form = ModLockPostForm {
        mod_person_id: local_user_view.person.id,
        post_id: data.post_id,
        locked: Some(locked),
    };
    ModLockPost::create(&mut context.pool(), &form).await?;

    let person_id = local_user_view.person.id;
    ActivityChannel::submit_activity(
        SendActivityData::LockPost(post, local_user_view.person, data.locked),
        &context,
    )
    .await?;

    build_post_response(&context, orig_post.community_id, person_id, post_id).await
}
