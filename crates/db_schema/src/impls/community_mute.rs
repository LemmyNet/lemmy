use crate::{
  schema::community_mute::dsl::{community_id, community_mute, person_id},
  source::community_mute::{CommunityMute, CommunityMuteForm},
  traits::Muteable,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Muteable for CommunityMute {
  type Form = CommunityMuteForm;
  async fn mute(pool: &DbPool, community_mute_form: &Self::Form) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_mute)
      .values(community_mute_form)
      .on_conflict((person_id, community_id))
      .do_update()
      .set(community_mute_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unmute(pool: &DbPool, community_mute_form: &Self::Form) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_mute
        .filter(person_id.eq(community_mute_form.person_id))
        .filter(community_id.eq(community_mute_form.community_id)),
    )
    .execute(conn)
    .await
  }
}
