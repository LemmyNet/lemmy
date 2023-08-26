use std::string::ToString;
use syn::{parse::Parse, punctuated::Punctuated, Ident, Token};

pub struct IdNewtype {
  pub ident: Ident,
  pub public: bool,
  pub impl_display: bool,
  pub ts: bool,
}

impl Parse for IdNewtype {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut tokens = Punctuated::<Ident, Token![+]>::parse_terminated(input)?
      .into_iter()
      .rev()
      .collect::<Vec<_>>();

    let ident;

    if let Some(name) = tokens.pop() {
      ident = name;
    } else {
      panic!("Macro must be passed struct name with plus symbols adding flags, e.g. id_newtype!(MyIdNewtype + public + ts)");
    }

    let mut newtype = IdNewtype {
      ident,
      impl_display: false,
      public: false,
      ts: false,
    };

    if !tokens.is_empty() {
      match tokens.len() {
        1..=3 => {
          let mut used_flags = Vec::new();

          for flag in tokens.iter().map(ToString::to_string) {
            if used_flags.contains(&flag) {
              panic!("Cannot pass same flag more than once. Duplicated flag: {flag}");
            }

            match flag.as_str() {
              "ts" => newtype.ts = true,
              "public" => newtype.public = true,
              "display" => newtype.impl_display = true,
              _ => panic!("Invalid flag: {flag}"),
            }

            used_flags.push(flag);
          }
        }
        _ => {
          panic!("Cannot pass more than 3 flags to macro");
        }
      }
    }

    Ok(newtype)
  }
}
