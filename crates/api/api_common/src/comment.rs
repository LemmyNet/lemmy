pub use lemmy_db_schema::{
  newtypes::CommentId,
  source::comment::{Comment, CommentActions},
};
pub use lemmy_db_views_comment::{
  api::{CommentResponse, GetComment, GetComments, GetCommentsResponse, GetCommentsSlimResponse},
  CommentSlimView,
  CommentView,
};

pub mod actions {
  pub use lemmy_db_views_comment::api::{
    CreateComment,
    CreateCommentLike,
    DeleteComment,
    EditComment,
    SaveComment,
  };

  pub mod moderation {
    pub use lemmy_db_views_comment::api::{
      DistinguishComment,
      ListCommentLikes,
      PurgeComment,
      RemoveComment,
    };
    pub use lemmy_db_views_vote::api::ListCommentLikesResponse;
  }
}
