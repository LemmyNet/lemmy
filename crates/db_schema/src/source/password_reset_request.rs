// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::newtypes::LocalUserId;

#[cfg(feature = "full")]
use crate::schema::password_reset_request;

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "password_reset_request")]
pub struct PasswordResetRequest {
  pub id: i32,
  pub token_encrypted: String,
  pub published: chrono::NaiveDateTime,
  pub local_user_id: LocalUserId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "password_reset_request")]
pub struct PasswordResetRequestForm {
  pub local_user_id: LocalUserId,
  pub token_encrypted: String,
}
