#[macro_use]
extern crate lazy_static;

pub mod activity_queue;
pub mod data;
pub mod signatures;
pub mod traits;
pub mod values;
pub mod verify;
pub mod webfinger;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";
