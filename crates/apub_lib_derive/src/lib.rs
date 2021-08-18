use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields::Unnamed, Ident, Variant};

/// Generates implementation ActivityHandler for an enum, which looks like the following (handling
/// all enum variants).
///
/// Based on this code:
/// ```ignore
/// #[derive(serde::Deserialize, serde::Serialize, ActivityHandler)]
/// #[serde(untagged)]
/// pub enum PersonInboxActivities {
///  CreateNote(CreateNote),
///  UpdateNote(UpdateNote),
/// ```
/// It will generate this:
/// ```ignore
/// impl ActivityHandler for PersonInboxActivities {
///
///     async fn verify(
///     &self,
///     context: &LemmyContext,
///     request_counter: &mut i32,
///   ) -> Result<(), LemmyError> {
///     match self {
///       PersonInboxActivities::CreateNote(a) => a.verify(context, request_counter).await,
///       PersonInboxActivities::UpdateNote(a) => a.verify(context, request_counter).await,
///     }
///   }
///
///   async fn receive(
///   &self,
///   context: &LemmyContext,
///   request_counter: &mut i32,
/// ) -> Result<(), LemmyError> {
///     match self {
///       PersonInboxActivities::CreateNote(a) => a.receive(context, request_counter).await,
///       PersonInboxActivities::UpdateNote(a) => a.receive(context, request_counter).await,
///     }
///   }
/// fn common(&self) -> &ActivityCommonFields  {
///     match self {
///       PersonInboxActivities::CreateNote(a) => a.common(),
///       PersonInboxActivities::UpdateNote(a) => a.common(),
///     }
///   }
///
/// ```
#[proc_macro_derive(ActivityHandler)]
pub fn derive_activity_handler(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let enum_name = input.ident;

  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let enum_variants = if let Data::Enum(d) = input.data {
    d.variants
  } else {
    unimplemented!()
  };

  let body_verify = quote! {a.verify(context, request_counter).await};
  let impl_verify = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &body_verify));
  let body_receive = quote! {a.receive(context, request_counter).await};
  let impl_receive = enum_variants
    .iter()
    .map(|v| generate_match_arm(&enum_name, v, &body_receive));

  let expanded = quote! {
      #[async_trait::async_trait(?Send)]
      impl #impl_generics lemmy_apub_lib::ActivityHandler for #enum_name #ty_generics #where_clause {
          async fn verify(
              &self,
              context: &lemmy_websocket::LemmyContext,
              request_counter: &mut i32,
            ) -> Result<(), lemmy_utils::LemmyError> {
            match self {
              #(#impl_verify)*
            }
          }
          async fn receive(
            self,
            context: &lemmy_websocket::LemmyContext,
            request_counter: &mut i32,
          ) -> Result<(), lemmy_utils::LemmyError> {
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

#[proc_macro_derive(ActivityFields)]
pub fn derive_activity_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let name = input.ident;

  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let expanded = match input.data {
    Data::Enum(e) => {
      let variants = e.variants;
      let impl_id = variants
        .iter()
        .map(|v| generate_match_arm(&name, v, &quote! {a.id_unchecked()}));
      let impl_actor = variants
        .iter()
        .map(|v| generate_match_arm(&name, v, &quote! {a.actor()}));
      let impl_cc = variants
        .iter()
        .map(|v| generate_match_arm(&name, v, &quote! {a.cc()}));
      quote! {
          impl #impl_generics lemmy_apub_lib::ActivityFields for #name #ty_generics #where_clause {
              fn id_unchecked(&self) -> &url::Url { match self { #(#impl_id)* } }
              fn actor(&self) -> &url::Url { match self { #(#impl_actor)* } }
              fn cc(&self) -> Vec<url::Url> { match self { #(#impl_cc)* } }
          }
      }
    }
    Data::Struct(s) => {
      // check if the struct has a field "cc", and generate impl for cc() function depending on that
      let has_cc = if let syn::Fields::Named(n) = s.fields {
        n.named
          .iter()
          .any(|i| format!("{}", i.ident.as_ref().unwrap()) == "cc")
      } else {
        unimplemented!()
      };
      let cc_impl = if has_cc {
        quote! {self.cc.clone().into()}
      } else {
        quote! {vec![]}
      };
      quote! {
          impl #impl_generics lemmy_apub_lib::ActivityFields for #name #ty_generics #where_clause {
              fn id_unchecked(&self) -> &url::Url { &self.id }
              fn actor(&self) -> &url::Url { &self.actor }
              fn cc(&self) -> Vec<url::Url> { #cc_impl }
          }
      }
    }
    _ => unimplemented!(),
  };
  expanded.into()
}
