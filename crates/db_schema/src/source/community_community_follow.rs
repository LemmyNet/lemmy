use crate::newtypes::CommunityId;
use lemmy_db_schema_file::schema::community_community_follow;

#[derive(Clone, Debug, PartialEq, Queryable, Selectable)]
#[diesel(belongs_to(crate::source::community::Community))]
#[ diesel(table_name = community_community_follow)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommunityCommunityFollow {
  pub target_id: CommunityId,
  pub community_id: CommunityId,
}
