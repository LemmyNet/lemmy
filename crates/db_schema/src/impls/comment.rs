use crate::{
  newtypes::{CommentId, DbUrl, PersonId},
  source::comment::{
    Comment,
    CommentForm,
    CommentLike,
    CommentLikeForm,
    CommentSaved,
    CommentSavedForm,
  },
  traits::{Crud, DeleteableOrRemoveable, Likeable, Saveable},
  utils::naive_now,
};
use diesel::{dsl::*, result::Error, *};
use diesel_ltree::Ltree;
use url::Url;

impl Comment {
  pub fn update_ap_id(
    conn: &PgConnection,
    comment_id: CommentId,
    apub_id: DbUrl,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;

    diesel::update(comment.find(comment_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  pub fn permadelete_for_creator(
    conn: &PgConnection,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((
        content.eq("*Permananently Deleted*"),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
  }

  pub fn update_deleted(
    conn: &PgConnection,
    comment_id: CommentId,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed(
    conn: &PgConnection,
    comment_id: CommentId,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: PersonId,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
  }

  pub fn update_content(
    conn: &PgConnection,
    comment_id: CommentId,
    new_content: &str,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((content.eq(new_content), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn create(
    conn: &PgConnection,
    comment_form: &CommentForm,
    parent_path: Option<&Ltree>,
  ) -> Result<Comment, Error> {
    use crate::schema::comment::dsl::*;

    // Insert, to get the id
    let inserted_comment = insert_into(comment)
      .values(comment_form)
      .on_conflict(ap_id)
      .do_update()
      .set(comment_form)
      .get_result::<Self>(conn);

    if let Ok(comment_insert) = inserted_comment {
      let comment_id = comment_insert.id;

      // You need to update the ltree column
      let ltree = Ltree(if let Some(parent_path) = parent_path {
        // The previous parent will already have 0 in it
        // Append this comment id
        format!("{}.{}", parent_path.0, comment_id)
      } else {
        // '0' is always the first path, append to that
        format!("{}.{}", 0, comment_id)
      });

      let updated_comment = diesel::update(comment.find(comment_id))
        .set(path.eq(ltree))
        .get_result::<Self>(conn);

      // Update the child count for the parent comment_aggregates
      // You could do this with a trigger, but since you have to do this manually anyway,
      // you can just have it here
      if let Some(parent_path) = parent_path {
        // You have to update counts for all parents, not just the immediate one
        // TODO if the performance of this is terrible, it might be better to do this as part of a
        // scheduled query... although the counts would often be wrong.
        //
        // The child_count query for reference:
        // select c.id, c.path, count(c2.id) as child_count from comment c
        // left join comment c2 on c2.path <@ c.path and c2.path != c.path
        // group by c.id

        let top_parent = format!("0.{}", parent_path.0.split('.').collect::<Vec<&str>>()[1]);
        let update_child_count_stmt = format!(
          "
update comment_aggregates ca set child_count = c.child_count
from (
  select c.id, c.path, count(c2.id) as child_count from comment c
  join comment c2 on c2.path <@ c.path and c2.path != c.path
  and c.path <@ '{}'
  group by c.id
) as c
where ca.comment_id = c.id",
          top_parent
        );

        sql_query(update_child_count_stmt).execute(conn)?;
      }
      updated_comment
    } else {
      inserted_comment
    }
  }
  pub fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, Error> {
    use crate::schema::comment::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      comment
        .filter(ap_id.eq(object_id))
        .first::<Comment>(conn)
        .ok()
        .map(Into::into),
    )
  }

  pub fn parent_comment_id(&self) -> Option<CommentId> {
    let mut ltree_split: Vec<&str> = self.path.0.split('.').collect();
    ltree_split.remove(0); // The first is always 0
    if ltree_split.len() > 1 {
      ltree_split[ltree_split.len() - 2]
        .parse::<i32>()
        .map(CommentId)
        .ok()
    } else {
      None
    }
  }
}

impl Crud for Comment {
  type Form = CommentForm;
  type IdType = CommentId;
  fn read(conn: &PgConnection, comment_id: CommentId) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    comment.find(comment_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, comment_id: CommentId) -> Result<usize, Error> {
    use crate::schema::comment::dsl::*;
    diesel::delete(comment.find(comment_id)).execute(conn)
  }

  /// This is unimplemented, use [[Comment::create]]
  fn create(_conn: &PgConnection, _comment_form: &CommentForm) -> Result<Self, Error> {
    unimplemented!();
  }

  fn update(
    conn: &PgConnection,
    comment_id: CommentId,
    comment_form: &CommentForm,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
  }
}

impl Likeable for CommentLike {
  type Form = CommentLikeForm;
  type IdType = CommentId;
  fn like(conn: &PgConnection, comment_like_form: &CommentLikeForm) -> Result<Self, Error> {
    use crate::schema::comment_like::dsl::*;
    insert_into(comment_like)
      .values(comment_like_form)
      .on_conflict((comment_id, person_id))
      .do_update()
      .set(comment_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(
    conn: &PgConnection,
    person_id: PersonId,
    comment_id: CommentId,
  ) -> Result<usize, Error> {
    use crate::schema::comment_like::dsl;
    diesel::delete(
      dsl::comment_like
        .filter(dsl::comment_id.eq(comment_id))
        .filter(dsl::person_id.eq(person_id)),
    )
    .execute(conn)
  }
}

impl Saveable for CommentSaved {
  type Form = CommentSavedForm;
  fn save(conn: &PgConnection, comment_saved_form: &CommentSavedForm) -> Result<Self, Error> {
    use crate::schema::comment_saved::dsl::*;
    insert_into(comment_saved)
      .values(comment_saved_form)
      .on_conflict((comment_id, person_id))
      .do_update()
      .set(comment_saved_form)
      .get_result::<Self>(conn)
  }
  fn unsave(conn: &PgConnection, comment_saved_form: &CommentSavedForm) -> Result<usize, Error> {
    use crate::schema::comment_saved::dsl::*;
    diesel::delete(
      comment_saved
        .filter(comment_id.eq(comment_saved_form.comment_id))
        .filter(person_id.eq(comment_saved_form.person_id)),
    )
    .execute(conn)
  }
}

impl DeleteableOrRemoveable for Comment {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.content = "".into();
    self
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      comment::*,
      community::{Community, CommunityForm},
      person::{Person, PersonForm},
      post::*,
    },
    traits::{Crud, Likeable, Saveable},
    utils::establish_unpooled_connection,
  };
  use diesel_ltree::Ltree;
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "terry".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let new_community = CommunityForm {
      name: "test community".to_string(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment = Comment::create(&conn, &comment_form, None).unwrap();

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
      ap_id: inserted_comment.ap_id.to_owned(),
      local: true,
    };

    let child_comment_form = CommentForm {
      content: "A child comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      // path: Some(text2ltree(inserted_comment.id),
      ..CommentForm::default()
    };

    let inserted_child_comment =
      Comment::create(&conn, &child_comment_form, Some(&inserted_comment.path)).unwrap();

    // Comment Like
    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_comment_like = CommentLike::like(&conn, &comment_like_form).unwrap();

    let expected_comment_like = CommentLike {
      id: inserted_comment_like.id,
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

    let inserted_comment_saved = CommentSaved::save(&conn, &comment_saved_form).unwrap();

    let expected_comment_saved = CommentSaved {
      id: inserted_comment_saved.id,
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      published: inserted_comment_saved.published,
    };

    let read_comment = Comment::read(&conn, inserted_comment.id).unwrap();
    let updated_comment = Comment::update(&conn, inserted_comment.id, &comment_form).unwrap();
    let like_removed = CommentLike::remove(&conn, inserted_person.id, inserted_comment.id).unwrap();
    let saved_removed = CommentSaved::unsave(&conn, &comment_saved_form).unwrap();
    let num_deleted = Comment::delete(&conn, inserted_comment.id).unwrap();
    Comment::delete(&conn, inserted_child_comment.id).unwrap();
    Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    Person::delete(&conn, inserted_person.id).unwrap();

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
