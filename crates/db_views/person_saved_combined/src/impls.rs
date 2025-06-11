use crate::{
  CommentView,
  LocalUserView,
  PersonSavedCombinedView,
  PersonSavedCombinedViewInternal,
  PostView,
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::{InstanceId, PaginationCursor, PersonId},
  source::combined::person_saved::{person_saved_combined_keys as key, PersonSavedCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      community_join,
      creator_community_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      creator_local_user_admin_join,
      image_details_join,
      my_comment_actions_join,
      my_community_actions_join,
      my_instance_actions_person_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    DbPool,
  },
  PersonContentType,
};
use lemmy_db_schema_file::schema::{comment, person, person_saved_combined, post};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[derive(Default)]
pub struct PersonSavedCombinedQuery {
  pub type_: Option<PersonContentType>,
  pub cursor_data: Option<PersonSavedCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl PaginationCursorBuilder for PersonSavedCombinedView {
  type CursorData = PersonSavedCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      PersonSavedCombinedView::Comment(v) => ('C', v.comment.id.0),
      PersonSavedCombinedView::Post(v) => ('P', v.post.id.0),
    };
    PaginationCursor::new_single(prefix, id)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let pids = cursor.prefixes_and_ids();
    let (prefix, id) = pids
      .as_slice()
      .first()
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;

    let mut query = person_saved_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(person_saved_combined::comment_id.eq(id)),
      'P' => query.filter(person_saved_combined::post_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

impl PersonSavedCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  pub(crate) fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let comment_join =
      comment::table.on(person_saved_combined::comment_id.eq(comment::id.nullable()));

    let post_join = post::table.on(
      person_saved_combined::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id)),
    );

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        // Need to filter out the post rows where the post_id given is null
        // Otherwise you'll get duped post rows
        .or(
          post::creator_id
            .eq(item_creator)
            .and(person_saved_combined::post_id.is_not_null()),
        ),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    person_saved_combined::table
      .left_join(comment_join)
      .inner_join(post_join)
      .inner_join(item_creator_join)
      .inner_join(community_join())
      .left_join(creator_community_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_community_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(image_details_join())
  }
}

impl PersonSavedCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<PersonSavedCombinedView>> {
    let my_person_id = user.local_user.person_id;
    let local_instance_id = user.person.instance_id;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let mut query = PersonSavedCombinedViewInternal::joins(my_person_id, local_instance_id)
      .filter(person_saved_combined::person_id.eq(my_person_id))
      .select(PersonSavedCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(type_) = self.type_ {
      query = match type_ {
        PersonContentType::All => query,
        PersonContentType::Comments => {
          query.filter(person_saved_combined::comment_id.is_not_null())
        }
        PersonContentType::Posts => query.filter(person_saved_combined::post_id.is_not_null()),
      }
    }

    // Sorting by saved desc
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::saved_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<PersonSavedCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for PersonSavedCombinedViewInternal {
  type CombinedView = PersonSavedCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let Some(comment) = v.comment {
      Some(PersonSavedCombinedView::Comment(CommentView {
        comment,
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        community_actions: v.community_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_home_instance_actions: v.creator_home_instance_actions,
        creator_local_instance_actions: v.creator_local_instance_actions,
        creator_community_actions: v.creator_community_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
      }))
    } else {
      Some(PersonSavedCombinedView::Post(PostView {
        post: v.post,
        community: v.community,
        creator: v.item_creator,
        image_details: v.image_details,
        community_actions: v.community_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        instance_actions: v.instance_actions,
        creator_home_instance_actions: v.creator_home_instance_actions,
        creator_local_instance_actions: v.creator_local_instance_actions,
        creator_community_actions: v.creator_community_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
      }))
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{impls::PersonSavedCombinedQuery, LocalUserView, PersonSavedCombinedView};
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentSavedForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostSavedForm},
    },
    traits::{Crud, Saveable},
    utils::{build_db_pool_for_tests, DbPool},
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    timmy_view: LocalUserView,
    sara: Person,
    timmy_post: Post,
    sara_comment: Comment,
    sara_comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: timmy.clone(),
      instance_actions: None,
    };

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_pcv");
    let sara = Person::create(pool, &sara_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community pcv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let timmy_post_form = PostInsertForm::new("timmy post prv".into(), timmy.id, community.id);
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let timmy_post_form_2 = PostInsertForm::new("timmy post prv 2".into(), timmy.id, community.id);
    let timmy_post_2 = Post::create(pool, &timmy_post_form_2).await?;

    let sara_post_form = PostInsertForm::new("sara post prv".into(), sara.id, community.id);
    let _sara_post = Post::create(pool, &sara_post_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv".into());
    let _timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.id, timmy_post.id, "sara comment prv".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

    let sara_comment_form_2 =
      CommentInsertForm::new(sara.id, timmy_post_2.id, "sara comment prv 2".into());
    let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

    Ok(Data {
      instance,
      timmy,
      timmy_view,
      sara,
      timmy_post,
      sara_comment,
      sara_comment_2,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_combined() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Do a batch read of timmy saved
    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(0, timmy_saved.len());

    // Save a few things
    let save_sara_comment_2 =
      CommentSavedForm::new(data.timmy_view.person.id, data.sara_comment_2.id);
    CommentActions::save(pool, &save_sara_comment_2).await?;

    let save_sara_comment = CommentSavedForm::new(data.timmy_view.person.id, data.sara_comment.id);
    CommentActions::save(pool, &save_sara_comment).await?;

    let post_save_form = PostSavedForm::new(data.timmy_post.id, data.timmy.id);
    PostActions::save(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(3, timmy_saved.len());

    // Make sure the types and order are correct
    if let PersonSavedCombinedView::Post(v) = &timmy_saved[0] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PersonSavedCombinedView::Comment(v) = &timmy_saved[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }
    if let PersonSavedCombinedView::Comment(v) = &timmy_saved[2] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    // Try unsaving 2 things
    CommentActions::unsave(pool, &save_sara_comment).await?;
    PostActions::unsave(pool, &post_save_form).await?;

    let timmy_saved = PersonSavedCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(1, timmy_saved.len());

    if let PersonSavedCombinedView::Comment(v) = &timmy_saved[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.sara.id, v.comment.creator_id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
