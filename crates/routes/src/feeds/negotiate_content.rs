use actix_web::{Error, HttpRequest, http::header::*, web};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{local_user_view_from_jwt, read_auth_token},
};
use lemmy_email::{translations::Lang, user_language};
use rosetta_i18n::{Language, LanguageId};

pub(crate) async fn get_lang_or_negotiate(
  req: &HttpRequest,
  context: &web::Data<LemmyContext>,
) -> Result<Lang, Error> {
  let jwt = read_auth_token(req)?;

  let lang = if let Some(jwt) = jwt {
    let local_user_view = local_user_view_from_jwt(&jwt, context).await?;
    user_language(&local_user_view.local_user)
  } else if req.headers().contains_key(ACCEPT_LANGUAGE) {
    negotiate_lang(req).unwrap_or(Lang::En)
  } else {
    Lang::En
  };
  Ok(lang)
}

fn negotiate_lang(req: &HttpRequest) -> Option<Lang> {
  let client_langs = AcceptLanguage::parse(req).ok()?;

  client_langs.ranked().iter().find_map(|cl| {
    cl.item()
      .map(|l| LanguageId::new(l.primary_language()))
      .and_then(|l| Lang::from_language_id(&l))
  })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
  use super::*;
  use actix_web::test::TestRequest;

  fn parse_lang_items(
    accept_language_header_value: &str,
  ) -> Vec<QualityItem<Preference<LanguageTag>>> {
    accept_language_header_value
      .split(',')
      .map(|s| s.parse().unwrap())
      .collect()
  }

  #[test]
  fn test_negotiate_language_lang_supported_by_server() {
    let req = TestRequest::default()
      .insert_header(AcceptLanguage(parse_lang_items(
        "fj, sm, lo, da, en-GB;q=0.8, en;q=0.7",
      )))
      .to_http_request();

    let resolved_lang = negotiate_lang(&req).unwrap();

    // This test will fail if support for Fijian language is introduced
    // Fix: Remove it and simply move one of the other (rare) languages to the top of the list
    assert_eq!(resolved_lang, Lang::Da);
  }

  #[test]
  fn test_negotiate_language_lang_unsupported_by_server() {
    let req = TestRequest::default()
      .insert_header(AcceptLanguage(parse_lang_items("fj, sm, lo, km")))
      .to_http_request();

    let resolved_lang = negotiate_lang(&req);

    // This test will fail if support for Fijian language is introduced
    // Fix: Remove it and simply move one of the other (rare) languages to the top of the list
    assert!(resolved_lang.is_none());
  }

  #[test]
  fn test_negotiate_language_wildcard_alone() {
    let req = TestRequest::default()
      .insert_header(AcceptLanguage(parse_lang_items("*")))
      .to_http_request();

    let resolved_lang = negotiate_lang(&req);

    assert!(resolved_lang.is_none());
  }

  #[test]
  fn test_negotiate_language_wildcard_with_langs_after() {
    let req = TestRequest::default()
      .insert_header(AcceptLanguage(parse_lang_items("*, fr")))
      .to_http_request();

    let resolved_lang = negotiate_lang(&req);

    assert!(resolved_lang.is_some());
  }
}
