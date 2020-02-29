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

//! Actor traits and types

use activitystreams_derive::Properties;
pub use activitystreams_traits::Actor;
pub use activitystreams_types::actor::kind;
use serde_derive::{Deserialize, Serialize};

pub mod properties;

use self::{kind::*, properties::*};
use activitypub::object::{
  properties::{ApObjectProperties, ObjectProperties},
  ApObjectExt, Object, ObjectExt,
};

/// The ActivityPub Actor Extension Trait
///
/// This trait provides generic access to an activitypub actor's properties
pub trait ApActorExt: Actor {
  fn props(&self) -> &ApActorProperties;
  fn props_mut(&mut self) -> &mut ApActorProperties;
}

/// Describes a software application.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Properties)]
#[serde(rename_all = "camelCase")]
pub struct Application {
  #[serde(rename = "type")]
  kind: ApplicationType,

  /// Adds all valid object properties to this struct
  #[serde(flatten)]
  pub object_props: ObjectProperties,

  /// Adds all valid activitypub object properties to this struct
  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  /// Adds all valid activitypub actor properties to this struct
  #[serde(flatten)]
  pub ap_actor_props: ApActorProperties,
}

impl Object for Application {}
impl ObjectExt for Application {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Application {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Actor for Application {}
impl ApActorExt for Application {
  fn props(&self) -> &ApActorProperties {
    &self.ap_actor_props
  }

  fn props_mut(&mut self) -> &mut ApActorProperties {
    &mut self.ap_actor_props
  }
}

/// Represents a formal or informal collective of Actors.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Properties)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  #[serde(rename = "type")]
  kind: GroupType,

  /// Adds all valid object properties to this struct
  #[serde(flatten)]
  pub object_props: ObjectProperties,

  /// Adds all valid activitypub object properties to this struct
  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  /// Adds all valid activitypub actor properties to this struct
  #[serde(flatten)]
  pub ap_actor_props: ApActorProperties,
}

impl Object for Group {}
impl ObjectExt for Group {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Group {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Actor for Group {}
impl ApActorExt for Group {
  fn props(&self) -> &ApActorProperties {
    &self.ap_actor_props
  }

  fn props_mut(&mut self) -> &mut ApActorProperties {
    &mut self.ap_actor_props
  }
}

/// Represents an organization.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Properties)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
  #[serde(rename = "type")]
  kind: OrganizationType,

  /// Adds all valid object properties to this struct
  #[serde(flatten)]
  pub object_props: ObjectProperties,

  /// Adds all valid activitypub object properties to this struct
  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  /// Adds all valid activitypub actor properties to this struct
  #[serde(flatten)]
  pub ap_actor_props: ApActorProperties,
}

impl Object for Organization {}
impl ObjectExt for Organization {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Organization {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Actor for Organization {}
impl ApActorExt for Organization {
  fn props(&self) -> &ApActorProperties {
    &self.ap_actor_props
  }

  fn props_mut(&mut self) -> &mut ApActorProperties {
    &mut self.ap_actor_props
  }
}

/// Represents an individual person.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Properties)]
#[serde(rename_all = "camelCase")]
pub struct Person {
  #[serde(rename = "type")]
  kind: PersonType,

  /// Adds all valid object properties to this struct
  #[serde(flatten)]
  pub object_props: ObjectProperties,

  /// Adds all valid activitypub object properties to this struct
  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  /// Adds all valid activitypub actor properties to this struct
  #[serde(flatten)]
  pub ap_actor_props: ApActorProperties,
}

impl Object for Person {}
impl ObjectExt for Person {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Person {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Actor for Person {}
impl ApActorExt for Person {
  fn props(&self) -> &ApActorProperties {
    &self.ap_actor_props
  }

  fn props_mut(&mut self) -> &mut ApActorProperties {
    &mut self.ap_actor_props
  }
}

/// Represents a service of any kind.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Properties)]
#[serde(rename_all = "camelCase")]
pub struct Service {
  #[serde(rename = "type")]
  kind: ServiceType,

  /// Adds all valid object properties to this struct
  #[serde(flatten)]
  pub object_props: ObjectProperties,

  /// Adds all valid activitypub object properties to this struct
  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  /// Adds all valid activitypub actor properties to this struct
  #[serde(flatten)]
  pub ap_actor_props: ApActorProperties,
}

impl Object for Service {}
impl ObjectExt for Service {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Service {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Actor for Service {}
impl ApActorExt for Service {
  fn props(&self) -> &ApActorProperties {
    &self.ap_actor_props
  }

  fn props_mut(&mut self) -> &mut ApActorProperties {
    &mut self.ap_actor_props
  }
}
