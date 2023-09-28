use crate::{
  newtypes::{DbUrl, LocalUserId, PersonId},
  schema::local_user::dsl::{
    accepted_application,
    email,
    email_verified,
    local_user,
    password_encrypted,
    validator_time,
  },
  source::{
    actor_language::{LocalUserLanguage, SiteLanguage},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, naive_now, DbPool},
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;

impl LocalUser {
  pub async fn update_password(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    new_password: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(local_user.find(local_user_id))
      .set((
        password_encrypted.eq(password_hash),
        validator_time.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
      .await
  }

  pub async fn set_all_users_email_verified(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user)
      .set(email_verified.eq(true))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn set_all_users_registration_applications_accepted(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_user)
      .set(accepted_application.eq(true))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn is_email_taken(pool: &mut DbPool<'_>, email_: &str) -> Result<bool, Error> {
    use diesel::dsl::{exists, select};
    let conn = &mut get_conn(pool).await?;
    select(exists(local_user.filter(email.eq(email_))))
      .get_result(conn)
      .await
  }

  // TODO: maybe move this and pass in LocalUserView
  pub async fn export_backup(
    pool: &mut DbPool<'_>,
    person_id_: PersonId,
  ) -> Result<UserBackupLists, Error> {
    use crate::schema::{
      community,
      community_block,
      community_follower,
      person,
      person_block,
      post,
      post_saved,
    };
    let conn = &mut get_conn(pool).await?;

    let followed_communities = community_follower::dsl::community_follower
      .filter(community_follower::person_id.eq(person_id_))
      .inner_join(community::table.on(community_follower::community_id.eq(community::id)))
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let blocked_communities = community_block::dsl::community_block
      .filter(community_block::person_id.eq(person_id_))
      .inner_join(community::table)
      .select(community::actor_id)
      .get_results(conn)
      .await?;

    let blocked_users = person_block::dsl::person_block
      .filter(person_block::person_id.eq(person_id_))
      .inner_join(person::table.on(person_block::target_id.eq(person::id)))
      .select(person::actor_id)
      .get_results(conn)
      .await?;

    let saved_posts = post_saved::dsl::post_saved
      .filter(post_saved::person_id.eq(person_id_))
      .inner_join(post::table.on(post_saved::post_id.eq(post::id)))
      .select(post::ap_id)
      .get_results(conn)
      .await?;

    // TODO: use join for parallel queries?

    Ok(UserBackupLists {
      followed_communities,
      blocked_communities,
      blocked_users,
      saved_posts,
    })
  }
}

pub struct UserBackupLists {
  pub followed_communities: Vec<DbUrl>,
  pub blocked_communities: Vec<DbUrl>,
  pub blocked_users: Vec<DbUrl>,
  pub saved_posts: Vec<DbUrl>,
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

    let local_user_ = insert_into(local_user)
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
    diesel::update(local_user.find(local_user_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
