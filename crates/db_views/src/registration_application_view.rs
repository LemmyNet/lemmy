use crate::structs::RegistrationApplicationView;
use diesel::{
  dsl::count,
  pg::Pg,
  result::Error,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  schema::{local_user, person, registration_application},
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, RegistrationApplicationView, i32>,
  impl ListFn<'a, RegistrationApplicationView, RegistrationApplicationQuery>,
> {
  let all_joins = |query: registration_application::BoxedQuery<'a, Pg>| {
    query
      .inner_join(local_user::table.on(registration_application::local_user_id.eq(local_user::id)))
      .inner_join(person::table.on(local_user::person_id.eq(person::id)))
      .left_join(
        aliases::person1
          .on(registration_application::admin_id.eq(aliases::person1.field(person::id).nullable())),
      )
      .order_by(registration_application::published.desc())
      .select((
        registration_application::all_columns,
        local_user::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns).nullable(),
      ))
  };

  let read = move |mut conn: DbConn<'a>, registration_application_id: i32| async move {
    all_joins(
      registration_application::table
        .find(registration_application_id)
        .into_boxed(),
    )
    .first(&mut conn)
    .await
  };

  let list = move |mut conn: DbConn<'a>, options: RegistrationApplicationQuery| async move {
    let mut query = all_joins(registration_application::table.into_boxed());

    // If viewing all applications, order by newest, but if viewing unresolved only, show the oldest first (FIFO)
    if options.unread_only {
      query = query
        .filter(registration_application::admin_id.is_null())
        .order_by(registration_application::published.asc());
    } else {
      query = query.order_by(registration_application::published.desc());
    }

    if options.verified_email_only {
      query = query.filter(local_user::email_verified.eq(true))
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query.limit(limit).offset(offset);

    query.load::<RegistrationApplicationView>(&mut conn).await
  };

  Queries::new(read, list)
}

impl RegistrationApplicationView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    registration_application_id: i32,
  ) -> Result<Option<Self>, Error> {
    queries().read(pool, registration_application_id).await
  }

  /// Returns the current unread registration_application count
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    verified_email_only: bool,
  ) -> Result<i64, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);

    let mut query = registration_application::table
      .inner_join(local_user::table.on(registration_application::local_user_id.eq(local_user::id)))
      .inner_join(person::table.on(local_user::person_id.eq(person::id)))
      .left_join(
        person_alias_1
          .on(registration_application::admin_id.eq(person_alias_1.field(person::id).nullable())),
      )
      .filter(registration_application::admin_id.is_null())
      .into_boxed();

    if verified_email_only {
      query = query.filter(local_user::email_verified.eq(true))
    }

    query
      .select(count(registration_application::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default)]
pub struct RegistrationApplicationQuery {
  pub unread_only: bool,
  pub verified_email_only: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl RegistrationApplicationQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<RegistrationApplicationView>, Error> {
    queries().list(pool, self).await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::registration_application_view::{
    RegistrationApplicationQuery,
    RegistrationApplicationView,
  };
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
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let timmy_person_form = PersonInsertForm::builder()
      .name("timmy_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_timmy_person = Person::create(pool, &timmy_person_form).await.unwrap();

    let timmy_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_timmy_person.id)
      .password_encrypted("nada".to_string())
      .admin(Some(true))
      .build();

    let _inserted_timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![])
      .await
      .unwrap();

    let sara_person_form = PersonInsertForm::builder()
      .name("sara_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_sara_person = Person::create(pool, &sara_person_form).await.unwrap();

    let sara_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_sara_person.id)
      .password_encrypted("nada".to_string())
      .build();

    let inserted_sara_local_user = LocalUser::create(pool, &sara_local_user_form, vec![])
      .await
      .unwrap();

    // Sara creates an application
    let sara_app_form = RegistrationApplicationInsertForm {
      local_user_id: inserted_sara_local_user.id,
      answer: "LET ME IIIIINN".to_string(),
    };

    let sara_app = RegistrationApplication::create(pool, &sara_app_form)
      .await
      .unwrap();

    let read_sara_app_view = RegistrationApplicationView::read(pool, sara_app.id)
      .await
      .unwrap()
      .unwrap();

    let jess_person_form = PersonInsertForm::builder()
      .name("jess_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_jess_person = Person::create(pool, &jess_person_form).await.unwrap();

    let jess_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_jess_person.id)
      .password_encrypted("nada".to_string())
      .build();

    let inserted_jess_local_user = LocalUser::create(pool, &jess_local_user_form, vec![])
      .await
      .unwrap();

    // Sara creates an application
    let jess_app_form = RegistrationApplicationInsertForm {
      local_user_id: inserted_jess_local_user.id,
      answer: "LET ME IIIIINN".to_string(),
    };

    let jess_app = RegistrationApplication::create(pool, &jess_app_form)
      .await
      .unwrap();

    let read_jess_app_view = RegistrationApplicationView::read(pool, jess_app.id)
      .await
      .unwrap()
      .unwrap();

    let mut expected_sara_app_view = RegistrationApplicationView {
      registration_application: sara_app.clone(),
      creator_local_user: LocalUser {
        id: inserted_sara_local_user.id,
        person_id: inserted_sara_local_user.person_id,
        email: inserted_sara_local_user.email,
        show_nsfw: inserted_sara_local_user.show_nsfw,
        auto_expand: inserted_sara_local_user.auto_expand,
        blur_nsfw: inserted_sara_local_user.blur_nsfw,
        theme: inserted_sara_local_user.theme,
        default_sort_type: inserted_sara_local_user.default_sort_type,
        default_listing_type: inserted_sara_local_user.default_listing_type,
        interface_language: inserted_sara_local_user.interface_language,
        show_avatars: inserted_sara_local_user.show_avatars,
        send_notifications_to_email: inserted_sara_local_user.send_notifications_to_email,
        show_bot_accounts: inserted_sara_local_user.show_bot_accounts,
        show_scores: inserted_sara_local_user.show_scores,
        show_read_posts: inserted_sara_local_user.show_read_posts,
        email_verified: inserted_sara_local_user.email_verified,
        accepted_application: inserted_sara_local_user.accepted_application,
        totp_2fa_secret: inserted_sara_local_user.totp_2fa_secret,
        password_encrypted: inserted_sara_local_user.password_encrypted,
        open_links_in_new_tab: inserted_sara_local_user.open_links_in_new_tab,
        infinite_scroll_enabled: inserted_sara_local_user.infinite_scroll_enabled,
        admin: false,
        post_listing_mode: inserted_sara_local_user.post_listing_mode,
        totp_2fa_enabled: inserted_sara_local_user.totp_2fa_enabled,
        enable_keyboard_navigation: inserted_sara_local_user.enable_keyboard_navigation,
        enable_animated_images: inserted_sara_local_user.enable_animated_images,
        collapse_bot_comments: inserted_sara_local_user.collapse_bot_comments,
      },
      creator: Person {
        id: inserted_sara_person.id,
        name: inserted_sara_person.name.clone(),
        display_name: None,
        published: inserted_sara_person.published,
        avatar: None,
        actor_id: inserted_sara_person.actor_id.clone(),
        local: true,
        banned: false,
        ban_expires: None,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_sara_person.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        instance_id: inserted_instance.id,
        private_key: inserted_sara_person.private_key,
        public_key: inserted_sara_person.public_key,
        last_refreshed_at: inserted_sara_person.last_refreshed_at,
      },
      admin: None,
    };

    assert_eq!(read_sara_app_view, expected_sara_app_view);

    // Do a batch read of the applications
    let apps = RegistrationApplicationQuery {
      unread_only: (true),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    assert_eq!(
      apps,
      [expected_sara_app_view.clone(), read_jess_app_view.clone()]
    );

    // Make sure the counts are correct
    let unread_count = RegistrationApplicationView::get_unread_count(pool, false)
      .await
      .unwrap();
    assert_eq!(unread_count, 2);

    // Approve the application
    let approve_form = RegistrationApplicationUpdateForm {
      admin_id: Some(Some(inserted_timmy_person.id)),
      deny_reason: None,
    };

    RegistrationApplication::update(pool, sara_app.id, &approve_form)
      .await
      .unwrap();

    // Update the local_user row
    let approve_local_user_form = LocalUserUpdateForm {
      accepted_application: Some(true),
      ..Default::default()
    };

    LocalUser::update(pool, inserted_sara_local_user.id, &approve_local_user_form)
      .await
      .unwrap();

    let read_sara_app_view_after_approve = RegistrationApplicationView::read(pool, sara_app.id)
      .await
      .unwrap()
      .unwrap();

    // Make sure the columns changed
    expected_sara_app_view
      .creator_local_user
      .accepted_application = true;
    expected_sara_app_view.registration_application.admin_id = Some(inserted_timmy_person.id);

    expected_sara_app_view.admin = Some(Person {
      id: inserted_timmy_person.id,
      name: inserted_timmy_person.name.clone(),
      display_name: None,
      published: inserted_timmy_person.published,
      avatar: None,
      actor_id: inserted_timmy_person.actor_id.clone(),
      local: true,
      banned: false,
      ban_expires: None,
      deleted: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated: None,
      inbox_url: inserted_timmy_person.inbox_url.clone(),
      shared_inbox_url: None,
      matrix_user_id: None,
      instance_id: inserted_instance.id,
      private_key: inserted_timmy_person.private_key,
      public_key: inserted_timmy_person.public_key,
      last_refreshed_at: inserted_timmy_person.last_refreshed_at,
    });
    assert_eq!(read_sara_app_view_after_approve, expected_sara_app_view);

    // Do a batch read of apps again
    // It should show only jessicas which is unresolved
    let apps_after_resolve = RegistrationApplicationQuery {
      unread_only: (true),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(apps_after_resolve, vec![read_jess_app_view]);

    // Make sure the counts are correct
    let unread_count_after_approve = RegistrationApplicationView::get_unread_count(pool, false)
      .await
      .unwrap();
    assert_eq!(unread_count_after_approve, 1);

    // Make sure the not undenied_only has all the apps
    let all_apps = RegistrationApplicationQuery::default()
      .list(pool)
      .await
      .unwrap();
    assert_eq!(all_apps.len(), 2);

    Person::delete(pool, inserted_timmy_person.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_sara_person.id).await.unwrap();
    Person::delete(pool, inserted_jess_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();
  }
}
