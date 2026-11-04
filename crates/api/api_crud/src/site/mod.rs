use lemmy_db_schema_file::enums::{ListingType, RegistrationMode};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod read;
pub mod update;

/// Checks whether the default post listing type is valid for a site.
pub fn site_default_post_listing_type_check(
  default_post_listing_type: &Option<ListingType>,
) -> LemmyResult<()> {
  if let Some(listing_type) = default_post_listing_type {
    // Dont allow Subscribed or ModeratorView as default listing type
    if [ListingType::Subscribed, ListingType::ModeratorView].contains(listing_type) {
      Err(LemmyErrorType::InvalidDefaultPostListingType.into())
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
    Err(LemmyErrorType::ApplicationQuestionRequired.into())
  } else {
    Ok(())
  }
}

fn not_zero(val: Option<i32>) -> Option<i32> {
  match val {
    Some(0) => None,
    v => v,
  }
}

fn validate_resend_verification_limits(
  ip_max: Option<i32>,
  ip_interval: Option<i32>,
  account_max: Option<i32>,
  account_interval: Option<i32>,
) -> LemmyResult<()> {
  let valid_counts = [ip_max, account_max]
    .into_iter()
    .flatten()
    .all(|value| value >= 0);
  let valid_intervals = [ip_interval, account_interval]
    .into_iter()
    .flatten()
    .all(|value| value > 0);
  if !valid_counts || !valid_intervals {
    return Err(LemmyErrorType::InvalidRateLimit.into());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::site::{
    application_question_check,
    not_zero,
    site_default_post_listing_type_check,
    validate_resend_verification_limits,
  };
  use lemmy_db_schema_file::enums::{ListingType, RegistrationMode};
  use lemmy_utils::error::LemmyErrorType;

  #[test]
  fn verification_email_limits_require_non_negative_counts_and_positive_windows() {
    assert!(validate_resend_verification_limits(None, None, None, None).is_ok());
    assert!(validate_resend_verification_limits(Some(10), Some(3600), Some(3), Some(3600)).is_ok());
    assert!(validate_resend_verification_limits(Some(0), Some(1), Some(0), Some(1)).is_ok());
    for (ip_max, ip_interval, account_max, account_interval) in [
      (Some(-1), None, None, None),
      (None, None, Some(-1), None),
      (None, Some(0), None, None),
      (None, Some(-1), None, None),
      (None, None, None, Some(0)),
      (None, None, None, Some(-1)),
    ] {
      assert!(matches!(
        validate_resend_verification_limits(ip_max, ip_interval, account_max, account_interval),
        Err(error) if error.error_type == LemmyErrorType::InvalidRateLimit
      ));
    }
  }

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
      application_question_check(
        &Some(String::from("q")),
        &Some(String::new()),
        RegistrationMode::RequireApplication
      )
      .is_err(),
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
      application_question_check(
        &None,
        &Some(String::from("q")),
        RegistrationMode::RequireApplication
      )
      .is_ok(),
      "Expected application to be valid because new application provided, current question: {:?}, new question: {:?}, mode: {:?}",
      None::<String>,
      Some(String::from("q")),
      RegistrationMode::RequireApplication
    );
    assert!(
      application_question_check(
        &Some(String::from("q")),
        &None,
        RegistrationMode::RequireApplication
      )
      .is_ok(),
      "Expected application to be valid because application existed, current question: {:?}, new question: {:?}, mode: {:?}",
      Some(String::from("q")),
      None::<String>,
      RegistrationMode::RequireApplication
    );
  }

  #[test]
  fn test_not_zero() {
    assert_eq!(None, not_zero(None));
    assert_eq!(None, not_zero(Some(0)));
    assert_eq!(Some(5), not_zero(Some(5)));
  }
}
