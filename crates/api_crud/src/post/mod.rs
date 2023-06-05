use actix_web::web::Data;
use lemmy_api_common::{context::LemmyContext, post::PostResponse, utils::is_mod_or_admin};
use lemmy_db_schema::newtypes::{CommunityId, PersonId, PostId};
use lemmy_db_views::structs::PostView;
use lemmy_utils::error::LemmyError;

mod create;
mod delete;
mod read;
mod remove;
mod update;
