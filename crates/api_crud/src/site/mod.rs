use lemmy_db_schema::{ListingType, RegistrationMode};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod read;
pub mod update;

/// Checks whether the default post listing type is valid for a site.
pub fn site_default_post_listing_type_check(
  default_post_listing_type: &Option<ListingType>,
) -> LemmyResult<()> {
  if let Some(listing_type) = default_post_listing_type {
    // Only allow all or local as default listing types...
    if listing_type != &ListingType::All && listing_type != &ListingType::Local {
      Err(LemmyErrorType::InvalidDefaultPostListingType)?
    } else {
      Ok(())
    }
  } else {
    Ok(())
  }
}

/// Checks whether the application question and registration mode align.
pub fn application_question_check(
  current_application_question: &Option<String>,
  new_application_question: &Option<String>,
  registration_mode: RegistrationMode,
) -> LemmyResult<()> {
  let has_no_question: bool =
    current_application_question.is_none() && new_application_question.is_none();
  let is_nullifying_question: bool = new_application_question == &Some(String::new());

  if registration_mode == RegistrationMode::RequireApplication
    && (has_no_question || is_nullifying_question)
  {
    Err(LemmyErrorType::ApplicationQuestionRequired)?
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::site::{application_question_check, site_default_post_listing_type_check};
  use lemmy_db_schema::{ListingType, RegistrationMode};

  #[test]
  fn test_site_default_post_listing_type_check() {
    assert!(site_default_post_listing_type_check(&None::<ListingType>).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::All)).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::Local)).is_ok());
    assert!(site_default_post_listing_type_check(&Some(ListingType::Subscribed)).is_err());
  }

  #[test]
  fn test_application_question_check() {
    assert!(
      application_question_check(&Some(String::from("q")), &Some(String::new()), RegistrationMode::RequireApplication).is_err(),
      "Expected application to be invalid because an application is required, current question: {:?}, new question: {:?}",
      "q",
      String::new(),
    );
    assert!(
      application_question_check(&None, &None, RegistrationMode::RequireApplication).is_err(),
      "Expected application to be invalid because an application is required, current question: {:?}, new question: {:?}",
      None::<String>,
      None::<String>
    );

    assert!(
      application_question_check(&None, &None, RegistrationMode::Open).is_ok(),
      "Expected application to be valid because no application required, current question: {:?}, new question: {:?}, mode: {:?}",
      None::<String>,
      None::<String>,
      RegistrationMode::Open
    );
    assert!(
      application_question_check(&None, &Some(String::from("q")), RegistrationMode::RequireApplication).is_ok(),
      "Expected application to be valid because new application provided, current question: {:?}, new question: {:?}, mode: {:?}",
      None::<String>,
      Some(String::from("q")),
      RegistrationMode::RequireApplication
    );
    assert!(
      application_question_check(&Some(String::from("q")), &None, RegistrationMode::RequireApplication).is_ok(),
      "Expected application to be valid because application existed, current question: {:?}, new question: {:?}, mode: {:?}",
      Some(String::from("q")),
      None::<String>,
      RegistrationMode::RequireApplication
    );
  }
}
