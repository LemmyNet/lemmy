use actix_web::web::Data;
use lemmy_api_common::{comment::CommentResponse, context::LemmyContext};
use lemmy_db_schema::{
  newtypes::{CommentId, LocalUserId},
  source::comment::Comment,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::LemmyError;

mod create;
mod delete;
mod read;
mod remove;
mod update;
