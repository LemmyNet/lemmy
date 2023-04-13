use actix::{Message, Recipient};

pub mod captcha;
pub mod connect;
pub mod join_rooms;
pub mod messages;
pub mod online_users;

/// A string message sent to a websocket session
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// TODO move this?
pub struct SessionInfo {
  pub addr: Recipient<WsMessage>,
  // pub ip: IpAddr
}
