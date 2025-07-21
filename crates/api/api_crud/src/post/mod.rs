use chrono::{DateTime, TimeZone, Utc};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_action};
use lemmy_db_schema::{
  newtypes::TagId,
  source::{community::Community, post::Post, tag::PostTag},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod delete;
pub mod read;
pub mod remove;
pub mod update;

async fn convert_published_time(
  scheduled_publish_time: Option<i64>,
  local_user_view: &LocalUserView,
  context: &LemmyContext,
) -> LemmyResult<Option<DateTime<Utc>>> {
  const MAX_SCHEDULED_POSTS: i64 = 10;
  if let Some(scheduled_publish_time) = scheduled_publish_time {
    let converted = Utc
      .timestamp_opt(scheduled_publish_time, 0)
      .single()
      .ok_or(LemmyErrorType::InvalidUnixTime)?;
    if converted < Utc::now() {
      Err(LemmyErrorType::PostScheduleTimeMustBeInFuture)?;
    }
    if !local_user_view.local_user.admin {
      let count =
        Post::user_scheduled_post_count(local_user_view.person.id, &mut context.pool()).await?;
      if count >= MAX_SCHEDULED_POSTS {
        Err(LemmyErrorType::TooManyScheduledPosts)?;
      }
    }
    Ok(Some(converted))
  } else {
    Ok(None)
  }
}

async fn update_post_tags(
  context: &LemmyContext,
  post: &Post,
  community: &Community,
  tags: &Option<Vec<TagId>>,
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  let Some(tags) = tags else {
    return Ok(());
  };

  // Check if user is either the post author or a community mod
  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);
  if !is_author {
    check_community_mod_action(local_user_view, community, false, &mut context.pool()).await?;
  }

  PostTag::update(&mut context.pool(), post, tags.clone()).await?;
  Ok(())
}
