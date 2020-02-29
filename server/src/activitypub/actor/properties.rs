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

//! Namespace for properties of standard Actor types
//!
//! To use these properties in your own types, you can flatten them into your struct with serde:
//!
//! ```rust
//! use activitypub::{Object, Actor, actor::properties::ApActorProperties};
//! use serde_derive::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! pub struct MyActor {
//!     #[serde(rename = "type")]
//!     pub kind: String,
//!
//!     /// Define a require property for the MyActor type
//!     pub my_property: String,
//!
//!     #[serde(flatten)]
//!     pub actor_props: ApActorProperties,
//! }
//!
//! impl Object for MyActor {}
//! impl Actor for MyActor {}
//! #
//! # fn main() {}
//! ```

use activitystreams_derive::Properties;
use serde_derive::{Deserialize, Serialize};

use crate::activitypub::endpoint::Endpoint;

/// Define activitypub properties for the Actor type as described by the Activity Pub vocabulary.
#[derive(Clone, Debug, Default, Deserialize, Properties, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApActorProperties {
  // TODO: IRI
  /// A reference to an [[ActivityStreams](https://www.w3.org/ns/activitystreams)]
  /// OrderedCollection comprised of all the messages received by the actor.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[activitystreams(concrete(String), functional)]
  pub inbox: serde_json::Value,

  // TODO: IRI
  /// An [ActivityStreams](https://www.w3.org/ns/activitystreams)] OrderedCollection comprised of
  /// all the messages produced by the actor.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[activitystreams(concrete(String), functional)]
  pub outbox: serde_json::Value,

  // TODO: IRI
  /// A link to an [[ActivityStreams](https://www.w3.org/ns/activitystreams)] collection of the
  /// actors that this actor is following.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[activitystreams(concrete(String), functional)]
  pub following: Option<serde_json::Value>,

  // TODO: IRI
  /// A link to an [[ActivityStreams](https://www.w3.org/ns/activitystreams)] collection of the
  /// actors that follow this actor.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[activitystreams(concrete(String), functional)]
  pub followers: Option<serde_json::Value>,

  // TODO: IRI
  /// A link to an [[ActivityStreams](https://www.w3.org/ns/activitystreams)] collection of
  /// objects this actor has liked.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[activitystreams(concrete(String), functional)]
  pub liked: Option<serde_json::Value>,

  // TODO: IRI
  /// A list of supplementary Collections which may be of interest.
  ///
  /// - Range: `anyUri`
  /// - Functional: false
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub streams: Option<serde_json::Value>,

  /// A short username which may be used to refer to the actor, with no uniqueness guarantees.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String), functional)]
  pub preferred_username: Option<serde_json::Value>,

  /// A json object which maps additional (typically server/domain-wide) endpoints which may be
  /// useful either for this actor or someone referencing this actor.
  ///
  /// This mapping may be nested inside the actor document as the value or may be a link to a
  /// JSON-LD document with these properties.
  ///
  /// - Range: `Endpoint`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(Endpoint), functional)]
  pub endpoints: Option<serde_json::Value>,
}
