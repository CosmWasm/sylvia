use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Error, Ident, MetaList, Result, Token};

/// Type wrapping data parsed from `sv::data` attribute.
#[derive(Default, Debug)]
pub struct DataFieldParams {
    pub raw: bool,
    pub opt: bool,
}

impl DataFieldParams {
    pub fn new(attr: &MetaList) -> Result<Self> {
        DataFieldParams::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })
    }
}

impl Parse for DataFieldParams {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut data = Self::default();

        while !input.is_empty() {
            let option: Ident = input.parse()?;
            match option.to_string().as_str() {
                "raw" => data.raw = true,
                "opt" => data.opt = true,
                _ => {
                    return Err(Error::new(
                        option.span(),
                        "Invalid data parameter.\n
  = note: Expected one of [`raw`, `opt`] comma separated.\n",
                    ))
                }
            }
            if !input.peek(Token![,]) {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(data)
    }
}
