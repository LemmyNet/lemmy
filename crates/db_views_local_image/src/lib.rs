#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local image view.
pub struct LocalImageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_image: LocalImage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person: Person,
}
