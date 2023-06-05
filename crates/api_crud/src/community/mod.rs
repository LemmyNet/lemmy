use actix_web::web::Data;
use lemmy_api_common::{
  community::CommunityResponse,
  context::LemmyContext,
  utils::is_mod_or_admin,
};
use lemmy_db_schema::{newtypes::CommunityId, source::actor_language::CommunityLanguage};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::LemmyError;

mod create;
mod delete;
mod list;
mod remove;
mod update;
