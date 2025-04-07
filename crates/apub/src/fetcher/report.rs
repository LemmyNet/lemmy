use crate::{fetcher::PostOrComment, objects::community::ApubCommunity};
use either::Either;

// TODO don't use separate module for this

/// The types of ActivityPub objects that reports can be created for.
pub(crate) type ReportableObjects = Either<PostOrComment, ApubCommunity>;
