use crate::{fetcher::post_or_comment::PostOrComment, objects::community::ApubCommunity};

/// The types of ActivityPub objects that reports can be created for.
#[derive(Debug)]
pub(crate) enum ReportableObjects {
  PostOrComment(PostOrComment),
  Community(ApubCommunity),
}
