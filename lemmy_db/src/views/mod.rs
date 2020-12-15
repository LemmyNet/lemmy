pub mod comment_view;
pub mod community_follower_view;
pub mod community_moderator_view;
pub mod community_user_ban_view;
pub mod community_view;
pub mod post_view;
pub mod site_view;
pub mod user_view;

pub(crate) trait ViewToVec {
  type DbTuple;
  fn to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
  where
    Self: Sized;
}
