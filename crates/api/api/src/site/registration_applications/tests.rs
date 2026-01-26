use crate::{
  local_user::unread_counts::get_unread_counts,
  site::registration_applications::{
    approve::approve_registration_application,
    list::list_registration_applications,
  },
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_crud::site::update::edit_site;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    local_site::{LocalSite, LocalSiteUpdateForm},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
  },
  test_data::TestData,
};
use lemmy_db_schema_file::{InstanceId, enums::RegistrationMode};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_registration_applications::{
  RegistrationApplicationView,
  api::ApproveRegistrationApplication,
};
use lemmy_db_views_site::api::EditSite;
use lemmy_diesel_utils::{connection::DbPool, traits::Crud};
use lemmy_utils::{CACHE_DURATION_API, error::LemmyResult};
use serial_test::serial;

async fn create_test_site(context: &Data<LemmyContext>) -> LemmyResult<(TestData, LocalUserView)> {
  let pool = &mut context.pool();
  let data = TestData::create(pool).await?;

  // Enable some local site settings
  let local_site_form = LocalSiteUpdateForm {
    require_email_verification: Some(true),
    application_question: Some(Some(".".to_string())),
    registration_mode: Some(RegistrationMode::RequireApplication),
    site_setup: Some(true),
    ..Default::default()
  };
  LocalSite::update(pool, &local_site_form).await?;

  let admin_person = Person::create(
    pool,
    &PersonInsertForm::test_form(data.instance.id, "admin"),
  )
  .await?;
  LocalUser::create(
    pool,
    &LocalUserInsertForm::test_form_admin(admin_person.id),
    vec![],
  )
  .await?;

  let admin_local_user_view = LocalUserView::read_person(pool, admin_person.id).await?;

  Ok((data, admin_local_user_view))
}

async fn signup(
  pool: &mut DbPool<'_>,
  instance_id: InstanceId,
  name: &str,
  email: Option<&str>,
) -> LemmyResult<(LocalUser, RegistrationApplication)> {
  let person_insert_form = PersonInsertForm::test_form(instance_id, name);
  let person = Person::create(pool, &person_insert_form).await?;

  let local_user_insert_form = match email {
    Some(email) => LocalUserInsertForm {
      email: Some(email.to_string()),
      email_verified: Some(false),
      ..LocalUserInsertForm::test_form(person.id)
    },
    None => LocalUserInsertForm::test_form(person.id),
  };

  let local_user = LocalUser::create(pool, &local_user_insert_form, vec![]).await?;

  let application_insert_form = RegistrationApplicationInsertForm {
    local_user_id: local_user.id,
    answer: "x".to_string(),
  };
  let application = RegistrationApplication::create(pool, &application_insert_form).await?;

  Ok((local_user, application))
}

async fn get_application_statuses(
  context: &Data<LemmyContext>,
  admin: LocalUserView,
) -> LemmyResult<(
  i64,
  Vec<RegistrationApplicationView>,
  Vec<RegistrationApplicationView>,
)> {
  let Json(unread_counts) = get_unread_counts(context.clone(), admin.clone()).await?;

  let Json(unread_applications) = list_registration_applications(
    Query::from_query("unread_only=true")?,
    context.clone(),
    admin.clone(),
  )
  .await?;

  let Json(all_applications) = list_registration_applications(
    Query::from_query("unread_only=false")?,
    context.clone(),
    admin,
  )
  .await?;

  Ok((
    unread_counts
      .registration_application_count
      .unwrap_or_default(),
    unread_applications.items,
    all_applications.items,
  ))
}

#[serial]
#[tokio::test]
#[expect(clippy::indexing_slicing)]
async fn test_application_approval() -> LemmyResult<()> {
  let context = LemmyContext::init_test_context().await;
  let pool = &mut context.pool();

  let (data, admin_local_user_view) = create_test_site(&context).await?;

  // Non-unread counts unfortunately are duplicated due to different types (i64 vs usize)
  let mut expected_total_applications = 0;
  let mut expected_unread_applications = 0u8;

  let (local_user_with_email, app_with_email) = signup(
    pool,
    data.instance.id,
    "user_w_email",
    Some("lemmy@localhost"),
  )
  .await?;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // When email verification is required and the email is not verified the application should not
  // be visible to admins
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  LocalUser::update(
    pool,
    local_user_with_email.id,
    &LocalUserUpdateForm {
      email_verified: Some(true),
      ..Default::default()
    },
  )
  .await?;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // When email verification is required and the email is verified the application should be
  // visible to admins
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert!(
    !unread_applications[0]
      .creator_local_user
      .accepted_application
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  approve_registration_application(
    Json(ApproveRegistrationApplication {
      id: app_with_email.id,
      approve: true,
      deny_reason: None,
    }),
    context.clone(),
    admin_local_user_view.clone(),
  )
  .await?;

  expected_unread_applications -= 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // When the application is approved it should only be returned for unread queries
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);
  assert!(all_applications[0].creator_local_user.accepted_application);

  let (_local_user, app_with_email_2) = signup(
    pool,
    data.instance.id,
    "user_w_email_2",
    Some("lemmy2@localhost"),
  )
  .await?;
  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // Email not verified, so application still not visible
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  Box::pin(edit_site(
    Json(EditSite {
      require_email_verification: Some(false),
      ..Default::default()
    }),
    context.clone(),
    admin_local_user_view.clone(),
  ))
  .await?;

  // TODO: There is probably a better way to ensure cache invalidation
  tokio::time::sleep(CACHE_DURATION_API).await;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // After disabling email verification the application should now be visible
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  approve_registration_application(
    Json(ApproveRegistrationApplication {
      id: app_with_email_2.id,
      approve: false,
      deny_reason: None,
    }),
    context.clone(),
    admin_local_user_view.clone(),
  )
  .await?;

  expected_unread_applications -= 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // Denied applications should not be marked as unread
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  signup(pool, data.instance.id, "user_wo_email", None).await?;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // New user without email should immediately be visible
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  signup(pool, data.instance.id, "user_w_email_3", None).await?;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // New user with email should immediately be visible
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  Box::pin(edit_site(
    Json(EditSite {
      registration_mode: Some(RegistrationMode::Open),
      ..Default::default()
    }),
    context.clone(),
    admin_local_user_view.clone(),
  ))
  .await?;

  // TODO: There is probably a better way to ensure cache invalidation
  tokio::time::sleep(CACHE_DURATION_API).await;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // TODO: At this time applications do not get approved when switching to open registration, so the
  //       numbers will not change. See https://github.com/LemmyNet/lemmy/issues/4969
  // expected_application_count = 0;
  // expected_unread_applications_len = 0;

  // When applications are not required all previous applications should become approved but still
  // visible
  assert_eq!(application_count, i64::from(expected_unread_applications),);
  assert_eq!(
    unread_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(all_applications.len(), expected_total_applications,);

  LocalSite::delete(pool).await?;
  // Instance deletion cascades cleanup of all created persons
  data.delete(pool).await?;

  Ok(())
}
