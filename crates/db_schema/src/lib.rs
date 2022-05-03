#[cfg(feature = "full")]
#[macro_use]
extern crate strum_macros;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_newtype;
// this is used in tests
#[cfg(feature = "full")]
#[allow(unused_imports)]
#[macro_use]
extern crate diesel_migrations;

pub mod aggregates;
#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
#[cfg(feature = "full")]
pub mod schema;
pub mod source;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;
