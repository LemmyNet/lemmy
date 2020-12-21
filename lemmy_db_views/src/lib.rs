pub mod comment_report_view;
pub mod comment_view;
pub mod community;
pub mod moderator;
pub mod post_report_view;
pub mod post_view;
pub mod private_message_view;
pub mod site_view;
pub mod user_mention_view;
pub mod user_view;

pub(crate) trait ViewToVec {
  type DbTuple;
  fn to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
  where
    Self: Sized;
}
