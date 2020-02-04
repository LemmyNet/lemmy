use crate::apub::make_apub_endpoint;
use crate::db::post::Post;
use crate::to_datetime_utc;
use activitypub::{context, object::Page};

impl Post {
  pub fn as_page(&self) -> Page {
    let base_url = make_apub_endpoint("post", self.id);
    let mut page = Page::default();

    page.object_props.set_context_object(context()).ok();
    page.object_props.set_id_string(base_url).ok();
    page.object_props.set_name_string(self.name.to_owned()).ok();

    if let Some(body) = &self.body {
      page.object_props.set_content_string(body.to_owned()).ok();
    }

    if let Some(url) = &self.url {
      page.object_props.set_url_string(url.to_owned()).ok();
    }

    //page.object_props.set_attributed_to_string

    page
      .object_props
      .set_published_utctime(to_datetime_utc(self.published))
      .ok();
    if let Some(updated) = self.updated {
      page
        .object_props
        .set_updated_utctime(to_datetime_utc(updated))
        .ok();
    }

    page
  }
}
