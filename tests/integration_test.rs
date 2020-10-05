extern crate lemmy_server;

use activitystreams::{
  activity::{
    kind::{CreateType, FollowType},
    ActorAndObject,
  },
  base::{BaseExt, ExtendsExt},
  object::{Note, ObjectExt},
};
use actix::prelude::*;
use actix_web::{test::TestRequest, web, web::Path, HttpRequest};
use chrono::Utc;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use http_signature_normalization_actix::PrepareVerifyError;
use lemmy_db::{
  community::{Community, CommunityForm},
  user::{User_, *},
  Crud,
  ListingType,
  SortType,
};
use lemmy_rate_limit::{rate_limiter::RateLimiter, RateLimit};
use lemmy_server::{
  apub::{
    activity_queue::create_activity_queue,
    inbox::{
      community_inbox,
      community_inbox::community_inbox,
      shared_inbox,
      shared_inbox::shared_inbox,
      user_inbox,
      user_inbox::user_inbox,
    },
  },
  websocket::chat_server::ChatServer,
  LemmyContext,
};
use lemmy_utils::{apub::generate_actor_keypair, settings::Settings};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

fn create_context() -> LemmyContext {
  let settings = Settings::get();
  let db_url = settings.get_database_url();
  let manager = ConnectionManager::<PgConnection>::new(&db_url);
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .build(manager)
    .unwrap();
  let rate_limiter = RateLimit {
    rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
  };
  let activity_queue = create_activity_queue();
  let chat_server = ChatServer::startup(
    pool.clone(),
    rate_limiter.clone(),
    Client::default(),
    activity_queue.clone(),
  )
  .start();
  LemmyContext::new(
    pool,
    chat_server,
    Client::default(),
    create_activity_queue(),
  )
}

fn create_user(conn: &PgConnection, name: &str) -> User_ {
  let user_keypair = generate_actor_keypair().unwrap();
  let new_user = UserForm {
    name: name.into(),
    preferred_username: None,
    password_encrypted: "nope".into(),
    email: None,
    matrix_user_id: None,
    avatar: None,
    banner: None,
    admin: false,
    banned: false,
    updated: None,
    published: None,
    show_nsfw: false,
    theme: "browser".into(),
    default_sort_type: SortType::Hot as i16,
    default_listing_type: ListingType::Subscribed as i16,
    lang: "browser".into(),
    show_avatars: true,
    send_notifications_to_email: false,
    actor_id: Some(format!("http://localhost:8536/u/{}", name).to_string()),
    bio: None,
    local: true,
    private_key: Some(user_keypair.private_key),
    public_key: Some(user_keypair.public_key),
    last_refreshed_at: None,
  };

  User_::create(&conn, &new_user).unwrap()
}

fn create_community(conn: &PgConnection, creator_id: i32) -> Community {
  let new_community = CommunityForm {
    name: "test_community".into(),
    creator_id,
    title: "test_community".to_owned(),
    description: None,
    category_id: 1,
    nsfw: false,
    removed: None,
    deleted: None,
    updated: None,
    actor_id: None,
    local: true,
    private_key: None,
    public_key: None,
    last_refreshed_at: None,
    published: None,
    icon: None,
    banner: None,
  };
  Community::create(&conn, &new_community).unwrap()
}
fn create_activity<'a, Activity, Return>(user_id: String) -> web::Json<Return>
where
  for<'de> Return: Deserialize<'de> + 'a,
  Activity: std::default::Default + Serialize,
{
  let mut activity = ActorAndObject::<Activity>::new(user_id, Note::new().into_any_base().unwrap());
  activity
    .set_id(Url::parse("http://localhost:8536/create/1").unwrap())
    .set_many_ccs(vec![Url::parse("http://localhost:8536/c/main").unwrap()]);
  let activity = serde_json::to_value(&activity).unwrap();
  let activity: Return = serde_json::from_value(activity).unwrap();
  web::Json(activity)
}

fn create_http_request() -> HttpRequest {
  let time1 = Utc::now().timestamp();
  let time2 = Utc::now().timestamp();
  let signature = format!(
    r#"keyId="my-key-id",algorithm="hs2019",created="{}",expires="{}",headers="(request-target) (created) (expires) date content-type",signature="blah blah blah""#,
    time1, time2
  );
  TestRequest::post()
    .uri("http://localhost:8536/")
    .header("Signature", signature)
    .to_http_request()
}

#[actix_rt::test]
async fn test_shared_inbox_expired_signature() {
  let request = create_http_request();
  let context = create_context();
  let connection = &context.pool().get().unwrap();
  let user = create_user(connection, "shared_inbox_rvgfd");
  let activity =
    create_activity::<CreateType, ActorAndObject<shared_inbox::ValidTypes>>(user.actor_id);
  let response = shared_inbox(request, activity, web::Data::new(context)).await;
  assert_eq!(
    format!("{}", response.err().unwrap()),
    format!("{}", PrepareVerifyError::Expired)
  );
  User_::delete(connection, user.id).unwrap();
}

#[actix_rt::test]
async fn test_user_inbox_expired_signature() {
  let request = create_http_request();
  let context = create_context();
  let connection = &context.pool().get().unwrap();
  let user = create_user(connection, "user_inbox_cgsax");
  let activity =
    create_activity::<CreateType, ActorAndObject<user_inbox::ValidTypes>>(user.actor_id);
  let path = Path::<String> {
    0: "username".to_string(),
  };
  let response = user_inbox(request, activity, path, web::Data::new(context)).await;
  assert_eq!(
    format!("{}", response.err().unwrap()),
    format!("{}", PrepareVerifyError::Expired)
  );
  User_::delete(connection, user.id).unwrap();
}

#[actix_rt::test]
async fn test_community_inbox_expired_signature() {
  let context = create_context();
  let connection = &context.pool().get().unwrap();
  let user = create_user(connection, "community_inbox_hrxa");
  let community = create_community(connection, user.id);
  let request = create_http_request();
  let activity =
    create_activity::<FollowType, ActorAndObject<community_inbox::ValidTypes>>(user.actor_id);
  let path = Path::<String> { 0: community.name };
  let response = community_inbox(request, activity, path, web::Data::new(context)).await;
  assert_eq!(
    format!("{}", response.err().unwrap()),
    format!("{}", PrepareVerifyError::Expired)
  );
  User_::delete(connection, user.id).unwrap();
  Community::delete(connection, community.id).unwrap();
}
