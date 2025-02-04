#[cfg(test)]
extern crate serial_test;

#[cfg(feature = "full")]
pub mod combined;
#[cfg(feature = "full")]
pub mod comment;
#[cfg(feature = "full")]
pub mod community;
#[cfg(feature = "full")]
pub mod local_user;
#[cfg(feature = "full")]
pub mod person;
#[cfg(feature = "full")]
pub mod post;
#[cfg(feature = "full")]
pub mod private_message;
#[cfg(feature = "full")]
pub mod registration_applications;
#[cfg(feature = "full")]
pub mod reports;
#[cfg(feature = "full")]
pub mod site;
pub mod structs;
