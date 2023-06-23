use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Debug, Display},
};
use tracing_error::SpanTrace;

#[derive(serde::Serialize)]
struct ApiError {
  error: String,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(tag = "error_type", content = "message")]
pub enum LemmyErrorType {
  ReportReasonRequired,
  ReportTooLong,
  NotAModerator,
  NotAnAdmin,
  CannotBlockYourself,
  CannotBlockAdmin,
  CouldNotUpdateUser,
  PasswordsDoNotMatch,
  PasswordIncorrect,
  EmailNotVerified,
  EmailRequired,
  CouldNotUpdateComment,
  CouldNotUpdatePrivateMessage,
  CannotLeaveAdmin,
  NoLinesInHtml,
  SiteMetadataPageIsNotDoctypeHtml,
  PictrsResponseError(String),
  PictrsPurgeResponseError(String),
  ImageUrlMissingPathSegments,
  ImageUrlMissingLastPathSegment,
  PictrsApiKeyNotProvided,
  NoContentTypeHeader,
  NotAnImageType,
  NotAModOrAdmin,
  NoAdmins,
  NotTopAdmin,
  NotTopMod,
  NotLoggedIn,
  SiteBan,
  Deleted,
  BannedFromCommunity,
  CouldNotFindCommunity,
  PersonIsBlocked,
  DownvotesAreDisabled,
  InstanceIsPrivate,
  InvalidPassword,
  SiteDescriptionLengthOverflow,
  HoneypotFailed,
  RegistrationApplicationIsPending,
  PrivateInstanceCannotHaveFederationEnabled,
  Locked,
  CouldNotCreateComment,
  MaxCommentDepthReached,
  EditCommentNotAllowed,
  OnlyAdminsCanCreateCommunities,
  CommunityAlreadyExists,
  LanguageNotAllowed,
  OnlyModsCanPostInCommunity,
  CouldNotUpdatePost,
  EditPostNotAllowed,
  CouldNotFindPost,
  EditPrivateMessageNotAllowed,
  SiteAlreadyExists,
  ApplicationQuestionRequired,
  InvalidDefaultPostListingType,
  RegistrationClosed,
  RegistrationApplicationAnswerRequired,
  EmailAlreadyExists,
  FederationError(&'static str),
  FederationForbiddenByStrictAllowList,
  PersonIsBannedFromCommunity,
  ObjectIsNotPublic,
  InvalidCommunity,
  CannotCreatePostOrCommentInDeletedOrRemovedCommunity,
  CannotReceivePage,
  NewPostCannotBeLocked,
  OnlyLocalAdminCanRemoveCommunity,
  OnlyLocalAdminCanRestoreCommunity,
  NoIdGiven,
  CouldNotFindUsernameOrEmail,
  InvalidQuery,
  ObjectNotLocal,
  PostIsLocked,
  PersonIsBannedFromSite,
  InvalidVoteValue,
  PageDoesNotSpecifyCreator,
  PageDoesNotSpecifyGroup,
  NoCommunityFoundInCC,
  NoEmailSetup,
  EmailSmtpServerNeedsAPort,
  MissingAnEmail,
  RateLimitError,
  InvalidName,
  InvalidDisplayName,
  InvalidMatrixId,
  InvalidPostTitle,
  InvalidBodyField,
  BioLengthOverflow,
  MissingTotpToken,
  IncorrectTotpToken,
  CouldNotParseTotpSecret,
  CouldNotLikeComment,
  CouldNotSaveComment,
  CouldNotCreateReport,
  CouldNotResolveReport,
  CommunityModeratorAlreadyExists,
  CommunityUserIsAlreadyBanned,
  CommunityBlockAlreadyExists,
  CommunityFollowerAlreadyExists,
  CouldNotUpdateCommunityHiddenStatus,
  PersonBlockAlreadyExists,
  UserAlreadyExists,
  TokenNotFound,
  CouldNotLikePost,
  CouldNotSavePost,
  CouldNotMarkPostAsRead,
  CouldNotUpdateCommunity,
  CouldNotUpdateReplies,
  CouldNotUpdatePersonMentions,
  PostTitleTooLong,
  CouldNotCreatePost,
  CouldNotCreatePrivateMessage,
  CouldNotUpdatePrivate,
  SystemErrLogin,
  CouldNotSetAllRegistrationsAccepted,
  CouldNotSetAllEmailVerified,
  Banned,
  CouldNotGetComments,
  CouldNotGetPosts,
  InvalidUrl,
  EmailSendFailed,
  Slurs,
  CouldNotGenerateTotp,
  CouldNotFindObject,
  RegistrationDenied(String),
  FederationDisabled,
  DomainBlocked,
  DomainNotInAllowList,
  FederationDisabledByStrictAllowList,
}

pub type LemmyResult<T> = Result<T, LemmyError>;

pub struct LemmyError {
  pub error_type: Option<LemmyErrorType>,
  pub inner: anyhow::Error,
  pub context: SpanTrace,
}

impl LemmyError {
  /// Create LemmyError from a message, including stack trace
  pub fn from_message(error_type: LemmyErrorType) -> Self {
    let inner = anyhow::anyhow!("{}", error_type);
    LemmyError {
      error_type: Some(error_type),
      inner,
      context: SpanTrace::capture(),
    }
  }

  /// Create a LemmyError from error and message, including stack trace
  pub fn from_error_message<E>(error: E, error_type: LemmyErrorType) -> Self
  where
    E: Into<anyhow::Error>,
  {
    LemmyError {
      error_type: Some(error_type),
      inner: error.into(),
      context: SpanTrace::capture(),
    }
  }

  /// Add message to existing LemmyError (or overwrite existing error)
  pub fn with_message(self, error_type: LemmyErrorType) -> Self {
    LemmyError {
      error_type: Some(error_type),
      ..self
    }
  }

  pub fn to_json(&self) -> Result<String, Self> {
    let api_error = match &self.error_type {
      Some(error) => ApiError {
        error: error.to_string(),
      },
      None => ApiError {
        error: "Unknown".into(),
      },
    };

    Ok(serde_json::to_string(&api_error)?)
  }
}

impl<T> From<T> for LemmyError
where
  T: Into<anyhow::Error>,
{
  fn from(t: T) -> Self {
    LemmyError {
      error_type: None,
      inner: t.into(),
      context: SpanTrace::capture(),
    }
  }
}

impl Debug for LemmyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("LemmyError")
      .field("message", &self.error_type)
      .field("inner", &self.inner)
      .field("context", &self.context)
      .finish()
  }
}

impl Display for LemmyError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(message) = &self.error_type {
      write!(f, "{message}: ")?;
    }
    writeln!(f, "{}", self.inner)?;
    fmt::Display::fmt(&self.context, f)
  }
}

impl actix_web::error::ResponseError for LemmyError {
  fn status_code(&self) -> http::StatusCode {
    match self.inner.downcast_ref::<diesel::result::Error>() {
      Some(diesel::result::Error::NotFound) => http::StatusCode::NOT_FOUND,
      _ => http::StatusCode::BAD_REQUEST,
    }
  }

  fn error_response(&self) -> actix_web::HttpResponse {
    if let Some(message) = &self.error_type {
      actix_web::HttpResponse::build(self.status_code()).json(ApiError {
        error: message.to_string(),
      })
    } else {
      actix_web::HttpResponse::build(self.status_code())
        .content_type("text/plain")
        .body(self.inner.to_string())
    }
  }
}
