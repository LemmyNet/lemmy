use crate::PersonView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  PersonListingType,
  PersonSortType,
  impls::local_user::LocalUserOptionHelper,
  source::{
    local_user::LocalUser,
    person::{Person, person_keys as key},
    site::Site,
  },
  utils::limit_fetch,
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  joins::{
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
    my_person_actions_join,
  },
  schema::{local_user, person},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
  traits::Crud,
  utils::fuzzy_search,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl PaginationCursorConversion for PersonView {
  type PaginatedType = Person;

  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.person.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Person::read(pool, PersonId(cursor.id()?)).await
  }
}

impl PersonView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);

    person::table
      .left_join(local_user::table)
      .left_join(my_person_actions_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    my_person_id: Option<PersonId>,
    local_instance_id: InstanceId,
    is_admin: bool,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins(my_person_id, local_instance_id)
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .into_boxed();

    if !is_admin {
      query = query.filter(person::deleted.eq(false))
    }

    query
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn list_admins(
    my_person_id: Option<PersonId>,
    local_instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<PersonView>> {
    let conn = &mut get_conn(pool).await?;

    Self::joins(my_person_id, local_instance_id)
      .filter(person::deleted.eq(false))
      .filter(local_user::admin)
      // Order by admin created date (ie old)
      .then_order_by(person::published_at.asc())
      // Tie breaker
      .then_order_by(person::id)
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[derive(Default)]
pub struct PersonQuery<'a> {
  pub local_user: Option<&'a LocalUser>,
  pub sort: Option<PersonSortType>,
  pub listing_type: Option<PersonListingType>,
  pub search_term: Option<String>,
  pub search_title_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl PersonQuery<'_> {
  pub async fn list(
    self,
    site: &Site,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PagedResponse<PersonView>> {
    use PersonSortType::*;

    let o = self;
    let limit = limit_fetch(o.limit, None)?;

    let mut query = PersonView::joins(o.local_user.person_id(), site.instance_id)
      .select(PersonView::as_select())
      .limit(limit)
      .filter(person::deleted.eq(false))
      .into_boxed();

    if let Some(listing_type) = o.listing_type {
      query = match listing_type {
        PersonListingType::All => query,
        PersonListingType::Local => query.filter(person::local),
      };
    }

    // The search term
    if let Some(search_term) = o.search_term {
      let searcher = fuzzy_search(&search_term);

      let name_or_title_filter = person::name
        .ilike(searcher.clone())
        .or(person::display_name.ilike(searcher.clone()));

      query = if o.search_title_only.unwrap_or_default() {
        query.filter(name_or_title_filter)
      } else {
        let body_or_description_filter = person::bio.ilike(searcher.clone());
        query.filter(name_or_title_filter.or(body_or_description_filter))
      }
    }

    // Only sort by ascending for Old
    let sort = o.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old);

    let mut pq = PersonView::paginate(query, &o.page_cursor, sort_direction, pool, None).await?;
    pq = match o.sort.unwrap_or_default() {
      New | Old => pq.then_order_by(key::published_at),
      PostScore => pq.then_order_by(key::post_score),
      CommentScore => pq.then_order_by(key::comment_score),
    }
    // Tie breaker
    .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = pq
      .load::<PersonView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, o.page_cursor)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use lemmy_db_schema::{
    assert_length,
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonActions, PersonInsertForm, PersonNoteForm, PersonUpdateForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
      site::SiteInsertForm,
    },
    traits::Likeable,
  };
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    site: Site,
    alice: Person,
    alice_local_user: LocalUser,
    bob: Person,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let site_form = SiteInsertForm::new("test_site".to_string(), instance.id);
    let site = Site::create(pool, &site_form).await?;

    let alice_form = PersonInsertForm {
      local: Some(true),
      ..PersonInsertForm::test_form(instance.id, "alice")
    };
    let alice = Person::create(pool, &alice_form).await?;
    let alice_local_user_form = LocalUserInsertForm::test_form(alice.id);
    let alice_local_user = LocalUser::create(pool, &alice_local_user_form, vec![]).await?;

    let bob_form = PersonInsertForm {
      bot_account: Some(true),
      local: Some(false),
      ..PersonInsertForm::test_form(instance.id, "bob")
    };
    let bob = Person::create(pool, &bob_form).await?;

    // Create a post like to give bob a higher post score for sorting tests
    let community_form = CommunityInsertForm {
      summary: Some("bobs comm".into()),
      ..CommunityInsertForm::new(
        instance.id,
        "bobs_comm".to_string(),
        "Bobs Comm".to_owned(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;
    let bob_post_form = PostInsertForm {
      body: Some("person query list inside here".into()),
      ..PostInsertForm::new("bob post".into(), bob.id, community.id)
    };
    let bob_post = Post::create(pool, &bob_post_form).await?;
    let bob_like_post_form = PostLikeForm::new(bob_post.id, bob.id, Some(true));
    PostActions::like(pool, &bob_like_post_form).await?;

    Ok(Data {
      site,
      alice,
      alice_local_user,
      bob,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.bob.instance_id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn exclude_deleted() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Person::update(
      pool,
      data.alice.id,
      &PersonUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let read = PersonView::read(pool, data.alice.id, None, data.alice.instance_id, false).await;
    assert!(read.is_err());

    // only admin can view deleted users
    let read = PersonView::read(pool, data.alice.id, None, data.alice.instance_id, true).await;
    assert!(read.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn list_admins() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    LocalUser::update(
      pool,
      data.alice_local_user.id,
      &LocalUserUpdateForm {
        admin: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let list = PersonView::list_admins(None, data.alice.instance_id, pool).await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    let is_admin = PersonView::read(pool, data.alice.id, None, data.alice.instance_id, false)
      .await?
      .is_admin;
    assert!(is_admin);

    let is_admin = PersonView::read(pool, data.bob.id, None, data.alice.instance_id, false)
      .await?
      .is_admin;
    assert!(!is_admin);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn note() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let note_str = "Bob hates cats.";

    let note_form = PersonNoteForm::new(data.alice.id, data.bob.id, note_str.to_string());
    let inserted_note = PersonActions::note(pool, &note_form).await?;
    assert_eq!(Some(note_str.to_string()), inserted_note.note);

    let read = PersonView::read(
      pool,
      data.bob.id,
      Some(data.alice.id),
      data.alice.instance_id,
      false,
    )
    .await?;

    assert!(
      read
        .person_actions
        .is_some_and(|t| t.note == Some(note_str.to_string()) && t.noted_at.is_some())
    );

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn search() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Person search
    let list_all = PersonQuery::default().list(&data.site, pool).await?;
    assert_length!(2, list_all);
    assert_eq!(data.bob.id, list_all[0].person.id);
    assert_eq!(data.alice.id, list_all[1].person.id);

    // Using a term
    let search_by_name = PersonQuery {
      search_term: Some("alice".into()),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_length!(1, search_by_name);
    assert_eq!(data.alice.id, search_by_name[0].person.id);

    // Post score sorting score
    let list_sort_top = PersonQuery {
      sort: Some(PersonSortType::PostScore),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(2, list_sort_top);
    // Bobs should be at the top
    assert_eq!(data.bob.id, list_sort_top[0].person.id);

    cleanup(data, pool).await?;

    Ok(())
  }
}
