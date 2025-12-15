pub use lemmy_db_schema::{
  newtypes::OAuthProviderId,
  source::{
    oauth_account::OAuthAccount,
    oauth_provider::{AdminOAuthProvider, PublicOAuthProvider},
  },
};
pub use lemmy_db_views_site::api::{
  AuthenticateWithOauth,
  CreateOAuthProvider,
  DeleteOAuthProvider,
  EditOAuthProvider,
};
