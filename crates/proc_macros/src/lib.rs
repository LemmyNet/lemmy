use id_newtype::IdNewtype;
use lemmy_dto::DtoDerives;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, parse_macro_input, parse_quote, ItemStruct};

extern crate proc_macro;

mod id_newtype;
mod lemmy_dto;

#[proc_macro]
pub fn id_newtype(input: TokenStream) -> TokenStream {
  let IdNewtype {
    ident,
    impl_display,
    public,
    ts,
  } = parse_macro_input!(input);

  let struct_pub = if public { quote!(pub i32) } else { quote!(i32) };

  let mut newtype: ItemStruct = parse_quote! {
    #[derive(std::fmt::Debug, std::marker::Copy, std::clone::Clone, std::hash::Hash, std::cmp::Eq, std::cmp::PartialEq, std::default::Default, serde::Serialize, serde::Deserialize)]
    pub struct #ident(#struct_pub);
  };

  let mut derives = vec![quote!(DieselNewType)];
  let mut attrs = Vec::new();

  if ts {
    derives.push(quote!(ts_rs::TS));
    attrs.push(parse_quote!(#[cfg_attr(feature = "full", ts(export))]));
  }

  attrs.insert(
    0,
    parse_quote!(#[cfg_attr(feature = "full", derive(#(#derives),*))]),
  );

  newtype.attrs.append(&mut attrs);

  (if impl_display {
    quote! {
      #newtype
      impl std::fmt::Display for #ident {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           write!(f, "{}", self.0)
         }
       }
    }
  } else {
    quote!(#newtype)
  })
  .into()
}

#[proc_macro_attribute]
pub fn lemmy_dto(args: TokenStream, item: TokenStream) -> TokenStream {
  let DtoDerives(derives) = parse_macro_input!(args);
  let mut item =
    parse::<ItemStruct>(item).expect("lemmy_dto attribute can only be applied to structs.");

  item
    .attrs
    .push(parse_quote!(#[serde_with::skip_serializing_none]));

  let mut derive_attrs = vec![
    parse_quote!(std::fmt::Debug),
    parse_quote!(std::clone::Clone),
    parse_quote!(serde::Deserialize),
    parse_quote!(serde::Serialize),
  ];
  derive_attrs.extend(derives);

  item.attrs.push(parse_quote!(#[derive(#(#derive_attrs),*)]));
  item
    .attrs
    .push(parse_quote!(#[cfg_attr(feature = "full", derive(ts_rs::TS))]));
  item
    .attrs
    .push(parse_quote!(#[cfg_attr(feature = "full", ts(export))]));

  item.into_token_stream().into()
}
