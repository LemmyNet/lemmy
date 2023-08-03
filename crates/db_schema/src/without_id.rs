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
                $(#[$_field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty,
            )*
        }
    ) => {
        ::paste::paste! {
            #[derive(::diesel::Queryable, ::diesel::Selectable)]
            #[diesel(table_name = $table_name)]
            $vis struct [<$struct_name WithoutId>] {
                $(
                    // Field attributes are not kept because either they are for other
                    // derive macros, or they are `#[cfg(...)]` which is evaluated before
                    // macro expansion.
                    $field_vis $field_name : $field_type,
                )*
            }

            impl [<$struct_name WithoutId>] {
                pub fn into_full(self, id: $id_type) -> $struct_name {
                    $struct_name {
                        $($field_name : self.$field_name,)*
                        id,
                    }
                }
            }
        }
    };

    // Keep on removing the first attribute until `diesel(table_name = ...)` becomes
    // the first, which will cause the first pattern to be matched.
    (#[$_meta:meta] $($remaining:tt)*) => {
        $($remaining)*
    };

    // This pattern is matched when there's no attributes.
    ($_vis:vis struct $($_tt:tt)*) => {
        ::std::compile_error!("`#[diesel(table_name = ...)]` is missing");
    };
}
