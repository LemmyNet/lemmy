// This allows functions to return error types that are made pub(crate) only in test builds.
#![cfg_attr(test, allow(private_interfaces))]

#[cfg(feature = "full")]
pub mod comment_reply_view;
#[cfg(feature = "full")]
pub mod community_follower_view;
#[cfg(feature = "full")]
pub mod community_moderator_view;
#[cfg(feature = "full")]
pub mod community_person_ban_view;
#[cfg(feature = "full")]
pub mod community_view;
#[cfg(feature = "full")]
pub mod person_mention_view;
#[cfg(feature = "full")]
pub mod person_view;
pub mod structs;
