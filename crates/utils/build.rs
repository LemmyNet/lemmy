// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

fn main() -> Result<(), Box<dyn std::error::Error>> {
  rosetta_build::config()
    .source("en", "translations/email/en.json")
    .source("fi", "translations/email/fi.json")
    .source("ko", "translations/email/ko.json")
    .source("pt", "translations/email/pt.json")
    .fallback("en")
    .generate()?;

  Ok(())
}
