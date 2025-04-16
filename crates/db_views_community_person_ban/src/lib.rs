#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::{community::Community, person::Person};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A community person ban.
pub struct CommunityPersonBanView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}
