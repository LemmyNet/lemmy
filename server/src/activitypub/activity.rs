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

//! Activity traits and types

pub use activitystreams_traits::{Activity, IntransitiveActivity};
pub use activitystreams_types::activity::{kind, properties, ActivityExt};
use serde_derive::{Deserialize, Serialize};

use self::{kind::*, properties::*};
use activitypub::object::{
  properties::{ApObjectProperties, ObjectProperties},
  ApObjectExt, Object, ObjectExt,
};

/// Indicates that the actor accepts the object.
///
/// The target property can be used in certain circumstances to indicate the context into which the
/// object has been accepted.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
  #[serde(rename = "type")]
  kind: AcceptType,

  #[serde(flatten)]
  pub accept_props: AcceptProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Accept {}
impl ObjectExt for Accept {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Accept {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Accept {}
impl ActivityExt for Accept {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has added the object to the target.
///
/// If the target property is not explicitly specified, the target would need to be determined
/// implicitly by context. The origin can be used to identify the context from which the object
/// originated.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Add {
  #[serde(rename = "type")]
  kind: AddType,

  #[serde(flatten)]
  pub add_props: AddProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Add {}
impl ObjectExt for Add {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Add {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Add {}
impl ActivityExt for Add {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has moved object from origin to target.
///
/// If the origin or target are not specified, either can be determined by context.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AMove {
  #[serde(rename = "type")]
  kind: MoveType,

  #[serde(flatten)]
  pub move_props: MoveProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for AMove {}
impl ObjectExt for AMove {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for AMove {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for AMove {}
impl ActivityExt for AMove {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is calling the target's attention the object.
///
/// The origin typically has no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Announce {
  #[serde(rename = "type")]
  kind: AnnounceType,

  #[serde(flatten)]
  pub announce_props: AnnounceProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Announce {}
impl ObjectExt for Announce {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Announce {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Announce {}
impl ActivityExt for Announce {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// An IntransitiveActivity that indicates that the actor has arrived at the location.
///
/// The origin can be used to identify the context from which the actor originated. The target
/// typically has no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Arrive {
  #[serde(rename = "type")]
  kind: ArriveType,

  #[serde(flatten)]
  pub arrive_props: ArriveProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Arrive {}
impl ObjectExt for Arrive {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Arrive {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Arrive {}
impl ActivityExt for Arrive {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}
impl IntransitiveActivity for Arrive {}

/// Indicates that the actor is blocking the object.
///
/// Blocking is a stronger form of Ignore. The typical use is to support social systems that allow
/// one user to block activities or content of other users. The target and origin typically have no
/// defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
  #[serde(rename = "type")]
  kind: BlockType,

  #[serde(flatten)]
  pub block_props: BlockProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Block {}
impl ObjectExt for Block {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Block {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Block {}
impl ActivityExt for Block {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has created the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Create {
  #[serde(rename = "type")]
  kind: CreateType,

  #[serde(flatten)]
  pub create_props: CreateProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Create {}
impl ObjectExt for Create {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Create {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Create {}
impl ActivityExt for Create {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has deleted the object.
///
/// If specified, the origin indicates the context from which the object was deleted.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  #[serde(rename = "type")]
  kind: DeleteType,

  #[serde(flatten)]
  pub delete_props: DeleteProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Delete {}
impl ObjectExt for Delete {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Delete {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Delete {}
impl ActivityExt for Delete {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor dislikes the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dislike {
  #[serde(rename = "type")]
  kind: DislikeType,

  #[serde(flatten)]
  pub dislike_props: DislikeProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Dislike {}
impl ObjectExt for Dislike {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Dislike {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Dislike {}
impl ActivityExt for Dislike {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is "flagging" the object.
///
/// Flagging is defined in the sense common to many social platforms as reporting content as being
/// inappropriate for any number of reasons.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
  #[serde(rename = "type")]
  kind: FlagType,

  #[serde(flatten)]
  pub flag_props: FlagProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Flag {}
impl ObjectExt for Flag {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Flag {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Flag {}
impl ActivityExt for Flag {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is "following" the object.
///
/// Following is defined in the sense typically used within Social systems in which the actor is
/// interested in any activity performed by or on the object. The target and origin typically have
/// no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
  #[serde(rename = "type")]
  kind: FollowType,

  #[serde(flatten)]
  pub follow_props: FollowProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Follow {}
impl ObjectExt for Follow {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Follow {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Follow {}
impl ActivityExt for Follow {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is ignoring the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ignore {
  #[serde(rename = "type")]
  kind: IgnoreType,

  #[serde(flatten)]
  pub ignore_props: IgnoreProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Ignore {}
impl ObjectExt for Ignore {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Ignore {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Ignore {}
impl ActivityExt for Ignore {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// A specialization of Offer in which the actor is extending an invitation for the object to the
/// target.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
  #[serde(rename = "type")]
  kind: InviteType,

  #[serde(flatten)]
  pub invite_props: InviteProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Invite {}
impl ObjectExt for Invite {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Invite {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Invite {}
impl ActivityExt for Invite {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has joined the object.
///
/// The target and origin typically have no defined meaning
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Join {
  #[serde(rename = "type")]
  kind: JoinType,

  #[serde(flatten)]
  pub join_props: JoinProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Join {}
impl ObjectExt for Join {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Join {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Join {}
impl ActivityExt for Join {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has left the object.
///
/// The target and origin typically have no meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leave {
  #[serde(rename = "type")]
  kind: LeaveType,

  #[serde(flatten)]
  pub leave_props: LeaveProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Leave {}
impl ObjectExt for Leave {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Leave {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Leave {}
impl ActivityExt for Leave {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor likes, recommends or endorses the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Like {
  #[serde(rename = "type")]
  kind: LikeType,

  #[serde(flatten)]
  pub like_props: LikeProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Like {}
impl ObjectExt for Like {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Like {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Like {}
impl ActivityExt for Like {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has listened to the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listen {
  #[serde(rename = "type")]
  kind: ListenType,

  #[serde(flatten)]
  pub listen_props: ListenProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Listen {}
impl ObjectExt for Listen {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Listen {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Listen {}
impl ActivityExt for Listen {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is offering the object.
///
/// If specified, the target indicates the entity to which the object is being offered.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
  #[serde(rename = "type")]
  kind: OfferType,

  #[serde(flatten)]
  pub offer_props: OfferProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Offer {}
impl ObjectExt for Offer {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Offer {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Offer {}
impl ActivityExt for Offer {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Represents a question being asked.
///
/// Question objects are an extension of IntransitiveActivity. That is, the Question object is an
/// Activity, but the direct object is the question itself and therefore it would not contain an
/// object property.
///
/// Either of the anyOf and oneOf properties MAY be used to express possible answers, but a
/// Question object MUST NOT have both properties.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
  #[serde(rename = "type")]
  kind: QuestionType,

  #[serde(flatten)]
  pub question_props: QuestionProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Question {}
impl ObjectExt for Question {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Question {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Question {}
impl ActivityExt for Question {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}
impl IntransitiveActivity for Question {}

/// Indicates that the actor has read the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Read {
  #[serde(rename = "type")]
  kind: ReadType,

  #[serde(flatten)]
  pub read_props: ReadProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Read {}
impl ObjectExt for Read {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Read {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Read {}
impl ActivityExt for Read {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is rejecting the object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
  #[serde(rename = "type")]
  kind: RejectType,

  #[serde(flatten)]
  pub reject_props: RejectProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Reject {}
impl ObjectExt for Reject {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Reject {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Reject {}
impl ActivityExt for Reject {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is removing the object.
///
/// If specified, the origin indicates the context from which the object is being removed.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Remove {
  #[serde(rename = "type")]
  kind: RemoveType,

  #[serde(flatten)]
  pub remove_props: RemoveProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Remove {}
impl ObjectExt for Remove {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Remove {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Remove {}
impl ActivityExt for Remove {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// A specialization of Accept indicating that the acceptance is tentative.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentativeAccept {
  #[serde(rename = "type")]
  kind: TentativeAcceptType,

  #[serde(flatten)]
  pub tentative_accept_props: TentativeAcceptProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for TentativeAccept {}
impl ObjectExt for TentativeAccept {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for TentativeAccept {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for TentativeAccept {}
impl ActivityExt for TentativeAccept {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// A specialization of Reject in which the rejection is considered tentative.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TentativeReject {
  #[serde(rename = "type")]
  kind: TentativeRejectType,

  #[serde(flatten)]
  pub tentative_reject_props: TentativeRejectProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for TentativeReject {}
impl ObjectExt for TentativeReject {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for TentativeReject {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for TentativeReject {}
impl ActivityExt for TentativeReject {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor is traveling to target from origin.
///
/// Travel is an IntransitiveObject whose actor specifies the direct object. If the target or
/// origin are not specified, either can be determined by context.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Travel {
  #[serde(rename = "type")]
  kind: TravelType,

  #[serde(flatten)]
  pub travel_props: TravelProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Travel {}
impl ObjectExt for Travel {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Travel {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Travel {}
impl ActivityExt for Travel {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}
impl IntransitiveActivity for Travel {}

/// Indicates that the actor is undoing the object.
///
/// In most cases, the object will be an Activity describing some previously performed action (for
/// instance, a person may have previously "liked" an article but, for whatever reason, might
/// choose to undo that like at some later point in time).
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Undo {
  #[serde(rename = "type")]
  kind: UndoType,

  #[serde(flatten)]
  pub undo_props: UndoProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Undo {}
impl ObjectExt for Undo {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Undo {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Undo {}
impl ActivityExt for Undo {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has updated the object.
///
/// Note, however, that this vocabulary does not define a mechanism for describing the actual set
/// of modifications made to object.
///
/// The target and origin typically have no defined meaning.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
  #[serde(rename = "type")]
  kind: UpdateType,

  #[serde(flatten)]
  pub update_props: UpdateProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for Update {}
impl ObjectExt for Update {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for Update {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for Update {}
impl ActivityExt for Update {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}

/// Indicates that the actor has viewed the object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
  #[serde(rename = "type")]
  kind: ViewType,

  #[serde(flatten)]
  pub view_props: ViewProperties,

  #[serde(flatten)]
  pub object_props: ObjectProperties,

  #[serde(flatten)]
  pub ap_object_props: ApObjectProperties,

  #[serde(flatten)]
  pub activity_props: ActivityProperties,
}

impl Object for View {}
impl ObjectExt for View {
  fn props(&self) -> &ObjectProperties {
    &self.object_props
  }

  fn props_mut(&mut self) -> &mut ObjectProperties {
    &mut self.object_props
  }
}
impl ApObjectExt for View {
  fn props(&self) -> &ApObjectProperties {
    &self.ap_object_props
  }

  fn props_mut(&mut self) -> &mut ApObjectProperties {
    &mut self.ap_object_props
  }
}
impl Activity for View {}
impl ActivityExt for View {
  fn props(&self) -> &ActivityProperties {
    &self.activity_props
  }

  fn props_mut(&mut self) -> &mut ActivityProperties {
    &mut self.activity_props
  }
}
