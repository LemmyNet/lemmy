use crate::{
  newtypes::{CommentId, DbUrl, PersonId},
  schema::comment::dsl::{
    ap_id,
    comment,
    content,
    creator_id,
    deleted,
    id,
    path,
    removed,
    updated,
  },
  source::comment::{
    Comment,
    CommentInsertForm,
    CommentLike,
    CommentLikeForm,
    CommentSaved,
    CommentSavedForm,
    CommentUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{functions::AsText, get_conn, naive_now, DbPool, DELETED_REPLACEMENT_TEXT},
};
use diesel::{
  dsl::{insert_into, sql_query},
  result::Error,
  ExpressionMethods,
  QueryDsl,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{
  functions::{ltree2text, text2ltree},
  Ltree,
};
use futures_util::{
  future::TryFutureExt,
  stream::{self, StreamExt, TryStreamExt},
};
use url::Url;

impl Comment {
  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((
        content.eq(DELETED_REPLACEMENT_TEXT),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    comment_form: &CommentInsertForm,
    parent_path: Option<&Ltree>,
  ) -> Result<Comment, Error> {
    Comment::create_batch(pool, &[(comment_form.clone(), parent_path.cloned())])
      .await?
      .into_iter()
      .next()
      .ok_or(Error::NotFound)
  }

  pub async fn create_batch(
    pool: &mut DbPool<'_>,
    items: &[(CommentInsertForm, Option<Ltree>)],
  ) -> Result<Vec<Comment>, Error> {
    let conn = &mut get_conn(pool).await?;

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let forms = items
            .iter()
            .map(|(comment_form, parent_path)| CommentInsertForm {
              path: Some(parent_path.clone().unwrap_or(Ltree("0".to_owned()))),
              ..comment_form.clone()
            });

          // Insert, to get the ids
          let inserted_comments = insert_into(comment)
            .values(forms.clone().collect::<Vec<_>>())
            .load::<Self>(conn)
            .or_else(|_| {
              // `ap_id` unique constraint violation is handled individually for each row
              // because batched upsert requires having the same `set` argument for all rows
              stream::iter(forms)
                .then(|form| {
                  insert_into(comment)
                    .values(form.clone())
                    .on_conflict(ap_id)
                    .do_update()
                    .set(form)
                    .get_result::<Self>(conn)
                })
                .try_collect::<Vec<_>>()
            })
            .await?;

          // For each comment, append its id to its path
          let updated_comments = diesel::update(comment)
            .filter(id.eq_any(inserted_comments.into_iter().map(|c| c.id)))
            .set(path.eq(text2ltree(
              ltree2text(path).concat(".").concat(AsText::new(id)),
            )))
            .load::<Self>(conn)
            .await?;

          // Update the child count for the parent comment_aggregates
          // You could do this with a trigger, but since you have to do this manually anyway,
          // you can just have it here
          for parent_path in items.iter().filter_map(|(_, p)| p.as_ref()) {
            // You have to update counts for all parents, not just the immediate one
            // TODO if the performance of this is terrible, it might be better to do this as part of a
            // scheduled query... although the counts would often be wrong.
            //
            // The child_count query for reference:
            // select c.id, c.path, count(c2.id) as child_count from comment c
            // left join comment c2 on c2.path <@ c.path and c2.path != c.path
            // group by c.id

            let parent_id = parent_path.0.split('.').nth(1);

            if let Some(parent_id) = parent_id {
              let top_parent = format!("0.{}", parent_id);
              let update_child_count_stmt = format!(
                "
update comment_aggregates ca set child_count = c.child_count
from (
  select c.id, c.path, count(c2.id) as child_count from comment c
  join comment c2 on c2.path <@ c.path and c2.path != c.path
  and c.path <@ '{top_parent}'
  group by c.id
) as c
where ca.comment_id = c.id"
              );

              sql_query(update_child_count_stmt).execute(conn).await?;
            }
          }
          Ok(updated_comments)
        }) as _
      })
      .await
  }
  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    Ok(
      comment
        .filter(ap_id.eq(object_id))
        .first::<Comment>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  pub fn parent_comment_id(&self) -> Option<CommentId> {
    let mut ltree_split: Vec<&str> = self.path.0.split('.').collect();
    ltree_split.remove(0); // The first is always 0
    if ltree_split.len() > 1 {
      let parent_comment_id = ltree_split.get(ltree_split.len() - 2);
      parent_comment_id.and_then(|p| p.parse::<i32>().map(CommentId).ok())
    } else {
      None
    }
  }
}

#[async_trait]
impl Crud for Comment {
  type InsertForm = CommentInsertForm;
  type UpdateForm = CommentUpdateForm;
  type IdType = CommentId;

  /// This is unimplemented, use [[Comment::create]]
  async fn create(_pool: &mut DbPool<'_>, _comment_form: &Self::InsertForm) -> Result<Self, Error> {
    unimplemented!();
  }

  async fn update(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    comment_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Likeable for CommentLike {
  type Form = CommentLikeForm;
  type IdType = CommentId;
  async fn like(pool: &mut DbPool<'_>, comment_like_form: &CommentLikeForm) -> Result<Self, Error> {
    use crate::schema::comment_like::dsl::{comment_id, comment_like, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_like)
      .values(comment_like_form)
      .on_conflict((comment_id, person_id))
      .do_update()
      .set(comment_like_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn remove(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    comment_id: CommentId,
  ) -> Result<usize, Error> {
    use crate::schema::comment_like::dsl::comment_like;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(comment_like.find((person_id, comment_id)))
      .execute(conn)
      .await
  }
}

#[async_trait]
impl Saveable for CommentSaved {
  type Form = CommentSavedForm;
  async fn save(
    pool: &mut DbPool<'_>,
    comment_saved_form: &CommentSavedForm,
  ) -> Result<Self, Error> {
    use crate::schema::comment_saved::dsl::{comment_id, comment_saved, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_saved)
      .values(comment_saved_form)
      .on_conflict((comment_id, person_id))
      .do_update()
      .set(comment_saved_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unsave(
    pool: &mut DbPool<'_>,
    comment_saved_form: &CommentSavedForm,
  ) -> Result<usize, Error> {
    use crate::schema::comment_saved::dsl::comment_saved;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      comment_saved.find((comment_saved_form.person_id, comment_saved_form.comment_id)),
    )
    .execute(conn)
    .await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    newtypes::LanguageId,
    source::{
      comment::{
        Comment,
        CommentInsertForm,
        CommentLike,
        CommentLikeForm,
        CommentSaved,
        CommentSavedForm,
        CommentUpdateForm,
      },
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Crud, Likeable, Saveable},
    utils::build_db_pool_for_tests,
  };
  use diesel_ltree::Ltree;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("terry".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    let expected_comment = Comment {
      id: inserted_comment.id,
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      removed: false,
      deleted: false,
      path: Ltree(format!("0.{}", inserted_comment.id)),
      published: inserted_comment.published,
      updated: None,
      ap_id: inserted_comment.ap_id.clone(),
      distinguished: false,
      local: true,
      language_id: LanguageId::default(),
    };

    let child_comment_form = CommentInsertForm::builder()
      .content("A child comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path))
        .await
        .unwrap();

    // Comment Like
    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_comment_like = CommentLike::like(pool, &comment_like_form).await.unwrap();

    let expected_comment_like = CommentLike {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_comment_like.published,
      score: 1,
    };

    // Comment Saved
    let comment_saved_form = CommentSavedForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
    };

    let inserted_comment_saved = CommentSaved::save(pool, &comment_saved_form).await.unwrap();

    let expected_comment_saved = CommentSaved {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      published: inserted_comment_saved.published,
    };

    let comment_update_form = CommentUpdateForm {
      content: Some("A test comment".into()),
      ..Default::default()
    };

    let updated_comment = Comment::update(pool, inserted_comment.id, &comment_update_form)
      .await
      .unwrap();

    let read_comment = Comment::read(pool, inserted_comment.id).await.unwrap();
    let like_removed = CommentLike::remove(pool, inserted_person.id, inserted_comment.id)
      .await
      .unwrap();
    let saved_removed = CommentSaved::unsave(pool, &comment_saved_form)
      .await
      .unwrap();
    let num_deleted = Comment::delete(pool, inserted_comment.id).await.unwrap();
    Comment::delete(pool, inserted_child_comment.id)
      .await
      .unwrap();
    Post::delete(pool, inserted_post.id).await.unwrap();
    Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, inserted_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(expected_comment_like, inserted_comment_like);
    assert_eq!(expected_comment_saved, inserted_comment_saved);
    assert_eq!(
      format!("0.{}.{}", expected_comment.id, inserted_child_comment.id),
      inserted_child_comment.path.0,
    );
    assert_eq!(1, like_removed);
    assert_eq!(1, saved_removed);
    assert_eq!(1, num_deleted);
  }
}
