use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine};
use captcha::Captcha;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::slurs::check_slurs,
};
use regex::Regex;
use std::io::Cursor;

pub mod comment;
pub mod community;
pub mod local_user;
pub mod post;
pub mod private_message;
pub mod reports;
pub mod site;
pub mod sitemap;

/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> LemmyResult<String> {
  let letters = captcha.as_wav();

  // Decode each wav file, concatenate the samples
  let mut concat_samples: Vec<i16> = Vec::new();
  let mut any_header: Option<hound::WavSpec> = None;
  for letter in letters {
    let mut cursor = Cursor::new(letter.unwrap_or_default());
    let reader = hound::WavReader::new(&mut cursor)?;
    any_header = Some(reader.spec());
    let samples16 = reader
      .into_samples::<i16>()
      .collect::<Result<Vec<_>, _>>()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    concat_samples.extend(samples16);
  }

  // Encode the concatenated result as a wav file
  let mut output_buffer = Cursor::new(vec![]);
  if let Some(header) = any_header {
    let mut writer = hound::WavWriter::new(&mut output_buffer, header)
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    let mut writer16 = writer.get_i16_writer(concat_samples.len() as u32);
    for sample in concat_samples {
      writer16.write_sample(sample);
    }
    writer16
      .flush()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    writer
      .finalize()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;

    Ok(base64.encode(output_buffer.into_inner()))
  } else {
    Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
  }
}

/// Check size of report
pub(crate) fn check_report_reason(reason: &str, slur_regex: &Regex) -> LemmyResult<()> {
  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(LemmyErrorType::ReportReasonRequired)?
  } else if reason.chars().count() > 1000 {
    Err(LemmyErrorType::ReportTooLong)?
  } else {
    Ok(())
  }
}
