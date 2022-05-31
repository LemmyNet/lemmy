use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields::Unnamed, Ident, Variant};

/// Generates implementation ActivityHandler for an enum, which looks like the following (handling
/// all enum variants).
///
/// Based on this code:
/// ```ignore
/// #[derive(serde::Deserialize, serde::Serialize)]
/// #[serde(untagged)]
/// #[activity_handler(LemmyContext, LemmyError)]
/// pub enum PersonInboxActivities {
///  CreateNote(CreateNote),
///  UpdateNote(UpdateNote),
/// }
/// ```
/// It will generate this:
/// ```ignore
/// impl ActivityHandler for PersonInboxActivities {
///     type DataType = LemmyContext;
///     type Error = LemmyError;
///
///     async fn verify(
///     &self,
///     data: &Self::DataType,
///     request_counter: &mut i32,
///   ) -> Result<(), Self::Error> {
///     match self {
///       PersonInboxActivities::CreateNote(a) => a.verify(data, request_counter).await,
///       PersonInboxActivities::UpdateNote(a) => a.verify(context, request_counter).await,
///     }
///   }
///
///   async fn receive(
///   &self,
///   data: &Self::DataType,
///   request_counter: &mut i32,
/// ) -> Result<(), Self::Error> {
///     match self {
///       PersonInboxActivities::CreateNote(a) => a.receive(data, request_counter).await,
///       PersonInboxActivities::UpdateNote(a) => a.receive(data, request_counter).await,
///     }
///   }
/// ```
#[proc_macro_attribute]
pub fn activity_handler(
  attr: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let derive_input = parse_macro_input!(input as DeriveInput);
  let derive_input2 = derive_input.clone();
  let attr = proc_macro2::TokenStream::from(attr);
  let mut attr = attr.into_iter();
  let data_type = attr.next().unwrap();
  let _delimiter = attr.next();
  let error = attr.next().unwrap();

  let enum_name = derive_input2.ident;

  let (impl_generics, ty_generics, where_clause) = derive_input2.generics.split_for_impl();

  let enum_variants = if let Data::Enum(d) = derive_input2.data {
    d.variants
  } else {
    unimplemented!()
  };

  let impl_id = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &quote! {a.id()}));
  let impl_actor = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &quote! {a.actor()}));
  let body_verify = quote! {a.verify(context, request_counter).await};
  let impl_verify = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &body_verify));
  let body_receive = quote! {a.receive(context, request_counter).await};
  let impl_receive = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &body_receive));

  let expanded = quote! {
      #derive_input
      #[async_trait::async_trait(?Send)]
      impl #impl_generics activitypub_federation::traits::ActivityHandler for #enum_name #ty_generics #where_clause {
        type DataType = #data_type;
        type Error = #error;
          fn id(
              &self,
            ) -> &Url {
            match self {
              #(#impl_id)*
            }
          }
          fn actor(
            &self,
          ) -> &Url {
            match self {
              #(#impl_actor)*
            }
          }
          async fn verify(
              &self,
              context: &activitypub_federation::data::Data<Self::DataType>,
              request_counter: &mut i32,
            ) -> Result<(), Self::Error> {
            match self {
              #(#impl_verify)*
            }
          }
          async fn receive(
            self,
            context: &activitypub_federation::data::Data<Self::DataType>,
            request_counter: &mut i32,
          ) -> Result<(), Self::Error> {
            match self {
              #(#impl_receive)*
            }
          }
      }
  };
  expanded.into()
}

fn generate_match_arm(enum_name: &Ident, variant: &Variant, body: &TokenStream) -> TokenStream {
  let id = &variant.ident;
  match &variant.fields {
    Unnamed(_) => {
      quote! {
        #enum_name::#id(a) => #body,
      }
    }
    _ => unimplemented!(),
  }
}
