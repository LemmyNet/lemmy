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

//! Object traits and types

pub use activitystreams_traits::Object;
pub use activitystreams_types::object::{kind, ObjectExt};
use serde_derive::{Deserialize, Serialize};

pub mod properties;

use self::{kind::*, properties::*};

/// The ActivityPub Object Extension Trait
///
/// This trait provides generic access to an activitypub object's properties
pub trait ApObjectExt: Object {
  fn props(&self) -> &ApObjectProperties;
  fn props_mut(&mut self) -> &mut ApObjectProperties;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
  #[serde(rename = "type")]
  kind: ArticleType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Article {}
impl ObjectExt for Article {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Article {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
  #[serde(rename = "type")]
  kind: AudioType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Audio {}
impl ObjectExt for Audio {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Audio {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
  #[serde(rename = "type")]
  kind: DocumentType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Document {}
impl ObjectExt for Document {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Document {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
  #[serde(rename = "type")]
  kind: EventType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Event {}
impl ObjectExt for Event {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Event {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
  #[serde(rename = "type")]
  kind: ImageType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Image {}
impl ObjectExt for Image {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Image {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "type")]
  kind: NoteType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Note {}
impl ObjectExt for Note {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Note {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  #[serde(rename = "type")]
  kind: PageType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Page {}
impl ObjectExt for Page {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Page {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
  #[serde(rename = "type")]
  kind: PlaceType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub place_props: PlaceProperties,
}

impl Object for Place {}
impl ObjectExt for Place {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Place {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
  #[serde(rename = "type")]
  kind: ProfileType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub profile_props: ProfileProperties,
}

impl Object for Profile {}
impl ObjectExt for Profile {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Profile {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
  #[serde(rename = "type")]
  kind: RelationshipType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub relationship_props: RelationshipProperties,
}

impl Object for Relationship {}
impl ObjectExt for Relationship {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Relationship {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
  #[serde(rename = "type")]
  kind: TombstoneType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub tombstone_props: TombstoneProperties,
}

impl Object for Tombstone {}
impl ObjectExt for Tombstone {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Tombstone {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
  #[serde(rename = "type")]
  kind: VideoType,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,
}

impl Object for Video {}
impl ObjectExt for Video {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Video {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
