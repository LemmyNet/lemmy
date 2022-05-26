use serde::{Deserialize, Deserializer};

/// Deserialize either a single json value, or a json array. In either case, the items are returned
/// as an array.
///
/// Usage:
/// `#[serde(deserialize_with = "deserialize_one_or_many")]`
pub fn deserialize_one_or_many<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
  T: Deserialize<'de>,
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
  }

  let result: OneOrMany<T> = Deserialize::deserialize(deserializer)?;
  Ok(match result {
    OneOrMany::Many(list) => list,
    OneOrMany::One(value) => vec![value],
  })
}

/// Deserialize either a single json value, or a json array with one element. In both cases it
/// returns a single value.
///
/// Usage:
/// `#[serde(deserialize_with = "deserialize_one")]`
pub fn deserialize_one<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: Deserialize<'de>,
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum MaybeArray<T> {
    Simple(T),
    Array([T; 1]),
  }

  let result: MaybeArray<T> = Deserialize::deserialize(deserializer)?;
  Ok(match result {
    MaybeArray::Simple(value) => value,
    MaybeArray::Array([value]) => value,
  })
}

/// Attempts to deserialize the item. If any error happens, its ignored and the type's default
/// value is returned.
///
/// Usage:
/// `#[serde(deserialize_with = "deserialize_skip_error")]`
pub fn deserialize_skip_error<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: Deserialize<'de> + Default,
  D: Deserializer<'de>,
{
  let result = Deserialize::deserialize(deserializer);
  Ok(match result {
    Ok(o) => o,
    Err(_) => Default::default(),
  })
}
