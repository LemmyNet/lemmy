use crate::{
  activities::{generate_activity_id, verify_activity, verify_person_in_community},
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
  send_lemmy_activity,
  PostOrComment,
};
use activitystreams::{
  activity::kind::FlagType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::{blocking, comment::CommentReportResponse, post::PostReportResponse};
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  source::{
    comment_report::{CommentReport, CommentReportForm},
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
};
use lemmy_db_views::{comment_report_view::CommentReportView, post_report_view::PostReportView};
use lemmy_utils::LemmyError;
use lemmy_websocket::{messages::SendModRoomMessage, LemmyContext, UserOperation};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  actor: ObjectId<ApubPerson>,
  to: [ObjectId<ApubCommunity>; 1],
  object: ObjectId<PostOrComment>,
  summary: String,
  #[serde(rename = "type")]
  kind: FlagType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl Report {
  pub async fn send(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community_id: ObjectId<ApubCommunity>,
    reason: String,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = community_id.dereference_local(context).await?;
    let kind = FlagType::Flag;
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let report = Report {
      actor: ObjectId::new(actor.actor_id()),
      to: [ObjectId::new(community.actor_id())],
      object: object_id,
      summary: reason,
      kind,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    send_lemmy_activity(
      context,
      &report,
      &id,
      actor,
      vec![community.shared_inbox_or_inbox_url()],
      false,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Report {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self, &context.settings())?;
    verify_person_in_community(&self.actor, &self.to[0], context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context, request_counter).await?;
    match self.object.dereference(context, request_counter).await? {
      PostOrComment::Post(post) => {
        let report_form = PostReportForm {
          creator_id: actor.id,
          post_id: post.id,
          original_post_name: post.name.clone(),
          original_post_url: post.url.clone(),
          reason: self.summary,
          original_post_body: post.body.clone(),
        };

        let report = blocking(context.pool(), move |conn| {
          PostReport::report(conn, &report_form)
        })
        .await??;

        let post_report_view = blocking(context.pool(), move |conn| {
          PostReportView::read(conn, report.id, actor.id)
        })
        .await??;

        context.chat_server().do_send(SendModRoomMessage {
          op: UserOperation::CreateCommentReport,
          response: PostReportResponse { post_report_view },
          community_id: post.community_id,
          websocket_id: None,
        });
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: actor.id,
          comment_id: comment.id,
          original_comment_text: comment.content.clone(),
          reason: self.summary,
        };

        let report = blocking(context.pool(), move |conn| {
          CommentReport::report(conn, &report_form)
        })
        .await??;

        let comment_report_view = blocking(context.pool(), move |conn| {
          CommentReportView::read(conn, report.id, actor.id)
        })
        .await??;
        let community_id = comment_report_view.community.id;

        context.chat_server().do_send(SendModRoomMessage {
          op: UserOperation::CreateCommentReport,
          response: CommentReportResponse {
            comment_report_view,
          },
          community_id,
          websocket_id: None,
        });
      }
    };
    Ok(())
  }
}
