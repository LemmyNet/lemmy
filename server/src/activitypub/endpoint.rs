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

//! Endpoint traits and types

use activitystreams_derive::Properties;
use serde_derive::{Deserialize, Serialize};

/// A json object which maps additional (typically server/domain-wide) endpoints which may be
/// useful either for this actor or someone referencing this actor.
///
/// This mapping may be nested inside the actor document as the value or may be a link to a JSON-LD
/// document with these properties.
#[derive(Clone, Debug, Default, Deserialize, Properties, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
  // TODO: IRI
  /// Endpoint URI so this actor's clients may access remote ActivityStreams objects which
  /// require authentication to access.
  ///
  /// To use this endpoint, the client posts an x-www-form-urlencoded id parameter with the value
  /// being the id of the requested ActivityStreams object.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub proxy_url: Option<serde_json::Value>,

  // TODO: IRI
  /// If OAuth 2.0 bearer tokens [[RFC6749](https://tools.ietf.org/html/rfc6749)]
  /// [[RFC6750](https://tools.ietf.org/html/rfc6750)] are being used for authenticating client
  /// to server interactions, this endpoint specifies a URI at which a browser-authenticated user
  /// may obtain a new authorization grant.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub oauth_authorization_endpoint: Option<serde_json::Value>,

  // TODO: IRI
  /// If OAuth 2.0 bearer tokens [[RFC6749](https://tools.ietf.org/html/rfc6749)]
  /// [[RFC6750](https://tools.ietf.org/html/rfc6750)] are being used for authenticating client
  /// to server interactions, this endpoint specifies a URI at which a client may acquire an
  /// access token.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub oauth_token_endpoint: Option<serde_json::Value>,

  // TODO: IRI
  /// If Linked Data Signatures and HTTP Signatures are being used for authentication and
  /// authorization, this endpoint specifies a URI at which browser-authenticated users may
  /// authorize a client's public key for client to server interactions.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub provide_client_key: Option<serde_json::Value>,

  // TODO: IRI
  /// If Linked Data Signatures and HTTP Signatures are being used for authentication and
  /// authorization, this endpoint specifies a URI at which a client key may be signed by the
  /// actor's key for a time window to act on behalf of the actor in interacting with foreign
  /// servers.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub sign_client_key: Option<serde_json::Value>,

  // TODO: IRI
  /// An optional endpoint used for wide delivery of publicly addressed activities and activities
  /// sent to followers.
  ///
  /// `shared_inbox`endpoints SHOULD also be publicly readable `OrderedCollection` objects
  /// containing objects addressed to the Public special collection. Reading from the
  /// `shared_inbox` endpoint MUST NOT present objects which are not addressed to the Public
  /// endpoint.
  ///
  /// - Range: `anyUri`
  /// - Functional: true
  #[serde(skip_serializing_if = "Option::is_none")]
  #[activitystreams(concrete(String))]
  pub shared_inbox: Option<serde_json::Value>,
}
