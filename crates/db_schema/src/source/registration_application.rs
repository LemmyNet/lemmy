use crate::newtypes::{LocalUserId, PersonId};
#[cfg(feature = "full")]
use crate::schema::registration_application;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = registration_application))]
#[cfg_attr(feature = "full", ts(export))]
/// A registration application.
pub struct RegistrationApplication {
    pub id: i32,
    pub local_user_id: LocalUserId,
    pub answer: String,
    pub admin_id: Option<PersonId>,
    pub deny_reason: Option<String>,
    pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = registration_application))]
pub struct RegistrationApplicationInsertForm {
    pub local_user_id: LocalUserId,
    pub answer: String,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = registration_application))]
pub struct RegistrationApplicationUpdateForm {
    pub admin_id: Option<Option<PersonId>>,
    pub deny_reason: Option<Option<String>>,
}
