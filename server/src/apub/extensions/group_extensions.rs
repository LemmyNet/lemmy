use crate::db::{category::Category, Crud};
use activitystreams::{ext::Extension, Actor};
use diesel::PgConnection;
use failure::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupExtension {
  pub category: GroupCategory,
  pub sensitive: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCategory {
  // Using a string because that's how Peertube does it.
  pub identifier: String,
  pub name: String,
}

impl GroupExtension {
  pub fn new(
    conn: &PgConnection,
    category_id: i32,
    sensitive: bool,
  ) -> Result<GroupExtension, Error> {
    let category = Category::read(conn, category_id)?;
    let group_category = GroupCategory {
      identifier: category_id.to_string(),
      name: category.name,
    };
    Ok(GroupExtension {
      category: group_category,
      sensitive,
    })
  }
}

impl<T> Extension<T> for GroupExtension where T: Actor {}
