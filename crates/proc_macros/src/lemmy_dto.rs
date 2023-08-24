use syn::{
  parse::{Parse, ParseStream, Result},
  punctuated::Punctuated,
  Ident,
  Token,
};

pub struct DtoOptions {
  pub default: bool,
  pub skip_none: bool,
}

impl Parse for DtoOptions {
  fn parse(input: ParseStream) -> Result<Self> {
    let options = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

    if options.len() > 2 {
      panic!("lemmy_dto cannot take more than 2 arguments!");
    }

    let mut used_options = Vec::new();

    Ok(options.into_iter().fold(
      DtoOptions {
        default: false,
        skip_none: false,
      },
      |mut acc, option| {
        if used_options.contains(&option) {
          panic!("Cannot pass duplicate option: {}", option);
        }

        match option.to_string().as_str() {
          "default" => acc.default = true,
          "skip_none" => acc.skip_none = true,
          o @ _ => panic!("lemmy_dto recieved invalid option: {}", o),
        };

        used_options.push(option);

        acc
      },
    ))
  }
}
