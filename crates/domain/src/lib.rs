#[macro_use]
extern crate diesel_derive_newtype;

mod sealed {
  pub trait LemmyEntity {}
}

mod entities;
