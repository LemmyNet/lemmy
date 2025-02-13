use crate::{
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  newtypes::PaginationCursor,
  schema::modlog_combined,
  source::combined::modlog::ModlogCombined,
  traits::PageCursorReader,
  utils::DbConn,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[async_trait]
impl PageCursorReader for ModlogCombined {
  async fn from_cursor(cursor: PaginationCursor, conn: &mut DbConn<'_>) -> LemmyResult<Self> {
    let (prefix, id) = cursor.prefix_and_id()?;

    let mut query = modlog_combined::table
      .select(Self::as_select())
      .into_boxed();

    query = match prefix {
      'A' => query.filter(modlog_combined::admin_allow_instance_id.eq(id)),
      'B' => query.filter(modlog_combined::admin_block_instance_id.eq(id)),
      'C' => query.filter(modlog_combined::admin_purge_comment_id.eq(id)),
      'D' => query.filter(modlog_combined::admin_purge_community_id.eq(id)),
      'E' => query.filter(modlog_combined::admin_purge_person_id.eq(id)),
      'F' => query.filter(modlog_combined::admin_purge_post_id.eq(id)),
      'G' => query.filter(modlog_combined::mod_add_id.eq(id)),
      'H' => query.filter(modlog_combined::mod_add_community_id.eq(id)),
      'I' => query.filter(modlog_combined::mod_ban_id.eq(id)),
      'J' => query.filter(modlog_combined::mod_ban_from_community_id.eq(id)),
      'K' => query.filter(modlog_combined::mod_feature_post_id.eq(id)),
      'L' => query.filter(modlog_combined::mod_hide_community_id.eq(id)),
      'M' => query.filter(modlog_combined::mod_lock_post_id.eq(id)),
      'N' => query.filter(modlog_combined::mod_remove_comment_id.eq(id)),
      'O' => query.filter(modlog_combined::mod_remove_community_id.eq(id)),
      'P' => query.filter(modlog_combined::mod_remove_post_id.eq(id)),
      'Q' => query.filter(modlog_combined::mod_transfer_community_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };

    let token = query.first(conn).await?;

    Ok(token)
  }
}
