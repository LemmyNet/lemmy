// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::structs::SiteView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::SiteAggregates,
  schema::{site, site_aggregates},
  source::site::Site,
};

impl SiteView {
  pub fn read_local(conn: &PgConnection) -> Result<Self, Error> {
    let (mut site, counts) = site::table
      .inner_join(site_aggregates::table)
      .select((site::all_columns, site_aggregates::all_columns))
      .order_by(site::id)
      .first::<(Site, SiteAggregates)>(conn)?;

    site.private_key = None;
    Ok(SiteView { site, counts })
  }
}
