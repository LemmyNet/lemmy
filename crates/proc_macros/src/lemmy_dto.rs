use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
  parse::{Parse, ParseStream, Result},
  punctuated::Punctuated,
  Ident,
  Token,
};

pub struct DtoDerives(pub Vec<TokenStream>);

impl Parse for DtoDerives {
  fn parse(input: ParseStream) -> Result<Self> {
    let options = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

    Ok(DtoDerives(
      options
        .into_iter()
        .fold(Vec::new(), |mut acc, option| {
          if acc.contains(&option) {
            panic!("Cannot pass duplicate {option} to lemmy_dto!")
          }

          acc.push(option);

          acc
        })
        .into_iter()
        .map(ToTokens::into_token_stream)
        .collect(),
    ))
  }
}
