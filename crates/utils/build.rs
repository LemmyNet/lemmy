fn main() -> Result<(), Box<dyn std::error::Error>> {
  rosetta_build::config()
    .source("en", "translations/email/en.json")
    .fallback("en")
    .generate()?;

  Ok(())
}
