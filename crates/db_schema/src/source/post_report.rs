// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::newtypes::{DbUrl, PersonId, PostId, PostReportId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::post_report;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(crate::source::post::Post))]
#[cfg_attr(feature = "full", table_name = "post_report")]
pub struct PostReport {
  pub id: PostReportId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "post_report")]
pub struct PostReportForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub original_post_name: String,
  pub original_post_url: Option<DbUrl>,
  pub original_post_body: Option<String>,
  pub reason: String,
}
