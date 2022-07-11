// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{GetPersonMentions, GetPersonMentionsResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_views_actor::person_mention_view::PersonMentionQueryBuilder;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetPersonMentions {
  type Response = GetPersonMentionsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPersonMentionsResponse, LemmyError> {
    let data: &GetPersonMentions = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let sort = data.sort;
    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let person_id = local_user_view.person.id;
    let mentions = blocking(context.pool(), move |conn| {
      PersonMentionQueryBuilder::create(conn)
        .recipient_id(person_id)
        .my_person_id(person_id)
        .sort(sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(GetPersonMentionsResponse { mentions })
  }
}
