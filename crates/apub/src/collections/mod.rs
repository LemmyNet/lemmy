// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use lemmy_websocket::LemmyContext;

use crate::objects::community::ApubCommunity;

pub(crate) mod community_moderators;
pub(crate) mod community_outbox;

/// Put community in the data, so we dont have to read it again from the database.
pub(crate) struct CommunityContext(pub ApubCommunity, pub LemmyContext);
