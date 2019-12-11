use super::*;
use crate::schema::site;

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "site"]
pub struct Site {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize)]
#[table_name = "site"]
pub struct SiteForm {
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
}

impl Crud<SiteForm> for Site {
  fn read(conn: &PgConnection, _site_id: i32) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    site.first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, site_id: i32) -> Result<usize, Error> {
    use crate::schema::site::dsl::*;
    diesel::delete(site.find(site_id)).execute(conn)
  }

  fn create(conn: &PgConnection, new_site: &SiteForm) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    insert_into(site).values(new_site).get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, site_id: i32, new_site: &SiteForm) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    diesel::update(site.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
  }
}
