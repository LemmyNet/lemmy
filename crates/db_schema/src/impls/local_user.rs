use crate::{
  newtypes::{DbUrl, LocalUserId, PersonId},
  schema::{local_user, person, registration_application},
  source::{
    actor_language::{LocalUserLanguage, SiteLanguage},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
  },
  traits::Crud,
  utils::{
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

    let followed_communities = community_actions::table
      .filter(community_actions::person_id.eq(person_id_))
      .filter(community_actions::followed.is_not_null())
      .inner_join(community::table.on(community_actions::community_id.eq(community::id)))
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let saved_posts = post_actions::table
      .filter(post_actions::person_id.eq(person_id_))
      .filter(post_actions::saved.is_not_null())
      .inner_join(post::table.on(post_actions::post_id.eq(post::id)))
      .select(post::ap_id)
      .get_results(conn)
      .await?;

    let saved_comments = comment_actions::table
      .filter(comment_actions::person_id.eq(person_id_))
      .filter(comment_actions::saved.is_not_null())
      .inner_join(comment::table.on(comment_actions::comment_id.eq(comment::id)))
      .select(comment::ap_id)
      .get_results(conn)
      .await?;

    let blocked_communities = community_actions::table
      .filter(community_actions::person_id.eq(person_id_))
      .filter(community_actions::blocked.is_not_null())
      .inner_join(community::table)
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let blocked_users = person_actions::table
      .filter(person_actions::person_id.eq(person_id_))
      .filter(person_actions::blocked.is_not_null())
      .inner_join(person::table.on(person_actions::target_id.eq(person::id)))
      .select(person::actor_id)
      .get_results(conn)
      .await?;

    let blocked_instances = instance_actions::table
      .filter(instance_actions::person_id.eq(person_id_))
      .filter(instance_actions::blocked.is_not_null())
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

#[async_trait]
impl Crud for LocalUser {
  type InsertForm = LocalUserInsertForm;
  type UpdateForm = LocalUserUpdateForm;
  type IdType = LocalUserId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut form_with_encrypted_password = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    form_with_encrypted_password.password_encrypted = password_hash;

    let local_user_ = insert_into(local_user::table)
      .values(form_with_encrypted_password)
      .get_result::<Self>(conn)
      .await?;

    let site_languages = SiteLanguage::read_local_raw(pool).await;
    if let Ok(langs) = site_languages {
      // if site exists, init user with site languages
      LocalUserLanguage::update(pool, langs, local_user_.id).await?;
    } else {
      // otherwise, init with all languages (this only happens during tests and
      // for first admin user, which is created before site)
      LocalUserLanguage::update(pool, vec![], local_user_.id).await?;
    }

    Ok(local_user_)
  }
  async fn update(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user::table.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
