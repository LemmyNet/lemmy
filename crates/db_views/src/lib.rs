#[cfg(test)]
extern crate serial_test;

#[cfg(feature = "full")]
pub mod comment_report_view;
#[cfg(feature = "full")]
pub mod comment_view;
#[cfg(feature = "full")]
pub mod custom_emoji_view;
#[cfg(feature = "full")]
pub mod local_user_view;
#[cfg(feature = "full")]
pub mod post_report_view;
#[cfg(feature = "full")]
pub mod post_view;
#[cfg(feature = "full")]
pub mod private_message_report_view;
#[cfg(feature = "full")]
pub mod private_message_view;
#[cfg(feature = "full")]
pub mod registration_application_view;
#[cfg(feature = "full")]
pub mod site_view;
pub mod structs;
#[cfg(feature = "full")]
pub mod tagline_view;
#[cfg(feature = "full")]
pub mod vote_view;
