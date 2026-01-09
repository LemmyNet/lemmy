use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, panic::Location};
use strum::{Display, EnumIter};

/// Errors used in the API, all of these are translated in lemmy-ui.
#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, EnumIter, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
#[non_exhaustive]
pub enum LemmyErrorType {
  BlockKeywordTooShort,
  BlockKeywordTooLong,
  CouldntUpdate,
  CouldntCreate,
  ReportReasonRequired,
  ReportTooLong,
  NotAModerator,
  NotAnAdmin,
  CantBlockYourself,
  CantNoteYourself,
  CantBlockAdmin,
  PasswordsDoNotMatch,
  EmailNotVerified,
  EmailRequired,
  CannotLeaveAdmin,
  CannotLeaveMod,
  PictrsResponseError(String),
  PictrsPurgeResponseError(String),
  PictrsApiKeyNotProvided,
  PictrsInvalidImageUpload(String),
  NoContentTypeHeader,
  NotAnImageType,
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
  MaxCommentDepthReached,
  NoCommentEditAllowed,
  OnlyAdminsCanCreateCommunities,
  AlreadyExists,
  LanguageNotAllowed,
  NoPostEditAllowed,
  NsfwNotAllowed,
  EditPrivateMessageNotAllowed,
  ApplicationQuestionRequired,
  InvalidDefaultPostListingType,
  RegistrationClosed,
  RegistrationApplicationAnswerRequired,
  RegistrationUsernameRequired,
  EmailAlreadyTaken,
  UsernameAlreadyTaken,
  PersonIsBannedFromCommunity,
  NoIdGiven,
  IncorrectLogin,
  NoEmailSetup,
  LocalSiteNotSetup,
  InvalidEmailAddress(String),
  InvalidName,
  InvalidCodeVerifier,
  InvalidDisplayName,
  InvalidMatrixId,
  InvalidPostTitle,
  InvalidBodyField,
  BioLengthOverflow,
  AltTextLengthOverflow,
  CouldntParseTotpSecret,
  CouldntGenerateTotp,
  MissingTotpToken,
  MissingTotpSecret,
  IncorrectTotpToken,
  TotpAlreadyEnabled,
  BlockedUrl,
  InvalidUrl,
  EmailSendFailed,
  Slurs,
  RegistrationDenied(String),
  SiteNameRequired,
  SiteNameLengthOverflow,
  PermissiveRegex,
  InvalidRegex,
  CaptchaIncorrect,
  CouldntCreateAudioCaptcha,
  CouldntCreateImageCaptcha,
  InvalidUrlScheme,
  ContradictingFilters,
  /// Thrown when an API call is submitted with more than 1000 array elements, see
  /// [[MAX_API_PARAM_ELEMENTS]]
  TooManyItems,
  BanExpirationInPast,
  InvalidUnixTime,
  InvalidBotAction,
  TagNotInCommunity,
  CantBlockLocalInstance,
  Unknown(String),
  UrlLengthOverflow,
  OauthAuthorizationInvalid,
  OauthLoginFailed,
  OauthRegistrationClosed,
  NotFound,
  PostScheduleTimeMustBeInFuture,
  TooManyScheduledPosts,
  CannotCombineFederationBlocklistAndAllowlist,
  CouldntParsePaginationToken,
  PluginError(String),
  InvalidFetchLimit,
  EmailNotificationsDisabled,
  MultiCommunityUpdateWrongUser,
  CannotCombineCommunityIdAndMultiCommunityId,
  MultiCommunityEntryLimitReached,
  TooManyRequests,
  ResolveObjectFailed(String),
  #[serde(untagged)]
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  UntranslatedError(Option<UntranslatedError>),
}

/// These errors are only used for federation or internally and dont need to be translated.
#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
#[non_exhaustive]
pub enum UntranslatedError {
  InvalidCommunity,
  CannotCreatePostOrCommentInDeletedOrRemovedCommunity,
  CannotReceivePage,
  OnlyLocalAdminCanRemoveCommunity,
  OnlyLocalAdminCanRestoreCommunity,
  PostIsLocked,
  PersonIsBannedFromSite(String),
  InvalidVoteValue,
  PageDoesNotSpecifyCreator,
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
  InvalidFollow(String),
  PurgeInvalidImageUrl,
  Unreachable,
  CouldntSendWebmention,
  /// A remote community sent an activity to us, but actually no local user follows the community
  /// so the activity was rejected.
  CommunityHasNoFollowers(String),
}

cfg_if! {
  if #[cfg(feature = "full")] {

    use std::{fmt};
    pub type LemmyResult<T> = Result<T, LemmyError>;

    pub struct LemmyError {
      pub error_type: LemmyErrorType,
      pub inner: anyhow::Error,
      pub caller: Location<'static>,
    }

    /// Maximum number of items in an array passed as API parameter. See [[LemmyErrorType::TooManyItems]]
    pub(crate) const MAX_API_PARAM_ELEMENTS: usize = 10_000;

    impl<T> From<T> for LemmyError
    where
      T: Into<anyhow::Error>,
    {
    #[track_caller]
      fn from(t: T) -> Self {
        let cause = t.into();
        let error_type = match cause.downcast_ref::<diesel::result::Error>() {
          Some(&diesel::NotFound) => LemmyErrorType::NotFound,
          _ => LemmyErrorType::Unknown(format!("{}", &cause))
      };
        LemmyError {
          error_type,
          inner: cause,
          caller: *Location::caller(),
        }
      }
    }

    impl Debug for LemmyError {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LemmyError")
         .field("message", &self.error_type)
         .field("caller", &format_args!("{}", self.caller))
         .field("inner", &self.inner)
         .finish()
      }
    }

    impl fmt::Display for LemmyError {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.error_type)?;
        write!(f, "{}", self.caller)?;
        write!(f, "{}", self.inner)?;
        Ok(())
      }
    }

    impl actix_web::error::ResponseError for LemmyError {
      fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_type {
          LemmyErrorType::IncorrectLogin => actix_web::http::StatusCode::UNAUTHORIZED,
          LemmyErrorType::NotFound => actix_web::http::StatusCode::NOT_FOUND,
          _ => actix_web::http::StatusCode::BAD_REQUEST,
        }
      }

      fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(&self.error_type)
      }
    }

    impl From<LemmyErrorType> for LemmyError {
    #[track_caller]
      fn from(error_type: LemmyErrorType) -> Self {

        let inner = anyhow::anyhow!("{}", error_type);
        LemmyError {
          error_type,
          inner,
          caller: *Location::caller(),
        }
      }
    }

    impl From<UntranslatedError> for LemmyError {
    #[track_caller]
      fn from(error_type: UntranslatedError) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        LemmyError {
          error_type: LemmyErrorType::UntranslatedError( Some(error_type) ),
          inner,
          caller: *Location::caller(),
        }
      }
    }

    impl From<UntranslatedError> for LemmyErrorType {
      fn from(error: UntranslatedError) -> Self {
        LemmyErrorType::UntranslatedError (Some(error) )
      }
    }

    pub trait LemmyErrorExt<T, E: Into<anyhow::Error>> {
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T>;
    }

    impl<T, E: Into<anyhow::Error>> LemmyErrorExt<T, E> for Result<T, E> {
    #[track_caller]
      fn with_lemmy_type(self, error_type: LemmyErrorType) -> LemmyResult<T> {
        self.map_err(|error| LemmyError {
          error_type,
          inner: error.into(),
          caller: *Location::caller(),
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

      #[test]
      fn untranslated_error_format() -> LemmyResult<()> {
        let err = LemmyError::from(UntranslatedError::DomainBlocked("test".to_string())).error_response();
        let json = String::from_utf8(err.into_body().try_into_bytes().unwrap_or_default().to_vec())?;
        assert_eq!(&json, r#"{"error":"domain_blocked","message":"test"}"#);

        Ok(())
      }

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
    }
  }
}
