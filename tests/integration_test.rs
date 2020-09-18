extern crate lemmy_server;

use activitystreams::{
  activity::{kind::CreateType, ActorAndObject},
  base::{BaseExt, ExtendsExt},
  object::{Note, ObjectExt},
};
use actix::prelude::*;
use actix_web::{test::TestRequest, web};
use chrono::Utc;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use http_signature_normalization_actix::PrepareVerifyError;
use lemmy_db::{
  user::{User_, *},
  Crud,
  ListingType,
  SortType,
};
use lemmy_rate_limit::{rate_limiter::RateLimiter, RateLimit};
use lemmy_server::{
  apub::{
    activity_queue::create_activity_queue,
    inbox::shared_inbox::{shared_inbox, ValidTypes},
  },
  websocket::chat_server::ChatServer,
  LemmyContext,
};
use lemmy_utils::{apub::generate_actor_keypair, settings::Settings};
use reqwest::Client;
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

fn create_user(conn: &PgConnection) -> User_ {
  let user_keypair = generate_actor_keypair().unwrap();
  let new_user = UserForm {
    name: "integration_user_1".into(),
    preferred_username: None,
    password_encrypted: "nope".into(),
    email: None,
    matrix_user_id: None,
    avatar: None,
    banner: None,
    admin: false,
    banned: false,
    updated: None,
    show_nsfw: false,
    theme: "darkly".into(),
    default_sort_type: SortType::Hot as i16,
    default_listing_type: ListingType::Subscribed as i16,
    lang: "browser".into(),
    show_avatars: true,
    send_notifications_to_email: false,
    actor_id: Some("http://localhost:8536/u/integration_user_1".to_string()),
    bio: None,
    local: true,
    private_key: Some(user_keypair.private_key),
    public_key: Some(user_keypair.public_key),
    last_refreshed_at: None,
  };

  User_::create(&conn, &new_user).unwrap()
}

fn create_activity(user_id: String) -> web::Json<ActorAndObject<ValidTypes>> {
  let mut activity =
    ActorAndObject::<CreateType>::new(user_id, Note::new().into_any_base().unwrap());
  activity
    .set_id(Url::parse("http://localhost:8536/create/1").unwrap())
    .set_many_ccs(vec![Url::parse("http://localhost:8536/c/main").unwrap()]);
  let activity = serde_json::to_value(&activity).unwrap();
  let activity: ActorAndObject<ValidTypes> = serde_json::from_value(activity).unwrap();
  web::Json(activity)
}

#[actix_rt::test]
async fn test_expired_signature() {
  let time1 = Utc::now().timestamp();
  let time2 = Utc::now().timestamp();
  let signature = format!(
    r#"keyId="my-key-id",algorithm="hs2019",created="{}",expires="{}",headers="(request-target) (created) (expires) date content-type",signature="blah blah blah""#,
    time1, time2
  );
  let request = TestRequest::post()
    .uri("http://localhost:8536/inbox")
    .header("Signature", signature)
    .to_http_request();
  let context = create_context();
  let user = create_user(&context.pool().get().unwrap());
  let activity = create_activity(user.actor_id);
  let response = shared_inbox(request, activity, web::Data::new(context)).await;
  assert_eq!(
    format!("{}", response.err().unwrap()),
    format!("{}", PrepareVerifyError::Expired)
  );
}
