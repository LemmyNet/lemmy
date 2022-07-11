// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(feature = "full")]
use crate::schema::secret;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "secret")]
pub struct Secret {
  pub id: i32,
  pub jwt_secret: String,
}
