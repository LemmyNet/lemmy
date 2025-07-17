use crate::RegistrationApplicationView;
use diesel::{
  dsl::count,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  aliases,
  newtypes::{PaginationCursor, PersonId, RegistrationApplicationId},
  source::registration_application::RegistrationApplication,
  traits::{Crud, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use lemmy_db_schema_file::schema::{local_user, person, registration_application};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl PaginationCursorBuilder for RegistrationApplicationView {
  type CursorData = RegistrationApplication;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('R', self.registration_application.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let id = cursor.first_id()?;
    RegistrationApplication::read(pool, RegistrationApplicationId(id)).await
  }
}

impl RegistrationApplicationView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let local_user_join =
      local_user::table.on(registration_application::local_user_id.eq(local_user::id));

    let creator_join = person::table.on(local_user::person_id.eq(person::id));
    let admin_join = aliases::person1
      .on(registration_application::admin_id.eq(aliases::person1.field(person::id).nullable()));

    registration_application::table
      .inner_join(local_user_join)
      .inner_join(creator_join)
      .left_join(admin_join)
  }

  pub async fn read(pool: &mut DbPool<'_>, id: RegistrationApplicationId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(registration_application::id.eq(id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_by_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Returns the current unread registration_application count
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    verified_email_only: bool,
  ) -> LemmyResult<i64> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins()
      .filter(RegistrationApplication::is_unread())
      .select(count(registration_application::id))
      .into_boxed();

    if verified_email_only {
      query = query.filter(local_user::email_verified.eq(true))
    }

    query
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[derive(Default)]
pub struct RegistrationApplicationQuery {
  pub unread_only: Option<bool>,
  pub verified_email_only: Option<bool>,
  pub cursor_data: Option<RegistrationApplication>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl RegistrationApplicationQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<RegistrationApplicationView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;
    let o = self;

    let mut query = RegistrationApplicationView::joins()
      .select(RegistrationApplicationView::as_select())
      .limit(limit)
      .into_boxed();

    if o.unread_only.unwrap_or_default() {
      query = query
        .filter(RegistrationApplication::is_unread())
        .order_by(registration_application::published_at.asc());
    } else {
      query = query.order_by(registration_application::published_at.desc());
    }

    if o.verified_email_only.unwrap_or_default() {
      query = query.filter(local_user::email_verified.eq(true))
    }

    // Sorting by published
    let paginated_query = paginate(query, SortDirection::Desc, o.cursor_data, None, o.page_back);

    paginated_query
      .load::<RegistrationApplicationView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {

  use crate::{impls::RegistrationApplicationQuery, RegistrationApplicationView};
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonInsertForm},
      registration_application::{
        RegistrationApplication,
        RegistrationApplicationInsertForm,
        RegistrationApplicationUpdateForm,
      },
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_person_form = PersonInsertForm::test_form(instance.id, "timmy_rav");

    let timmy_person = Person::create(pool, &timmy_person_form).await?;

    let timmy_local_user_form = LocalUserInsertForm::test_form_admin(timmy_person.id);

    let _inserted_timmy_local_user =
      LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;

    let sara_person_form = PersonInsertForm::test_form(instance.id, "sara_rav");

    let sara_person = Person::create(pool, &sara_person_form).await?;

    let sara_local_user_form = LocalUserInsertForm::test_form(sara_person.id);

    let sara_local_user = LocalUser::create(pool, &sara_local_user_form, vec![]).await?;

    // Sara creates an application
    let sara_app_form = RegistrationApplicationInsertForm {
      local_user_id: sara_local_user.id,
      answer: "LET ME IIIIINN".to_string(),
    };

    let sara_app = RegistrationApplication::create(pool, &sara_app_form).await?;

    let read_sara_app_view = RegistrationApplicationView::read(pool, sara_app.id).await?;

    let jess_person_form = PersonInsertForm::test_form(instance.id, "jess_rav");

    let inserted_jess_person = Person::create(pool, &jess_person_form).await?;

    let jess_local_user_form = LocalUserInsertForm::test_form(inserted_jess_person.id);

    let jess_local_user = LocalUser::create(pool, &jess_local_user_form, vec![]).await?;

    // Sara creates an application
    let jess_app_form = RegistrationApplicationInsertForm {
      local_user_id: jess_local_user.id,
      answer: "LET ME IIIIINN".to_string(),
    };

    let jess_app = RegistrationApplication::create(pool, &jess_app_form).await?;

    let read_jess_app_view = RegistrationApplicationView::read(pool, jess_app.id).await?;

    let mut expected_sara_app_view = RegistrationApplicationView {
      registration_application: sara_app.clone(),
      creator_local_user: LocalUser {
        id: sara_local_user.id,
        person_id: sara_local_user.person_id,
        email: sara_local_user.email,
        show_nsfw: sara_local_user.show_nsfw,
        blur_nsfw: sara_local_user.blur_nsfw,
        theme: sara_local_user.theme,
        default_post_sort_type: sara_local_user.default_post_sort_type,
        default_comment_sort_type: sara_local_user.default_comment_sort_type,
        default_listing_type: sara_local_user.default_listing_type,
        interface_language: sara_local_user.interface_language,
        show_avatars: sara_local_user.show_avatars,
        send_notifications_to_email: sara_local_user.send_notifications_to_email,
        show_bot_accounts: sara_local_user.show_bot_accounts,
        show_read_posts: sara_local_user.show_read_posts,
        email_verified: sara_local_user.email_verified,
        accepted_application: sara_local_user.accepted_application,
        totp_2fa_secret: sara_local_user.totp_2fa_secret,
        password_encrypted: sara_local_user.password_encrypted,
        open_links_in_new_tab: sara_local_user.open_links_in_new_tab,
        infinite_scroll_enabled: sara_local_user.infinite_scroll_enabled,
        post_listing_mode: sara_local_user.post_listing_mode,
        totp_2fa_enabled: sara_local_user.totp_2fa_enabled,
        enable_keyboard_navigation: sara_local_user.enable_keyboard_navigation,
        enable_animated_images: sara_local_user.enable_animated_images,
        enable_private_messages: sara_local_user.enable_private_messages,
        collapse_bot_comments: sara_local_user.collapse_bot_comments,
        last_donation_notification_at: sara_local_user.last_donation_notification_at,
        show_upvotes: sara_local_user.show_upvotes,
        show_downvotes: sara_local_user.show_downvotes,
        admin: sara_local_user.admin,
        auto_mark_fetched_posts_as_read: sara_local_user.auto_mark_fetched_posts_as_read,
        hide_media: sara_local_user.hide_media,
        default_post_time_range_seconds: sara_local_user.default_post_time_range_seconds,
        show_score: sara_local_user.show_score,
        show_upvote_percentage: sara_local_user.show_upvote_percentage,
        show_person_votes: sara_local_user.show_person_votes,
      },
      creator: Person {
        id: sara_person.id,
        name: sara_person.name.clone(),
        display_name: None,
        published_at: sara_person.published_at,
        avatar: None,
        ap_id: sara_person.ap_id.clone(),
        local: true,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated_at: None,
        inbox_url: sara_person.inbox_url.clone(),
        matrix_user_id: None,
        instance_id: instance.id,
        private_key: sara_person.private_key,
        public_key: sara_person.public_key,
        last_refreshed_at: sara_person.last_refreshed_at,
        post_count: 0,
        post_score: 0,
        comment_count: 0,
        comment_score: 0,
      },
      admin: None,
    };

    assert_eq!(read_sara_app_view, expected_sara_app_view);

    // Do a batch read of the applications
    let apps = RegistrationApplicationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;

    assert_eq!(
      apps,
      [expected_sara_app_view.clone(), read_jess_app_view.clone()]
    );

    // Make sure the counts are correct
    let unread_count = RegistrationApplicationView::get_unread_count(pool, false).await?;
    assert_eq!(unread_count, 2);

    // Approve the application
    let approve_form = RegistrationApplicationUpdateForm {
      admin_id: Some(Some(timmy_person.id)),
      deny_reason: None,
    };

    RegistrationApplication::update(pool, sara_app.id, &approve_form).await?;

    // Update the local_user row
    let approve_local_user_form = LocalUserUpdateForm {
      accepted_application: Some(true),
      ..Default::default()
    };

    LocalUser::update(pool, sara_local_user.id, &approve_local_user_form).await?;

    let read_sara_app_view_after_approve =
      RegistrationApplicationView::read(pool, sara_app.id).await?;

    // Make sure the columns changed
    expected_sara_app_view
      .creator_local_user
      .accepted_application = true;
    expected_sara_app_view.registration_application.admin_id = Some(timmy_person.id);

    expected_sara_app_view.admin = Some(Person {
      id: timmy_person.id,
      name: timmy_person.name.clone(),
      display_name: None,
      published_at: timmy_person.published_at,
      avatar: None,
      ap_id: timmy_person.ap_id.clone(),
      local: true,
      deleted: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated_at: None,
      inbox_url: timmy_person.inbox_url.clone(),
      matrix_user_id: None,
      instance_id: instance.id,
      private_key: timmy_person.private_key,
      public_key: timmy_person.public_key,
      last_refreshed_at: timmy_person.last_refreshed_at,
      post_count: 0,
      post_score: 0,
      comment_count: 0,
      comment_score: 0,
    });
    assert_eq!(read_sara_app_view_after_approve, expected_sara_app_view);

    // Do a batch read of apps again
    // It should show only jessicas which is unresolved
    let apps_after_resolve = RegistrationApplicationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(apps_after_resolve, vec![read_jess_app_view]);

    // Make sure the counts are correct
    let unread_count_after_approve =
      RegistrationApplicationView::get_unread_count(pool, false).await?;
    assert_eq!(unread_count_after_approve, 1);

    // Make sure the not undenied_only has all the apps
    let all_apps = RegistrationApplicationQuery::default().list(pool).await?;
    assert_eq!(all_apps.len(), 2);

    Person::delete(pool, timmy_person.id).await?;
    Person::delete(pool, sara_person.id).await?;
    Person::delete(pool, inserted_jess_person.id).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
