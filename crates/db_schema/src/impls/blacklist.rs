use crate::{
  source::blacklist_community::{BlackList, BlackListForm},
  traits::Crud,
};
use diesel::{dsl::*, result::Error, *};

impl Crud for BlackList {
  type Form = BlackListForm;
  type IdType = i32;

  fn create(conn: &PgConnection, new_blacklist: &BlackListForm) -> Result<Self, Error> {
    use crate::schema::blacklist_community::dsl::*;
    insert_into(blacklist_community)
      .values(new_blacklist)
      .get_result::<Self>(conn)
  }
  fn read(conn: &PgConnection, blacklist_id: i32) -> Result<Self, Error> {
    use crate::schema::blacklist_community::dsl::*;
    blacklist_community.find(blacklist_id).first::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    blacklist_id: i32,
    updateblacklistform_community: &BlackListForm,
  ) -> Result<Self, Error> {
    use crate::schema::blacklist_community::dsl::*;
    diesel::update(blacklist_community.find(blacklist_id))
      .set(updateblacklistform_community)
      .get_result::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_key: i32) -> Result<usize, Error> {
    use crate::schema::blacklist_community::dsl::*;
    diesel::delete(blacklist_community.filter(community_id.eq(community_key))).execute(conn)
  }
}
