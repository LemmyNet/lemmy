use crate::apub::make_apub_endpoint;
use crate::convert_datetime;
use crate::db::post_view::PostView;
use activitystreams::{object::apub::Page, object::properties::ObjectProperties};
use failure::Error;

impl PostView {
  pub fn as_page(&self) -> Result<Page, Error> {
    let base_url = make_apub_endpoint("post", self.id);
    let mut page = Page::default();
    let oprops: &mut ObjectProperties = page.as_mut();

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      //.set_context_xsd_any_uri(context())?
      .set_id(base_url)?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint("u", &self.creator_id))?;

    if let Some(body) = &self.body {
      oprops.set_content_xsd_string(body.to_owned())?;
    }

    // TODO: hacky code because we get self.url == Some("")
    let url = self.url.as_ref();
    if url.is_some() && !url.unwrap().is_empty() {
      oprops.set_url_xsd_any_uri(url.unwrap().to_owned())?;
    }

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(page)
  }
}

// TODO: need to serve this via actix
