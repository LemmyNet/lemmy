use lemmy_utils::settings::structs::Settings;
use url::{ParseError, Url};
use uuid::Uuid;

pub(crate) mod comment;
pub(crate) mod community;
pub(crate) mod person;
pub(crate) mod post;
pub(crate) mod private_message;

/// Generate a unique ID for an activity, in the format:
/// `http(s)://example.com/receive/create/202daf0a-1489-45df-8d2e-c8a3173fed36`
fn generate_activity_id<T>(kind: T) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    Settings::get().get_protocol_and_hostname(),
    kind.to_string().to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}
