/// `macro_rules_attribute::derive(WithoutId!)` generates a variant of the struct with
/// `WithoutId` added to the name and no `id` field.
///
/// This is useful for making less redundant selections of multiple joined tables.
/// For example, selecting both `comment::post_id` and `post::id` is redundant if they
/// have the same value. In this case, the selection of `post::id` can be avoided by selecting
/// `PostWithoutId::as_select()` instead of `post::all_columns`.
///
/// This macro generates an `into_full` method, which converts to the sturct with `id`.
/// For example, `PostWithoutId` would have this:
///
/// `pub fn into_full(self, id: PostId) -> Post`
///
/// The `id` value can come from a column in another table, like `comment::post_id`.
///
/// The generated struct implements `Selectable` and `Queryable`.
macro_rules! WithoutId {
  (
    #[diesel(table_name = $table_name:ident)]
    $(#[$_struct_meta:meta])*
    $vis:vis struct $struct_name:ident {
      $(#[$_id_meta:meta])*
      $_id_vis:vis id: $id_type:ty,
      $(
        // TODO: more flexible and clean attribute matching
        $(#[doc = $_doc1:tt])*
        $(#[cfg($($cfgtt:tt)*)])*
        $(#[cfg_attr($($cfgattrtt:tt)*)] $(#[doc = $_doc2:tt])*)*
        $(#[serde($($serdett:tt)*)])*
        $field_vis:vis $field_name:ident : $field_type:ty,
      )*
    }
  ) => {
    ::paste::paste! {
      // TODO: remove serde derives
      #[derive(::diesel::Queryable, ::diesel::Selectable, ::serde::Serialize, ::serde::Deserialize)]
      #[diesel(table_name = $table_name)]
      $vis struct [<$struct_name WithoutId>] {
        $(
          $(#[cfg($($cfgtt)*)])*
          $field_vis $field_name : $field_type,
        )*
      }

      impl [<$struct_name WithoutId>] {
        pub fn into_full(self, id: $id_type) -> $struct_name {
          $struct_name {
            id,
            $($(#[cfg($($cfgtt)*)])* $field_name : self.$field_name,)*
          }
        }
      }
    }
  };

  // Keep on removing the first attribute until `diesel(table_name = ...)` becomes
  // the first, which will cause the first pattern to be matched.
  (#[$_meta:meta] $($remaining:tt)*) => {
    WithoutId!($($remaining)*);
  };

  // This pattern is matched when there's no attributes.
  ($_vis:vis struct $($_tt:tt)*) => {
    ::std::compile_error!("`#[diesel(table_name = ...)]` is missing");
  };
}
