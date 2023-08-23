use id_newtype::IdNewtype;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemStruct};

extern crate proc_macro;

mod id_newtype;

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
    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, serde::Serialize, serde::Deserialize)]
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
      impl fmt::Display for #ident {
         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "{}", self.0)
         }
       }
    }
  } else {
    quote!(#newtype)
  })
  .into()
}
