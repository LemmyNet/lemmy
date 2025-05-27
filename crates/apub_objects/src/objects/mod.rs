pub mod comment;
pub mod community;
pub mod instance;
pub mod multi_community;
pub mod multi_community_collection;
pub mod person;
pub mod post;
pub mod private_message;

use comment::ApubComment;
use community::ApubCommunity;
use either::Either;
use instance::ApubSite;
use multi_community::ApubMultiCommunity;
use person::ApubPerson;
use post::ApubPost;

pub type PostOrComment = Either<ApubPost, ApubComment>;

pub type SearchableObjects = Either<Either<PostOrComment, UserOrCommunity>, ApubMultiCommunity>;

pub type UserOrCommunity = Either<ApubPerson, ApubCommunity>;

pub type SiteOrCommunityOrUser = Either<ApubSite, UserOrCommunity>;
