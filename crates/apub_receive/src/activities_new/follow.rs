use url::Url;
use crate::inbox::new_inbox_routing::{ReceiveActivity, Activity, verify_domains_match};
use activitystreams::activity::kind::FollowType;
use activitystreams::activity::kind::AcceptType;
use crate::activities::receive::verify_activity_domains_valid;
use activitystreams::base::ExtendsExt;
use anyhow::Context;
use lemmy_apub::fetcher::community::get_or_fetch_and_upsert_community;
use lemmy_api_common::blocking;
use lemmy_db_schema::source::community::CommunityFollower;
use lemmy_websocket::LemmyContext;
use lemmy_utils::LemmyError;
use lemmy_utils::location_info;
use lemmy_db_queries::Followable;
use lemmy_apub::fetcher::person::get_or_fetch_and_upsert_person;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    // todo: implement newtypes PersonUrl, GroupUrl etc (with deref function)
    actor: Url,
    to: Url,
    object: Url,
    #[serde(rename = "type")]
    kind: FollowType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Follow {
    type Kind = FollowType;
    async fn receive(&self,activity: Activity<Self::Kind>,  context: &LemmyContext, request_counter: &mut i32) -> Result<(), LemmyError> {
        println!("receive follow");
        todo!()
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    // todo: implement newtypes PersonUrl, GroupUrl etc (with deref function)
    actor: Url,
    to: Url,
    object: Activity<Follow>,
    #[serde(rename = "type")]
    kind: AcceptType,
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Accept {
    type Kind = AcceptType;
    async fn receive(&self, activity: Activity<Self::Kind>, context: &LemmyContext, request_counter: &mut i32) -> Result<(), LemmyError> {
        verify_domains_match(&self.actor, &activity.id_unchecked())?;
        verify_domains_match(&self.object.inner.actor, &self.object.id_unchecked())?;

        let community =
            get_or_fetch_and_upsert_community(&self.actor, context, request_counter).await?;
        let person = get_or_fetch_and_upsert_person(&self.to, context, request_counter).await?;
        // This will throw an error if no follow was requested
        blocking(&context.pool(), move |conn| {
            CommunityFollower::follow_accepted(conn, community.id, person.id)
        })
            .await??;

        Ok(())
    }
}