use crate::{
  newtypes::{DbUrl, LanguageId, LocalUserId, PersonId},
  schema::{local_user, person, registration_application},
  source::{
    actor_language::LocalUserLanguage,
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    local_user_vote_display_mode::{LocalUserVoteDisplayMode, LocalUserVoteDisplayModeInsertForm},
  },
  utils::{
    action_query,
    functions::{coalesce, lower},
    get_conn,
    now,
    DbPool,
  },
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{
  dsl::{insert_into, not, IntervalDsl},
  result::Error,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl LocalUser {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &LocalUserInsertForm,
    languages: Vec<LanguageId>,
  ) -> Result<LocalUser, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut form_with_encrypted_password = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    form_with_encrypted_password.password_encrypted = password_hash;

    let local_user_ = insert_into(local_user::table)
      .values(form_with_encrypted_password)
      .get_result::<Self>(conn)
      .await?;

    LocalUserLanguage::update(pool, languages, local_user_.id).await?;

    // Create their vote_display_modes
    let vote_display_mode_form = LocalUserVoteDisplayModeInsertForm::builder()
      .local_user_id(local_user_.id)
      .build();
    LocalUserVoteDisplayMode::create(pool, &vote_display_mode_form).await?;

    Ok(local_user_)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    form: &LocalUserUpdateForm,
  ) -> Result<LocalUser, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user::table.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn delete(pool: &mut DbPool<'_>, id: LocalUserId) -> Result<usize, Error> {
    let conn = &mut *get_conn(pool).await?;
    diesel::delete(local_user::table.find(id))
      .execute(conn)
      .await
  }

  pub async fn update_password(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    new_password: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(local_user::table.find(local_user_id))
      .set((local_user::password_encrypted.eq(password_hash),))
      .get_result::<Self>(conn)
      .await
  }

  pub async fn set_all_users_email_verified(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user::table)
      .set(local_user::email_verified.eq(true))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn set_all_users_registration_applications_accepted(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user::table)
      .set(local_user::accepted_application.eq(true))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn delete_old_denied_local_users(pool: &mut DbPool<'_>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    // Make sure:
    // - The deny reason exists
    // - The app is older than a week
    // - The accepted_application is false
    let old_denied_registrations = registration_application::table
      .filter(registration_application::deny_reason.is_not_null())
      .filter(registration_application::published.lt(now() - 1.week()))
      .select(registration_application::local_user_id);

    // Delete based on join logic is here:
    // https://stackoverflow.com/questions/60836040/how-do-i-perform-a-delete-with-sub-query-in-diesel-against-a-postgres-database
    let local_users = local_user::table
      .filter(local_user::id.eq_any(old_denied_registrations))
      .filter(not(local_user::accepted_application))
      .select(local_user::person_id);

    // Delete the person rows, which should automatically clear the local_user ones
    let persons = person::table.filter(person::id.eq_any(local_users));

    diesel::delete(persons).execute(conn).await
  }

  pub async fn is_email_taken(pool: &mut DbPool<'_>, email: &str) -> Result<bool, Error> {
    use diesel::dsl::{exists, select};
    let conn = &mut get_conn(pool).await?;
    select(exists(local_user::table.filter(
      lower(coalesce(local_user::email, "")).eq(email.to_lowercase()),
    )))
    .get_result(conn)
    .await
  }

  // TODO: maybe move this and pass in LocalUserView
  pub async fn export_backup(
    pool: &mut DbPool<'_>,
    person_id_: PersonId,
  ) -> Result<UserBackupLists, Error> {
    use crate::schema::{
      comment,
      comment_actions,
      community,
      community_actions,
      instance,
      instance_actions,
      person_actions,
      post,
      post_actions,
    };
    let conn = &mut get_conn(pool).await?;

    let followed_communities = action_query(community_actions::followed)
      .filter(community_actions::person_id.eq(person_id_))
      .inner_join(community::table.on(community_actions::community_id.eq(community::id)))
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let saved_posts = action_query(post_actions::saved)
      .filter(post_actions::person_id.eq(person_id_))
      .inner_join(post::table.on(post_actions::post_id.eq(post::id)))
      .select(post::ap_id)
      .get_results(conn)
      .await?;

    let saved_comments = action_query(comment_actions::saved)
      .filter(comment_actions::person_id.eq(person_id_))
      .inner_join(comment::table.on(comment_actions::comment_id.eq(comment::id)))
      .select(comment::ap_id)
      .get_results(conn)
      .await?;

    let blocked_communities = action_query(community_actions::blocked)
      .filter(community_actions::person_id.eq(person_id_))
      .inner_join(community::table)
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let blocked_users = action_query(person_actions::blocked)
      .filter(person_actions::person_id.eq(person_id_))
      .inner_join(person::table.on(person_actions::target_id.eq(person::id)))
      .select(person::actor_id)
      .get_results(conn)
      .await?;

    let blocked_instances = action_query(instance_actions::blocked)
      .filter(instance_actions::person_id.eq(person_id_))
      .inner_join(instance::table)
      .select(instance::domain)
      .get_results(conn)
      .await?;

    // TODO: use join for parallel queries?

    Ok(UserBackupLists {
      followed_communities,
      saved_posts,
      saved_comments,
      blocked_communities,
      blocked_users,
      blocked_instances,
    })
  }
}

impl LocalUserInsertForm {
  pub fn test_form(person_id: PersonId) -> Self {
    Self::builder()
      .person_id(person_id)
      .password_encrypted(String::new())
      .build()
  }
}

pub struct UserBackupLists {
  pub followed_communities: Vec<DbUrl>,
  pub saved_posts: Vec<DbUrl>,
  pub saved_comments: Vec<DbUrl>,
  pub blocked_communities: Vec<DbUrl>,
  pub blocked_users: Vec<DbUrl>,
  pub blocked_instances: Vec<String>,
}
