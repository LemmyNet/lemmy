use crate::api::community::{
  GetCommunity, GetCommunityResponse, ListCommunities, ListCommunitiesResponse,
};
use crate::api::UserOperation;
use crate::api::{Oper, Perform};
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Serialize;

type DbParam = web::Data<Pool<ConnectionManager<PgConnection>>>;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    // TODO: need to repeat this for every endpoint
    .route(
      "/api/v1/list_communities",
      web::get().to(
        route::<ListCommunities, ListCommunitiesResponse>(UserOperation::ListCommunities)
      ),
    )
    .route(
      "/api/v1/get_community",
      web::get().to(route::<GetCommunity, GetCommunityResponse>(
        UserOperation::GetCommunity,
      )),
    );
}

fn perform<Request, Response>(
  op: UserOperation,
  data: Request,
  db: DbParam,
) -> Result<HttpResponse, Error>
where
  Response: Serialize,
  Oper<Request>: Perform<Response>,
{
  let conn = match db.get() {
    Ok(c) => c,
    Err(e) => return Err(format_err!("{}", e)),
  };
  let oper: Oper<Request> = Oper::new(op, data);
  let response = oper.perform(&conn);
  Ok(HttpResponse::Ok().json(response?))
}

fn route<Data, Response>(
  op: UserOperation,
) -> Box<(dyn Fn(web::Query<Data>, DbParam) -> Result<HttpResponse, Error> + 'static)>
where
  Data: Serialize,
  Response: Serialize,
  Oper<Data>: Perform<Response>,
{
  // TODO: want an implementation like this, where useroperation is passed without explicitly passing the other params
  //       maybe with a higher order functions? (but that would probably have worse performance)
  Box::new(|data, db| perform::<Data, Response>(op, data.0, db))
}
