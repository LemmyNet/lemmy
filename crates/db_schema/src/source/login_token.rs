use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::login_token;

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginToken {
  pub id: i32,
  pub token: String,
  pub user_id: LocalUserId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginTokenCreateForm {
  pub token: String,
  pub user_id: LocalUserId,
}
