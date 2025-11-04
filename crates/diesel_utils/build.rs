use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let migrations_dir = Path::new("../../migrations/");
  if !migrations_dir.exists() {
    return Err("Migrations dir not found".into());
  }
  println!("cargo:rerun-if-changed={}", migrations_dir.display());
  Ok(())
}
