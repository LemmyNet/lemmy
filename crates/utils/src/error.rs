use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::{Display, EnumIter};

#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, Hash)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
#[non_exhaustive]
// TODO: order these based on the crate they belong to (utils, federation, db, api)
pub enum LemmyErrorType {
  BlockKeywordTooShort,
  BlockKeywordTooLong,
  ReportReasonRequired,
  ReportTooLong,
  NotAModerator,
  NotAnAdmin,
  CantBlockYourself,
  CantBlockAdmin,
  CouldntUpdateUser,
  PasswordsDoNotMatch,
  EmailNotVerified,
  EmailRequired,
  CouldntUpdateComment,
  CouldntUpdatePrivateMessage,
  CannotLeaveAdmin,
  PictrsResponseError(String),
  PictrsPurgeResponseError(String),
  ImageUrlMissingPathSegments,
  ImageUrlMissingLastPathSegment,
  PictrsApiKeyNotProvided,
  NoContentTypeHeader,
  NotAnImageType,
  InvalidImageUpload,
  ImageUploadDisabled,
  NotAModOrAdmin,
  NotTopMod,
  NotLoggedIn,
  NotHigherMod,
  NotHigherAdmin,
  SiteBan,
  Deleted,
  PersonIsBlocked,
  CommunityIsBlocked,
  InstanceIsBlocked,
  InstanceIsPrivate,
  /// Password must be between 10 and 60 characters
  InvalidPassword,
  SiteDescriptionLengthOverflow,
  HoneypotFailed,
  RegistrationApplicationIsPending,
  Locked,
  CouldntCreateComment,
  MaxCommentDepthReached,
  NoCommentEditAllowed,
  OnlyAdminsCanCreateCommunities,
  CommunityAlreadyExists,
  LanguageNotAllowed,
  CouldntUpdatePost,
  NoPostEditAllowed,
  NsfwNotAllowed,
  EditPrivateMessageNotAllowed,
  SiteAlreadyExists,
  ApplicationQuestionRequired,
  InvalidDefaultPostListingType,
  RegistrationClosed,
  RegistrationApplicationAnswerRequired,
  RegistrationUsernameRequired,
  EmailAlreadyExists,
  UsernameAlreadyExists,
  PersonIsBannedFromCommunity,
  NoIdGiven,
  IncorrectLogin,
  ObjectNotLocal,
  NoEmailSetup,
  LocalSiteNotSetup,
  InvalidEmailAddress(String),
  RateLimitError,
  InvalidName,
  InvalidCodeVerifier,
  InvalidDisplayName,
  InvalidMatrixId,
  InvalidPostTitle,
  InvalidBodyField,
  BioLengthOverflow,
  AltTextLengthOverflow,
  MissingTotpToken,
  MissingTotpSecret,
  IncorrectTotpToken,
  CouldntParseTotpSecret,
  CouldntGenerateTotp,
  TotpAlreadyEnabled,
  CouldntLikeComment,
  CouldntSaveComment,
  CouldntCreateReport,
  CouldntResolveReport,
  CommunityModeratorAlreadyExists,
  CommunityUserAlreadyBanned,
  CommunityBlockAlreadyExists,
  CommunityFollowerAlreadyExists,
  PersonBlockAlreadyExists,
  UserAlreadyExists,
  CouldntLikePost,
  CouldntSavePost,
  CouldntMarkPostAsRead,
  CouldntUpdateReadComments,
  CouldntHidePost,
  CouldntUpdateCommunity,
  CouldntUpdateReplies,
  CouldntUpdatePersonCommentMentions,
  CouldntCreatePost,
  CouldntCreatePrivateMessage,
  CouldntUpdatePrivate,
  SystemErrLogin,
  CouldntSetAllRegistrationsAccepted,
  CouldntSetAllEmailVerified,
  BlockedUrl,
  CouldntGetComments,
  CouldntGetPosts,
  InvalidUrl,
  EmailSendFailed,
  Slurs,
  RegistrationDenied {
    #[cfg_attr(feature = "full", ts(optional))]
    reason: Option<String>,
  },
  SiteNameRequired,
  SiteNameLengthOverflow,
  PermissiveRegex,
  InvalidRegex,
  CaptchaIncorrect,
  CouldntCreateAudioCaptcha,
  CouldntCreateImageCaptcha,
  InvalidUrlScheme,
  CouldntSendWebmention,
  ContradictingFilters,
  InstanceBlockAlreadyExists,
  /// Thrown when an API call is submitted with more than 1000 array elements, see
  /// [[MAX_API_PARAM_ELEMENTS]]
  TooManyItems,
  BanExpirationInPast,
  InvalidUnixTime,
  InvalidBotAction,
  InvalidTagName,
  TagNotInCommunity,
  CantBlockLocalInstance,
  Unknown(String),
  UrlLengthOverflow,
  OauthAuthorizationInvalid,
  OauthLoginFailed,
  OauthRegistrationClosed,
  CouldntDeleteOauthProvider,
  NotFound,
  CommunityHasNoFollowers,
  PostScheduleTimeMustBeInFuture,
  TooManyScheduledPosts,
  CannotCombineFederationBlocklistAndAllowlist,
  FederationError {
    #[cfg_attr(feature = "full", ts(optional))]
    error: Option<FederationError>,
  },
  CouldntParsePaginationToken,
  PluginError(String),
  CouldntSubscribePost,
}

/// Federation related errors, these dont need to be translated.
#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, Hash)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
#[non_exhaustive]
pub enum FederationError {
  InvalidCommunity,
  CannotCreatePostOrCommentInDeletedOrRemovedCommunity,
  CannotReceivePage,
  OnlyLocalAdminCanRemoveCommunity,
  OnlyLocalAdminCanRestoreCommunity,
  PostIsLocked,
  PersonIsBannedFromSite(String),
  InvalidVoteValue,
  PageDoesNotSpecifyCreator,
  CouldntGetComments,
  CouldntGetPosts,
  FederationDisabled,
  DomainBlocked(String),
  DomainNotInAllowList(String),
  FederationDisabledByStrictAllowList,
  ContradictingFilters,
  UrlWithoutDomain,
  InboxTimeout,
  CantDeleteSite,
  ObjectIsNotPublic,
  ObjectIsNotPrivate,
  PlatformLackingPrivateCommunitySupport,
  Unreachable,
}

cfg_if! {
  if #[cfg(feature = "full")] {

    use std::{fmt, backtrace::Backtrace};
    pub type LemmyResult<T> = Result<T, LemmyError>;

    pub struct LemmyError {
      pub error_type: LemmyErrorType,
      pub inner: anyhow::Error,
      pub context: Backtrace,
    }

    /// Maximum number of items in an array passed as API parameter. See [[LemmyErrorType::TooManyItems]]
    pub const MAX_API_PARAM_ELEMENTS: usize = 10_000;

    impl<T> From<T> for LemmyError
    where
      T: Into<anyhow::Error>,
    {
      fn from(t: T) -> Self {
        let cause = t.into();
        let error_type = match cause.downcast_ref::<diesel::result::Error>() {
          Some(&diesel::NotFound) => LemmyErrorType::NotFound,
          _ => LemmyErrorType::Unknown(format!("{}", &cause))
      };
        LemmyError {
          error_type,
          inner: cause,
          context: Backtrace::capture(),
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

    impl fmt::Display for LemmyError {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.error_type)?;
        writeln!(f, "{}", self.inner)?;
        fmt::Display::fmt(&self.context, f)
      }
    }

    impl actix_web::error::ResponseError for LemmyError {
      fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_type {
          LemmyErrorType::IncorrectLogin => actix_web::http::StatusCode::UNAUTHORIZED,
          LemmyErrorType::NotFound => actix_web::http::StatusCode::NOT_FOUND,
          LemmyErrorType::RateLimitError => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
          _ => actix_web::http::StatusCode::BAD_REQUEST,
        }
      }

      fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(&self.error_type)
      }
    }

    impl From<LemmyErrorType> for LemmyError {
      fn from(error_type: LemmyErrorType) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        LemmyError {
          error_type,
          inner,
          context: Backtrace::capture(),
        }
      }
    }

    impl From<FederationError> for LemmyError {
      fn from(error_type: FederationError) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        LemmyError {
          error_type: LemmyErrorType::FederationError { error: Some(error_type) },
          inner,
          context: Backtrace::capture(),
        }
      }
    }

    impl From<FederationError> for LemmyErrorType {
      fn from(error: FederationError) -> Self {
        LemmyErrorType::FederationError { error: Some(error) }
      }
    }

    pub trait LemmyErrorExt<T, E: Into<anyhow::Error>> {
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T>;
    }

    impl<T, E: Into<anyhow::Error>> LemmyErrorExt<T, E> for Result<T, E> {
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T> {
        self.map_err(|error| LemmyError {
          error_type,
          inner: error.into(),
          context: Backtrace::capture(),
        })
      }
    }
    pub trait LemmyErrorExt2<T> {
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T>;
      fn into_anyhow(self) -> Result<T, anyhow::Error>;
    }

    impl<T> LemmyErrorExt2<T> for LemmyResult<T> {
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T> {
        self.map_err(|mut e| {
          e.error_type = error_type;
          e
        })
      }
      // this function can't be an impl From or similar because it would conflict with one of the other broad Into<> implementations
      fn into_anyhow(self) -> Result<T, anyhow::Error> {
        self.map_err(|e| e.inner)
      }
    }

    #[cfg(test)]
    mod tests {
      #![allow(clippy::indexing_slicing)]
      use super::*;
      use actix_web::{body::MessageBody, ResponseError};
      use pretty_assertions::assert_eq;
      use std::fs::read_to_string;
      use strum::IntoEnumIterator;

      #[test]
      fn deserializes_no_message() -> LemmyResult<()> {
        let err = LemmyError::from(LemmyErrorType::BlockedUrl).error_response();
        let json = String::from_utf8(err.into_body().try_into_bytes().unwrap_or_default().to_vec())?;
        assert_eq!(&json, "{\"error\":\"blocked_url\"}");

        Ok(())
      }

      #[test]
      fn deserializes_with_message() -> LemmyResult<()> {
        let reg_banned = LemmyErrorType::PictrsResponseError(String::from("reason"));
        let err = LemmyError::from(reg_banned).error_response();
        let json = String::from_utf8(err.into_body().try_into_bytes().unwrap_or_default().to_vec())?;
        assert_eq!(
          &json,
          "{\"error\":\"pictrs_response_error\",\"message\":\"reason\"}"
        );

        Ok(())
      }

      #[test]
      fn test_convert_diesel_errors() {
        let not_found_error = LemmyError::from(diesel::NotFound);
        assert_eq!(LemmyErrorType::NotFound, not_found_error.error_type);
        assert_eq!(404, not_found_error.status_code());

        let other_error = LemmyError::from(diesel::result::Error::NotInTransaction);
        assert!(matches!(other_error.error_type, LemmyErrorType::Unknown{..}));
        assert_eq!(400, other_error.status_code());
      }

      /// Check if errors match translations. Disabled because many are not translated at all.
      #[test]
      #[ignore]
      fn test_translations_match() -> LemmyResult<()> {
        #[derive(Deserialize)]
        struct Err {
          error: String,
        }

        let translations = read_to_string("translations/translations/en.json")?;

        for e in LemmyErrorType::iter() {
          let msg = serde_json::to_string(&e)?;
          let msg: Err = serde_json::from_str(&msg)?;
          let msg = msg.error;
          assert!(translations.contains(&format!("\"{msg}\"")), "{msg}");
        }

        Ok(())
      }
    }
  }
}
