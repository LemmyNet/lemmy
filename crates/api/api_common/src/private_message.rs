pub use lemmy_db_schema::{newtypes::PrivateMessageId, source::private_message::PrivateMessage};
pub use lemmy_db_views_private_message::{PrivateMessageView, api::PrivateMessageResponse};

pub mod actions {
  pub use lemmy_db_views_private_message::api::{
    CreatePrivateMessage,
    DeletePrivateMessage,
    EditPrivateMessage,
  };
}
