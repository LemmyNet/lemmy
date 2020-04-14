use crate::apub::activities::accept_follow;
use crate::apub::fetcher::fetch_remote_user;
use crate::db::community::{Community, CommunityFollower, CommunityFollowerForm};
use crate::db::post::{Post, PostForm};
use crate::db::Crud;
use crate::db::Followable;
use activitystreams::activity::{Accept, Create, Follow, Update};
use activitystreams::object::Page;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use url::Url;

// TODO: need a proper actor that has this inbox

pub async fn inbox(
  input: web::Json<AcceptedObjects>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  // TODO: make sure that things are received in the correct inbox
  //      (by using seperate handler functions and checking the user/community name in the path)
  let input = input.into_inner();
  let conn = &db.get().unwrap();
  match input {
    AcceptedObjects::Create(c) => handle_create(&c, conn),
    AcceptedObjects::Update(u) => handle_update(&u, conn),
    AcceptedObjects::Follow(f) => handle_follow(&f, conn),
    AcceptedObjects::Accept(a) => handle_accept(&a, conn),
  }
}

fn handle_create(create: &Create, conn: &PgConnection) -> Result<HttpResponse, Error> {
  let page = create
    .create_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  Post::create(conn, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
}

fn handle_update(update: &Update, conn: &PgConnection) -> Result<HttpResponse, Error> {
  let page = update
    .update_props
    .get_object_base_box()
    .to_owned()
    .unwrap()
    .to_owned()
    .to_concrete::<Page>()?;
  let post = PostForm::from_page(&page, conn)?;
  let id = Post::read_from_apub_id(conn, &post.ap_id)?.id;
  Post::update(conn, id, &post)?;
  // TODO: send the new post out via websocket
  Ok(HttpResponse::Ok().finish())
}

fn handle_follow(follow: &Follow, conn: &PgConnection) -> Result<HttpResponse, Error> {
  println!("received follow: {:?}", &follow);

  // TODO: make sure this is a local community
  let community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();
  let community = Community::read_from_actor_id(conn, &community_uri)?;
  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let user = fetch_remote_user(&Url::parse(&user_uri)?, conn)?;
  // TODO: insert ID of the user into follows of the community
  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };
  CommunityFollower::follow(&conn, &community_follower_form)?;
  accept_follow(&follow)?;
  Ok(HttpResponse::Ok().finish())
}

fn handle_accept(accept: &Accept, _conn: &PgConnection) -> Result<HttpResponse, Error> {
  println!("received accept: {:?}", &accept);
  // TODO: at this point, indicate to the user that they are following the community
  Ok(HttpResponse::Ok().finish())
}

#[serde(untagged)]
#[derive(serde::Deserialize)]
pub enum AcceptedObjects {
  Create(Create),
  Update(Update),
  Follow(Follow),
  Accept(Accept),
}
