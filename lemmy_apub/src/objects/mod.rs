use crate::check_is_apub_id_valid;
use activitystreams::{
  base::{AsBase, BaseExt, ExtendsExt},
  markers::Base,
  mime::{FromStrError, Mime},
  object::{ApObjectExt, Object, ObjectExt, Tombstone, TombstoneExt},
};
use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use lemmy_db::{ApubObject, Crud, DbPool};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

pub(crate) mod comment;
pub(crate) mod community;
pub(crate) mod post;
pub(crate) mod private_message;
pub(crate) mod user;

/// Trait for converting an object or actor into the respective ActivityPub type.
#[async_trait::async_trait(?Send)]
pub(crate) trait ToApub {
  type ApubType;
  async fn to_apub(&self, pool: &DbPool) -> Result<Self::ApubType, LemmyError>;
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError>;
}

#[async_trait::async_trait(?Send)]
pub(crate) trait FromApub {
  type ApubType;
  /// Converts an object from ActivityPub type to Lemmy internal type.
  ///
  /// * `apub` The object to read from
  /// * `context` LemmyContext which holds DB pool, HTTP client etc
  /// * `expected_domain` If present, ensure that the domains of this and of the apub object ID are
  ///                     identical
  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    expected_domain: Option<Url>,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

#[async_trait::async_trait(?Send)]
pub(in crate::objects) trait FromApubToForm<ApubType> {
  async fn from_apub(
    apub: &ApubType,
    context: &LemmyContext,
    expected_domain: Option<Url>,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized;
}

/// Updated is actually the deletion time
fn create_tombstone<T>(
  deleted: bool,
  object_id: &str,
  updated: Option<NaiveDateTime>,
  former_type: T,
) -> Result<Tombstone, LemmyError>
where
  T: ToString,
{
  if deleted {
    if let Some(updated) = updated {
      let mut tombstone = Tombstone::new();
      tombstone.set_id(object_id.parse()?);
      tombstone.set_former_type(former_type.to_string());
      tombstone.set_deleted(convert_datetime(updated));
      Ok(tombstone)
    } else {
      Err(anyhow!("Cant convert to tombstone because updated time was None.").into())
    }
  } else {
    Err(anyhow!("Cant convert object to tombstone if it wasnt deleted").into())
  }
}

pub(in crate::objects) fn check_object_domain<T, Kind>(
  apub: &T,
  expected_domain: Option<Url>,
) -> Result<String, LemmyError>
where
  T: Base + AsBase<Kind>,
{
  let object_id = if let Some(url) = expected_domain {
    let domain = url.domain().context(location_info!())?;
    apub.id(domain)?.context(location_info!())?
  } else {
    apub.id_unchecked().context(location_info!())?
  };
  check_is_apub_id_valid(&object_id)?;
  Ok(object_id.to_string())
}

pub(in crate::objects) fn set_content_and_source<T, Kind1, Kind2>(
  object: &mut T,
  markdown_text: &str,
) -> Result<(), LemmyError>
where
  T: ApObjectExt<Kind1> + ObjectExt<Kind2> + AsBase<Kind2>,
{
  let mut source = Object::<()>::new_none_type();
  source
    .set_content(markdown_text)
    .set_media_type(mime_markdown()?);
  object.set_source(source.into_any_base()?);

  // set `content` to markdown for compatibility with older Lemmy versions
  // TODO: change this to HTML in a while
  object.set_content(markdown_text);
  object.set_media_type(mime_markdown()?);
  //object.set_content(markdown_to_html(markdown_text));
  Ok(())
}

pub(in crate::objects) fn get_source_markdown_value<T, Kind1, Kind2>(
  object: &T,
) -> Result<Option<String>, LemmyError>
where
  T: ApObjectExt<Kind1> + ObjectExt<Kind2> + AsBase<Kind2>,
{
  let content = object
    .content()
    .map(|s| s.as_single_xsd_string())
    .flatten()
    .map(|s| s.to_string());
  if content.is_some() {
    let source = object.source();
    // updated lemmy version, read markdown from `source.content`
    if let Some(source) = source {
      let source = Object::<()>::from_any_base(source.to_owned())?.context(location_info!())?;
      check_is_markdown(source.media_type())?;
      let source_content = source
        .content()
        .map(|s| s.as_single_xsd_string())
        .flatten()
        .context(location_info!())?
        .to_string();
      return Ok(Some(source_content));
    }
    // older lemmy version, read markdown from `content`
    // TODO: remove this after a while
    else {
      return Ok(content);
    }
  }
  Ok(None)
}

pub(in crate::objects) fn mime_markdown() -> Result<Mime, FromStrError> {
  "text/markdown".parse()
}

pub(in crate::objects) fn check_is_markdown(mime: Option<&Mime>) -> Result<(), LemmyError> {
  let mime = mime.context(location_info!())?;
  if !mime.eq(&mime_markdown()?) {
    Err(LemmyError::from(anyhow!(
      "Lemmy only supports markdown content"
    )))
  } else {
    Ok(())
  }
}

/// Converts an ActivityPub object (eg `Note`) to a database object (eg `Comment`). If an object
/// with the same ActivityPub ID already exists in the database, it is returned directly. Otherwise
/// the apub object is parsed, inserted and returned.
pub(in crate::objects) async fn get_object_from_apub<From, Kind, To, ToForm>(
  from: &From,
  context: &LemmyContext,
  expected_domain: Option<Url>,
  request_counter: &mut i32,
) -> Result<To, LemmyError>
where
  From: BaseExt<Kind>,
  To: ApubObject + Crud<ToForm> + Send + 'static,
  ToForm: FromApubToForm<From> + Send + 'static,
{
  let object_id = from.id_unchecked().context(location_info!())?.to_owned();
  let object_in_database = blocking(context.pool(), move |conn| {
    To::read_from_apub_id(conn, object_id.as_str())
  })
  .await?;

  // if we already have the object in our database, return that directly
  if let Ok(to) = object_in_database {
    Ok(to)
  }
  // if we dont have it, parse from apub and insert into database
  // TODO: this is insecure, a `Like/Post` could result in a non-existent post from a different
  //       instance being inserted into our database. we should request the object over http in that
  //       case. this might happen exactly in the case where expected_domain = None, but i'm not sure.
  else {
    let to_form = ToForm::from_apub(&from, context, expected_domain, request_counter).await?;

    let to = blocking(context.pool(), move |conn| To::create(conn, &to_form)).await??;
    Ok(to)
  }
}
