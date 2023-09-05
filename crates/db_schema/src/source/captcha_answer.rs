#[cfg(feature = "full")]
use crate::schema::captcha_answer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(table_name = captcha_answer))]
pub struct CaptchaAnswer {
    pub id: i32,
    pub uuid: Uuid,
    pub answer: String,
    pub published: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(table_name = captcha_answer))]
pub struct CheckCaptchaAnswer {
    pub uuid: Uuid,
    pub answer: String,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = captcha_answer))]
pub struct CaptchaAnswerForm {
    pub answer: String,
}
