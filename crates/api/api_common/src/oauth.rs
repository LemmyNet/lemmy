pub use lemmy_db_schema::source::{
  oauth_account::OAuthAccount,
  oauth_provider::{AdminOAuthProvider, PublicOAuthProvider},
};
pub use lemmy_db_schema_file::newtypes::OAuthProviderId;
pub use lemmy_db_views_site::api::{
  AuthenticateWithOauth,
  CreateOAuthProvider,
  DeleteOAuthProvider,
  EditOAuthProvider,
};
