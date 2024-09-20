use crate::site::registration_applications::{
  approve::approve_registration_application,
  list::list_registration_applications,
  unread_count::get_unread_registration_application_count,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{
    ApproveRegistrationApplication,
    EditSite,
    GetUnreadRegistrationApplicationCountResponse,
    ListRegistrationApplicationsResponse,
  },
};
use lemmy_api_crud::site::update::update_site;
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    person::{Person, PersonInsertForm},
    registration_application::{RegistrationApplication, RegistrationApplicationInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType, CACHE_DURATION_API};
use serial_test::serial;

#[allow(clippy::unwrap_used)]
async fn create_test_site(context: &Data<LemmyContext>) -> LemmyResult<(Instance, LocalUserView)> {
  let pool = &mut context.pool();

  let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
    .await
    .expect("Create test instance");

  let admin_person = Person::create(
    pool,
    &PersonInsertForm::test_form(inserted_instance.id, "admin"),
  )
  .await?;
  LocalUser::create(
    pool,
    &LocalUserInsertForm::test_form_admin(admin_person.id),
    vec![],
  )
  .await?;

  let admin_local_user_view = LocalUserView::read_person(pool, admin_person.id)
    .await?
    .unwrap();

  let site_form = SiteInsertForm::new("test site".to_string(), inserted_instance.id);
  let site = Site::create(pool, &site_form).await.unwrap();

  // Create a local site, since this is necessary for determining if email verification is
  // required
  let mut local_site_form = LocalSiteInsertForm::new(site.id);
  local_site_form.require_email_verification = Some(true);
  local_site_form.application_question = Some(".".to_string());
  local_site_form.registration_mode = Some(RegistrationMode::RequireApplication);
  local_site_form.site_setup = Some(true);
  let local_site = LocalSite::create(pool, &local_site_form).await.unwrap();

  // Required to have a working local SiteView when updating the site to change email verification
  // requirement or registration mode
  let rate_limit_form = LocalSiteRateLimitInsertForm::new(local_site.id);
  LocalSiteRateLimit::create(pool, &rate_limit_form)
    .await
    .unwrap();

  Ok((inserted_instance, admin_local_user_view))
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

#[allow(clippy::unwrap_used)]
async fn get_application_statuses(
  context: &Data<LemmyContext>,
  admin: LocalUserView,
) -> LemmyResult<(
  Json<GetUnreadRegistrationApplicationCountResponse>,
  Json<ListRegistrationApplicationsResponse>,
  Json<ListRegistrationApplicationsResponse>,
)> {
  let application_count =
    get_unread_registration_application_count(context.reset_request_count(), admin.clone()).await?;

  let unread_applications = list_registration_applications(
    Query::from_query("unread_only=true").unwrap(),
    context.reset_request_count(),
    admin.clone(),
  )
  .await?;

  let all_applications = list_registration_applications(
    Query::from_query("unread_only=false").unwrap(),
    context.reset_request_count(),
    admin,
  )
  .await?;

  Ok((application_count, unread_applications, all_applications))
}

#[allow(clippy::indexing_slicing)]
#[allow(clippy::unwrap_used)]
#[tokio::test]
#[serial]
async fn test_application_approval() -> LemmyResult<()> {
  let context = LemmyContext::init_test_context().await;
  let pool = &mut context.pool();

  let (instance, admin_local_user_view) = create_test_site(&context).await?;

  // Non-unread counts unfortunately are duplicated due to different types (i64 vs usize)
  let mut expected_total_applications = 0;
  let mut expected_unread_applications = 0u8;

  let (local_user_with_email, app_with_email) =
    signup(pool, instance.id, "user_w_email", Some("lemmy@localhost")).await?;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // When email verification is required and the email is not verified the application should not
  // be visible to admins
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

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
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert!(
    !unread_applications.registration_applications[0]
      .creator_local_user
      .accepted_application
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  let approval = approve_registration_application(
    Json(ApproveRegistrationApplication {
      id: app_with_email.id,
      approve: true,
      deny_reason: None,
    }),
    context.reset_request_count(),
    admin_local_user_view.clone(),
  )
  .await;
  // Approval should be processed up until email sending is attempted
  assert!(approval.is_err_and(|e| e.error_type == LemmyErrorType::NoEmailSetup));

  expected_unread_applications -= 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // When the application is approved it should only be returned for unread queries
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );
  assert!(
    all_applications.registration_applications[0]
      .creator_local_user
      .accepted_application
  );

  let (_local_user, app_with_email_2) = signup(
    pool,
    instance.id,
    "user_w_email_2",
    Some("lemmy2@localhost"),
  )
  .await?;
  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // Email not verified, so application still not visible
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  update_site(
    Json(EditSite {
      require_email_verification: Some(false),
      ..Default::default()
    }),
    context.reset_request_count(),
    admin_local_user_view.clone(),
  )
  .await?;

  // TODO: There is probably a better way to ensure cache invalidation
  tokio::time::sleep(CACHE_DURATION_API).await;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // After disabling email verification the application should now be visible
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  approve_registration_application(
    Json(ApproveRegistrationApplication {
      id: app_with_email_2.id,
      approve: false,
      deny_reason: None,
    }),
    context.reset_request_count(),
    admin_local_user_view.clone(),
  )
  .await?;

  expected_unread_applications -= 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // Denied applications should not be marked as unread
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  signup(pool, instance.id, "user_wo_email", None).await?;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // New user without email should immediately be visible
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  signup(pool, instance.id, "user_w_email_3", None).await?;

  expected_total_applications += 1;
  expected_unread_applications += 1;

  let (application_count, unread_applications, all_applications) =
    get_application_statuses(&context, admin_local_user_view.clone()).await?;

  // New user with email should immediately be visible
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  update_site(
    Json(EditSite {
      registration_mode: Some(RegistrationMode::Open),
      ..Default::default()
    }),
    context.reset_request_count(),
    admin_local_user_view.clone(),
  )
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
  assert_eq!(
    application_count.registration_applications,
    i64::from(expected_unread_applications),
  );
  assert_eq!(
    unread_applications.registration_applications.len(),
    usize::from(expected_unread_applications),
  );
  assert_eq!(
    all_applications.registration_applications.len(),
    expected_total_applications,
  );

  LocalSite::delete(pool).await?;
  // Instance deletion cascades cleanup of all created persons
  Instance::delete(pool, instance.id).await?;

  Ok(())
}
