use crate::{
  naive_now,
  schema::{person, person_alias_1, person_alias_2},
  DbUrl,
  PersonId,
};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_apub_lib::traits::{ActorType, ApubObject};
use lemmy_utils::LemmyError;
use serde::Serialize;
use url::Url;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person"]
pub struct Person {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

/// A safe representation of person, without the sensitive info
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person"]
pub struct PersonSafe {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person_alias_1"]
pub struct PersonAlias1 {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person_alias_1"]
pub struct PersonSafeAlias1 {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person_alias_2"]
pub struct PersonAlias2 {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "person_alias_1"]
pub struct PersonSafeAlias2 {
  pub id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: DbUrl,
  pub bio: Option<String>,
  pub local: bool,
  pub banner: Option<DbUrl>,
  pub deleted: bool,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub matrix_user_id: Option<String>,
  pub admin: bool,
  pub bot_account: bool,
}

#[derive(Insertable, AsChangeset, Clone, Default)]
#[table_name = "person"]
pub struct PersonForm {
  pub name: String,
  pub display_name: Option<Option<String>>,
  pub avatar: Option<Option<DbUrl>>,
  pub banned: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub actor_id: Option<DbUrl>,
  pub bio: Option<Option<String>>,
  pub local: Option<bool>,
  pub private_key: Option<Option<String>>,
  pub public_key: Option<Option<String>>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub banner: Option<Option<DbUrl>>,
  pub deleted: Option<bool>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
  pub matrix_user_id: Option<Option<String>>,
  pub admin: Option<bool>,
  pub bot_account: Option<bool>,
}

impl ApubObject for Person {
  type DataType = PgConnection;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, LemmyError> {
    use crate::schema::person::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      person
        .filter(deleted.eq(false))
        .filter(actor_id.eq(object_id))
        .first::<Self>(conn)
        .ok(),
    )
  }

  fn delete(self, conn: &PgConnection) -> Result<(), LemmyError> {
    use crate::schema::person::dsl::*;
    diesel::update(person.find(self.id))
      .set((deleted.eq(true), updated.eq(naive_now())))
      .get_result::<Self>(conn)?;
    Ok(())
  }
}

impl ActorType for Person {
  fn is_local(&self) -> bool {
    self.local
  }
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into_inner()
  }
  fn name(&self) -> String {
    self.name.clone()
  }

  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }

  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn inbox_url(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox_url(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into_inner())
  }
}
