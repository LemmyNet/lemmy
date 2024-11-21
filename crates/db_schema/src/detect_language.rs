use crate::{newtypes::LanguageId, source::language::Language, utils::DbPool};
use lemmy_utils::error::LemmyResult;
use lingua::{IsoCode639_1, Language as LinguaLang, LanguageDetectorBuilder};

pub async fn detect_language(input: &str, pool: &mut DbPool<'_>) -> LemmyResult<LanguageId> {
  // TODO: should only detect languages which are allowed in community
  // TODO: cache conversion table Lingua to LanguageId and reverse (maybe load it directly from
  // migration sql)
  // TODO: instead of at post creation, could also run this as a background task
  // TODO: probably uses a lot of ram/cpu, need to make it configurable:
  //       - analyze only local posts or all posts
  //       - low accuracy or high accuracy setting
  //       - min confidence value
  //
  // >>>> This should be a plugin!
  let detector = LanguageDetectorBuilder::from_iso_codes_639_1(&[
    IsoCode639_1::EN,
    IsoCode639_1::ES,
    IsoCode639_1::DE,
  ])
  .build();

  let lang: Option<LinguaLang> = detector.detect_language_of(input);
  let Some(lang) = lang else {
    return Ok(LanguageId(0));
  };
  let confidence = detector.compute_language_confidence("languages are awesome", lang);
  let lang = lang.iso_code_639_1().to_string().to_lowercase();
  dbg!(&lang, &confidence);
  if confidence < 0.4 {
    return Ok(LanguageId(0));
  }

  Ok(Language::read_id_from_code(pool, &lang).await?)
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::utils::build_db_pool_for_tests;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_detect_language() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // some easy comments
    assert_eq!(
      LanguageId(37),
      detect_language(
        "I don't think it's supposed to be taken seriously. It's just a throwaway meme.
",
        pool
      )
      .await?
    );
    assert_eq!(
      LanguageId(39),
      detect_language(
        "Oh! Mencion casual de la mejor pelicula navideña… Die hard!
",
        pool
      )
      .await?
    );
    assert_eq!(
      LanguageId(32),
      detect_language(
        "Die Forderung finde ich nutzlos.
",
        pool
      )
      .await?
    );

    // different languages
    assert_eq!(
      LanguageId(0),
      detect_language(
        "Die Forderung finde ich nutzlos. It's just a throwaway meme.
",
        pool
      )
      .await?
    );
    Ok(())
  }
}
