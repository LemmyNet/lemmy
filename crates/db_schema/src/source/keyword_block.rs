use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::local_user_keyword_block;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_keyword_block))]
#[cfg_attr(feature = "full", diesel(primary_key(local_user_id, keyword)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct LocalUserKeywordBlock {
  pub local_user_id: LocalUserId,
  pub keyword: String,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_keyword_block))]
pub struct LocalUserKeywordBlockForm {
  pub local_user_id: LocalUserId,
  pub keyword: String,
}
