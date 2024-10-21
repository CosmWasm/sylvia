use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Error, Ident, MetaList, Result, Token};

/// Type wrapping data parsed from `sv::data` attribute.
#[derive(Default, Debug)]
pub struct DataFieldParams {
    pub raw: bool,
    pub opt: bool,
    pub instantiate: bool,
}

impl DataFieldParams {
    pub fn new(attr: &MetaList) -> Result<Self> {
        let data = DataFieldParams::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })?;

        if data.instantiate && data.raw {
            emit_error!(
                attr.tokens.span(),
                "The `instantiate` cannot be used in pair with `raw` parameter.";
                note = "Use any combination of [`raw`, `opt`] or [`instantiate`, `opt`] pairs."
            );
        }

        Ok(data)
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
                "instantiate" => data.instantiate = true,
                _ => {
                    return Err(Error::new(
                        option.span(),
                        "Invalid data parameter.\n
  = note: Expected one of [`raw`, `opt`, `instantiate`] comma separated.\n",
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
