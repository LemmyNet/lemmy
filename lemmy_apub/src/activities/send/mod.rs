use lemmy_utils::settings::Settings;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod comment;
pub mod community;
pub mod post;
pub mod private_message;
pub mod user;

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
