use crate::{
  newtypes::{CommentId, PersonId, PostId},
  source::{
    combined::person_saved::{
      PersonSavedCombined,
      PersonSavedCombinedCommentInsertForm,
      PersonSavedCombinedPostInsertForm,
    },
    history_status::{HistoryStatus, HistoryStatusUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::count_star,
  insert_into,
  upsert::excluded,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_db_schema_file::schema::{
  comment_actions,
  local_user,
  person_saved_combined,
  post_actions,
};
use lemmy_utils::{error::LemmyResult, DB_BATCH_SIZE};
use tracing::info;

impl PersonSavedCombined {
  pub async fn fill_post_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling post_actions history into person_saved_combined...");

    // Get the total count of post rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("post_actions".into(), "person_saved_combined".into());
    let mut last_scanned_id = HistoryStatus::read(pool, history_key())
      .await?
      .last_scanned_id
      .unwrap_or(i32::MAX);

    let conn = &mut get_conn(pool).await?;

    // Actions should only be saved for local users
    let local_user_join = local_user::table.on(local_user::person_id.eq(post_actions::person_id));

    let post_actions_count = post_actions::table
      .inner_join(local_user_join)
      .select(count_star())
      .filter(post_actions::id.lt(last_scanned_id))
      .filter(post_actions::saved_at.is_not_null())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < post_actions_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into forms
            let actions = post_actions::table
              .inner_join(local_user_join)
              .select((
                post_actions::saved_at.assume_not_null(),
                post_actions::person_id,
                post_actions::post_id,
                post_actions::id,
              ))
              .filter(post_actions::id.lt(last_scanned_id))
              .filter(post_actions::saved_at.is_not_null())
              .order_by(post_actions::id.desc())
              .limit(DB_BATCH_SIZE)
              .get_results::<(DateTime<Utc>, PersonId, PostId, i32)>(conn)
              .await?;

            let forms = actions
              .iter()
              .map(|cl| PersonSavedCombinedPostInsertForm {
                saved_at: cl.0,
                person_id: cl.1,
                post_id: cl.2,
              })
              .collect::<Vec<PersonSavedCombinedPostInsertForm>>();

            let inserted_count = insert_into(person_saved_combined::table)
              .values(forms)
              .on_conflict((
                person_saved_combined::person_id,
                person_saved_combined::post_id,
              ))
              .do_update()
              .set(person_saved_combined::saved_at.eq(excluded(person_saved_combined::saved_at)))
              .execute(conn)
              .await?;

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = actions.last().map(|f| f.3).unwrap_or(i32::MAX);

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / post_actions_count;
      info!(
        "post_actions -> person_saved_combined: {processed_rows} / {post_actions_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling post_actions history into person_saved_combined.");
    Ok(())
  }

  pub async fn fill_comment_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling comment_actions history into person_saved_combined...");

    // Get the total count of comment rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("comment_actions".into(), "person_saved_combined".into());
    let mut last_scanned_id = HistoryStatus::read(pool, history_key())
      .await?
      .last_scanned_id
      .unwrap_or(i32::MAX);

    let conn = &mut get_conn(pool).await?;

    // Actions should only be saved for local users
    let local_user_join =
      local_user::table.on(local_user::person_id.eq(comment_actions::person_id));

    let comment_actions_count = comment_actions::table
      .inner_join(local_user_join)
      .select(count_star())
      .filter(comment_actions::id.lt(last_scanned_id))
      .filter(comment_actions::saved_at.is_not_null())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_actions_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into forms
            let actions = comment_actions::table
              .inner_join(local_user_join)
              .select((
                comment_actions::saved_at.assume_not_null(),
                comment_actions::person_id,
                comment_actions::comment_id,
                comment_actions::id,
              ))
              .filter(comment_actions::id.lt(last_scanned_id))
              .filter(comment_actions::saved_at.is_not_null())
              .order_by(comment_actions::id.desc())
              .limit(DB_BATCH_SIZE)
              .get_results::<(DateTime<Utc>, PersonId, CommentId, i32)>(conn)
              .await?;

            let forms = actions
              .iter()
              .map(|cl| PersonSavedCombinedCommentInsertForm {
                saved_at: cl.0,
                person_id: cl.1,
                comment_id: cl.2,
              })
              .collect::<Vec<PersonSavedCombinedCommentInsertForm>>();

            let inserted_count = insert_into(person_saved_combined::table)
              .values(forms)
              .on_conflict((
                person_saved_combined::person_id,
                person_saved_combined::comment_id,
              ))
              .do_update()
              .set(person_saved_combined::saved_at.eq(excluded(person_saved_combined::saved_at)))
              .execute(conn)
              .await?;

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = actions.last().map(|f| f.3).unwrap_or(i32::MAX);

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / comment_actions_count;
      info!(
        "comment_actions -> person_saved_combined: {processed_rows} / {comment_actions_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling comment_actions history into person_saved_combined.");
    Ok(())
  }
}
