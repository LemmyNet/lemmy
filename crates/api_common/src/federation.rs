pub use lemmy_db_schema::{newtypes::InstanceId, source::instance::Instance};
pub use lemmy_db_views_resolve_object::ResolveObject;
pub use lemmy_db_views_resolve_object_response::ResolveObjectResponse;

pub mod admin {
  pub use lemmy_db_schema::{
    newtypes::{AdminAllowInstanceId, AdminBlockInstanceId},
    source::{
      federation_allowlist::FederationAllowList, federation_blocklist::FederationBlockList,
      federation_queue_state::FederationQueueState,
    },
  };
}
