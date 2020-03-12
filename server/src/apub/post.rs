use crate::apub::make_apub_endpoint;
use crate::convert_datetime;
use crate::db::post::Post;
use activitystreams::{context, object::apub::Page, object::properties::ObjectProperties};
use failure::Error;

impl Post {
  pub fn as_page(&self) -> Result<Page, Error> {
    let base_url = make_apub_endpoint("post", self.id);
    let mut page = Page::default();
    let oprops: &mut ObjectProperties = page.as_mut();

    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(base_url)?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint("u", &self.creator_id))?;

    if let Some(body) = &self.body {
      oprops.set_content_xsd_string(body.to_owned())?;
    }

    if let Some(url) = &self.url {
      oprops.set_url_xsd_any_uri(url.to_owned())?;
    }

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(page)
  }
}

// TODO: need to serve this via actix
