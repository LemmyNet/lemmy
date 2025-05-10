pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;
pub mod private_message;

use comment::ApubComment;
use community::ApubCommunity;
use either::Either;
use instance::ApubSite;
use person::ApubPerson;
use post::ApubPost;

pub type PostOrComment = Either<ApubPost, ApubComment>;

pub type ReportableObjects = Either<PostOrComment, ApubCommunity>;

pub type SearchableObjects = Either<PostOrComment, UserOrCommunity>;

pub type UserOrCommunity = Either<ApubPerson, ApubCommunity>;

pub type SiteOrCommunityOrUser = Either<ApubSite, UserOrCommunity>;
