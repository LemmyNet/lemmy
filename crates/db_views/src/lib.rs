#[cfg(test)]
extern crate serial_test;

#[cfg(feature = "full")]
pub mod comment_report_view;
#[cfg(feature = "full")]
pub mod comment_view;
#[cfg(feature = "full")]
pub mod custom_emoji_view;
#[cfg(feature = "full")]
pub mod local_image_view;
#[cfg(feature = "full")]
pub mod local_user_view;
#[cfg(feature = "full")]
pub mod person_content_combined_view;
#[cfg(feature = "full")]
pub mod person_saved_combined_view;
#[cfg(feature = "full")]
pub mod post_report_view;
#[cfg(feature = "full")]
pub mod post_tags_view;
#[cfg(feature = "full")]
pub mod post_view;
#[cfg(feature = "full")]
pub mod private_message_report_view;
#[cfg(feature = "full")]
pub mod private_message_view;
#[cfg(feature = "full")]
pub mod registration_application_view;
#[cfg(feature = "full")]
pub mod report_combined_view;
#[cfg(feature = "full")]
pub mod site_view;
pub mod structs;
#[cfg(feature = "full")]
pub mod vote_view;

pub trait InternalToCombinedView {
  type CombinedView;

  /// Maps the combined DB row to an enum
  fn map_to_enum(&self) -> Option<Self::CombinedView>;
}
