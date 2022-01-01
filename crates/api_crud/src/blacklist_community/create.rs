use crate::PerformCrud;
use actix_web::web::Data;

use lemmy_api_common::{
    blacklist_community::*,
  };
use lemmy_utils::{
  ConnectionId,
  LemmyError,
};

use lemmy_websocket::{
  LemmyContext,
};

use lemmy_api_common::{
  blocking,
  blacklist_community::{BlackListCommunity}, 
  get_local_user_view_from_jwt,
  is_admin,
};

use lemmy_db_schema::{
  source::{
    blacklist_community::{BlackList, BlackListForm},
  },
  traits::{Crud},
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for BlackListCommunity {
    type Response = BlackListCommunityResponse;
    async fn perform(
      &self,
      context: &Data<LemmyContext>,
      _: Option<ConnectionId>,
    ) -> Result<BlackListCommunityResponse, LemmyError> {
      let data: &BlackListCommunity = self;
      let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
      is_admin(&local_user_view)?;
      
      let blacklist_form =  BlackListForm {
         community_id: data.community_id,
        reason: data.reason.clone(),
        creator_id: local_user_view.person.id,
        ..BlackListForm::default()
      };

      let inserted_blacklist = blocking(context.pool(), move |conn| {
        BlackList::create(conn, &blacklist_form)
      })
      .await? 
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldn't create blacklist community"))?;
      let response: BlackListCommunityResponse = BlackListCommunityResponse{blacklist_id:inserted_blacklist.id.clone()};
      
      return Ok(response);
    }
    
}


#[async_trait::async_trait(?Send)]
impl PerformCrud for  DeleteBlackListCommunity {
  type Response = bool;
    async fn perform(
      &self,
      context: &Data<LemmyContext>,
      _: Option<ConnectionId>,
    ) -> Result<bool, LemmyError> {
      let data: &DeleteBlackListCommunity = self;
      let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
      is_admin(&local_user_view)?;
      
      let community_id = data.community_id;

      blocking(context.pool(), move |conn| {
        BlackList::delete(conn, community_id)
      })
      .await? 
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldn't delete blacklist community"))?;
      
      return Ok(true);
    }
}