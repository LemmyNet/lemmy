use crate::{
  diesel::{BoolExpressionMethods, NullableExpressionMethods, OptionalExtension},
  newtypes::{CommunityId, DbUrl, InstanceId, LocalUserId, PersonId},
  source::person::{
    Person,
    PersonActions,
    PersonBlockForm,
    PersonFollowerForm,
    PersonInsertForm,
    PersonNoteForm,
    PersonUpdateForm,
  },
  traits::{ApubActor, Blockable, Crud, Followable},
  utils::{format_actor_url, functions::lower, get_conn, uplete, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{exists, insert_into, not, select},
  expression::SelectableHelper,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  instance,
  instance_actions,
  local_user,
  person,
  person_actions,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for Person {
  type InsertForm = PersonInsertForm;
  type UpdateForm = PersonUpdateForm;
  type IdType = PersonId;

  // Override this, so that you don't get back deleted
  async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .find(person_id)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn create(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreatePerson)
  }
  async fn update(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    form: &PersonUpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person::table.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdatePerson)
  }
}

impl Person {
  /// Update or insert the person.
  ///
  /// This is necessary for federation, because Activitypub doesn't distinguish between these
  /// actions.
  pub async fn upsert(pool: &mut DbPool<'_>, form: &PersonInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person::table)
      .values(form)
      .on_conflict(person::ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdatePerson)
  }

  pub async fn delete_account(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Person> {
    let conn = &mut get_conn(pool).await?;

    // Set the local user email to none, only if they aren't banned locally.
    let instance_actions_join = instance_actions::table.on(
      instance_actions::person_id
        .eq(person_id)
        .and(instance_actions::instance_id.eq(local_instance_id)),
    );

    let not_banned_local_user_id = local_user::table
      .left_join(instance_actions_join)
      .filter(local_user::person_id.eq(person_id))
      .filter(instance_actions::received_ban_at.nullable().is_null())
      .select(local_user::id)
      .first::<LocalUserId>(conn)
      .await
      .optional()?;

    if let Some(local_user_id) = not_banned_local_user_id {
      diesel::update(local_user::table.find(local_user_id))
        .set(local_user::email.eq::<Option<String>>(None))
        .execute(conn)
        .await?;
    };

    diesel::update(person::table.find(person_id))
      .set((
        person::display_name.eq::<Option<String>>(None),
        person::avatar.eq::<Option<String>>(None),
        person::banner.eq::<Option<String>>(None),
        person::bio.eq::<Option<String>>(None),
        person::matrix_user_id.eq::<Option<String>>(None),
        person::deleted.eq(true),
        person::updated_at.eq(Utc::now()),
      ))
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdatePerson)
  }

  pub async fn check_username_taken(pool: &mut DbPool<'_>, username: &str) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      person::table
        .filter(lower(person::name).eq(username.to_lowercase()))
        .filter(person::local.eq(true)),
    )))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::UsernameAlreadyExists.into())
  }
}

impl PersonInsertForm {
  pub fn test_form(instance_id: InstanceId, name: &str) -> Self {
    Self::new(name.to_owned(), "pubkey".to_string(), instance_id)
  }
}

impl ApubActor for Person {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    person::table
      .filter(person::deleted.eq(false))
      .filter(person::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    from_name: &str,
    include_deleted: bool,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut q = person::table
      .into_boxed()
      .filter(person::local.eq(true))
      .filter(lower(person::name).eq(from_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(person::deleted.eq(false))
    }
    q.first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    person_name: &str,
    for_domain: &str,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    person::table
      .inner_join(instance::table)
      .filter(lower(person::name).eq(person_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(person::all_columns)
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  fn actor_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let domain = self
      .ap_id
      .inner()
      .domain()
      .ok_or(LemmyErrorType::NotFound)?;

    format_actor_url(&self.name, domain, 'u', settings)
  }

  fn generate_local_actor_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/u/{name}"))?.into())
  }
}

impl Followable for PersonActions {
  type Form = PersonFollowerForm;
  type IdType = PersonId;

  async fn follow(pool: &mut DbPool<'_>, form: &PersonFollowerForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)
  }

  /// Currently no user following
  async fn follow_accepted(_: &mut DbPool<'_>, _: CommunityId, _: PersonId) -> LemmyResult<Self> {
    Err(LemmyErrorType::NotFound.into())
  }

  async fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: Self::IdType,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((person_id, target_id)))
      .set_null(person_actions::followed_at)
      .set_null(person_actions::follow_pending)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)
  }
}

impl Blockable for PersonActions {
  type Form = PersonBlockForm;
  type ObjectIdType = PersonId;
  type ObjectType = Person;

  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::PersonBlockAlreadyExists)
  }

  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((form.person_id, form.target_id)))
      .set_null(person_actions::blocked_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::PersonBlockAlreadyExists)
  }

  async fn read_block(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    recipient_id: Self::ObjectIdType,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = person_actions::table
      .find((person_id, recipient_id))
      .filter(person_actions::blocked_at.is_not_null());

    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::PersonIsBlocked.into())
  }

  async fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<Vec<Self::ObjectType>> {
    let conn = &mut get_conn(pool).await?;
    let target_person_alias = diesel::alias!(person as person1);

    person_actions::table
      .filter(person_actions::blocked_at.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .inner_join(
        target_person_alias.on(person_actions::target_id.eq(target_person_alias.field(person::id))),
      )
      .select(target_person_alias.fields(person::all_columns))
      .filter(person_actions::person_id.eq(person_id))
      .filter(target_person_alias.field(person::deleted).eq(false))
      .order_by(person_actions::blocked_at)
      .load::<Person>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PersonActions {
  pub async fn follower_inboxes(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> LemmyResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;
    person_actions::table
      .filter(person_actions::followed_at.is_not_null())
      .inner_join(person::table.on(person_actions::person_id.eq(person::id)))
      .filter(person_actions::target_id.eq(for_person_id))
      .select(person::inbox_url)
      .distinct()
      .load(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn note(pool: &mut DbPool<'_>, form: &PersonNoteForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_actions::table)
      .values(form)
      .on_conflict((person_actions::person_id, person_actions::target_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn delete_note(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    target_id: PersonId,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(person_actions::table.find((person_id, target_id)))
      .set_null(person_actions::note)
      .set_null(person_actions::noted_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm, CommentUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonActions, PersonFollowerForm, PersonInsertForm, PersonUpdateForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Crud, Followable, Likeable},
    utils::{build_db_pool_for_tests, uplete},
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "holly");

    let inserted_person = Person::create(pool, &new_person).await?;

    let expected_person = Person {
      id: inserted_person.id,
      name: "holly".into(),
      display_name: None,
      avatar: None,
      banner: None,
      deleted: false,
      published_at: inserted_person.published_at,
      updated_at: None,
      ap_id: inserted_person.ap_id.clone(),
      bio: None,
      local: true,
      bot_account: false,
      private_key: None,
      public_key: "pubkey".to_owned(),
      last_refreshed_at: inserted_person.published_at,
      inbox_url: inserted_person.inbox_url.clone(),
      matrix_user_id: None,
      instance_id: inserted_instance.id,
      post_count: 0,
      post_score: 0,
      comment_count: 0,
      comment_score: 0,
    };

    let read_person = Person::read(pool, inserted_person.id).await?;

    let update_person_form = PersonUpdateForm {
      ap_id: Some(inserted_person.ap_id.clone()),
      ..Default::default()
    };
    let updated_person = Person::update(pool, inserted_person.id, &update_person_form).await?;

    let num_deleted = Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_person, read_person);
    assert_eq!(expected_person, inserted_person);
    assert_eq!(expected_person, updated_person);
    assert_eq!(1, num_deleted);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn follow() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_form_1 = PersonInsertForm::test_form(inserted_instance.id, "erich");
    let person_1 = Person::create(pool, &person_form_1).await?;
    let person_form_2 = PersonInsertForm::test_form(inserted_instance.id, "michele");
    let person_2 = Person::create(pool, &person_form_2).await?;

    let follow_form = PersonFollowerForm::new(person_1.id, person_2.id, false);
    let person_follower = PersonActions::follow(pool, &follow_form).await?;
    assert_eq!(person_1.id, person_follower.target_id);
    assert_eq!(person_2.id, person_follower.person_id);
    assert!(person_follower.follow_pending.is_some_and(|x| !x));

    let followers = PersonActions::follower_inboxes(pool, person_1.id).await?;
    assert_eq!(vec![person_2.inbox_url], followers);

    let unfollow =
      PersonActions::unfollow(pool, follow_form.person_id, follow_form.target_id).await?;
    assert_eq!(uplete::Count::only_deleted(1), unfollow);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_user_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_user_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_site_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);
    let _inserted_post_like = PostActions::like(pool, &post_like).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let mut comment_like = CommentLikeForm::new(inserted_person.id, inserted_comment.id, 1);

    let _inserted_comment_like = CommentActions::like(pool, &comment_like).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let child_comment_like =
      CommentLikeForm::new(another_inserted_person.id, inserted_child_comment.id, 1);

    let _inserted_child_comment_like = CommentActions::like(pool, &child_comment_like).await?;

    let person_aggregates_before_delete = Person::read(pool, inserted_person.id).await?;

    assert_eq!(1, person_aggregates_before_delete.post_count);
    assert_eq!(1, person_aggregates_before_delete.post_score);
    assert_eq!(2, person_aggregates_before_delete.comment_count);
    assert_eq!(2, person_aggregates_before_delete.comment_score);

    // Remove a post like
    PostActions::remove_like(pool, inserted_person.id, inserted_post.id).await?;
    let after_post_like_remove = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_post_like_remove.post_score);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;
    Comment::update(
      pool,
      inserted_child_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let after_parent_comment_removed = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_removed.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_removed.comment_score);

    // Remove a parent comment (the scores should also be removed)
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_parent_comment_delete = Person::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_delete.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_delete.comment_score);

    // Add in the two comments again, then delete the post.
    let new_parent_comment = Comment::create(pool, &comment_form, None).await?;
    let _new_child_comment =
      Comment::create(pool, &child_comment_form, Some(&new_parent_comment.path)).await?;
    comment_like.comment_id = new_parent_comment.id;
    CommentActions::like(pool, &comment_like).await?;
    let after_comment_add = Person::read(pool, inserted_person.id).await?;
    assert_eq!(2, after_comment_add.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(1, after_comment_add.comment_score);

    Post::delete(pool, inserted_post.id).await?;
    let after_post_delete = Person::read(pool, inserted_person.id).await?;
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_post_delete.comment_score);
    assert_eq!(0, after_post_delete.comment_count);
    assert_eq!(0, after_post_delete.post_score);
    assert_eq!(0, after_post_delete.post_count);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);
    Person::delete(pool, another_inserted_person.id).await?;

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Should be none found
    let after_delete = Person::read(pool, inserted_person.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
