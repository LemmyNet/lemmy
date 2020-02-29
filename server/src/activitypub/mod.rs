/*
 * This file is part of ActivityPub.
 *
 * Copyright © 2018 Riley Trautman
 *
 * ActivityPub is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ActivityPub is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ActivityPub.  If not, see <http://www.gnu.org/licenses/>.
 */

//! ActivityPub
//!
//! This crate defines the base set of types from the ActivityPub specification.
//!
//! ## Example Usage
//! ```rust
//! use activitypub::{context, object::Video};
//! use anyhow::Error;
//!
//! fn run() -> Result<(), Error> {
//!     let mut video = Video::default();
//!     video.object_props.set_context_object(context())?;
//!     video.ap_object_props.set_likes_string("https://my-instance.com/likes".to_owned());
//!
//!     let video_string = serde_json::to_string(&video)?;
//!
//!     let video: Video = serde_json::from_str(&video_string)?;
//!
//!     Ok(())
//! }
//! ```
pub mod activity;
pub mod actor;
pub mod collection;
mod endpoint;
pub mod link;
pub mod object;

pub use self::{
  activity::{Activity, IntransitiveActivity},
  actor::Actor,
  collection::{Collection, CollectionPage},
  endpoint::Endpoint,
  link::Link,
  object::Object,
};
pub use activitystreams_traits::{properties, Error, Result};
pub use activitystreams_types::{context, ContextObject, CustomLink, CustomObject};
