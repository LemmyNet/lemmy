use crate::{send_email, user_language};
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{error::LemmyResult, settings::structs::Settings};

/// Send a new applicant email notification to all admins
pub async fn send_new_applicant_email_to_admins(
  applicant_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let applications_link = &format!(
    "{}/registration_applications",
    settings.get_protocol_and_hostname(),
  );

  for admin in &admins {
    if let Some(email) = &admin.local_user.email {
      let lang = user_language(admin);
      let subject = lang.new_application_subject(&settings.hostname, applicant_username);
      let body = lang.new_application_body(applications_link);
      send_email(&subject, email, &admin.person.name, &body, settings).await?;
    }
  }
  Ok(())
}

/// Send a report to all admins
pub async fn send_new_report_email_to_admins(
  reporter_username: &str,
  reported_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let reports_link = &format!("{}/reports", settings.get_protocol_and_hostname(),);

  for admin in &admins {
    if let Some(email) = &admin.local_user.email {
      let lang = user_language(admin);
      let subject =
        lang.new_report_subject(&settings.hostname, reported_username, reporter_username);
      let body = lang.new_report_body(reports_link);
      send_email(&subject, email, &admin.person.name, &body, settings).await?;
    }
  }
  Ok(())
}
