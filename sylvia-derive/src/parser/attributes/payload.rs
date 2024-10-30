use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Error, Ident, MetaList, Result};

/// Type wrapping data parsed from `sv::payload` attribute.
#[derive(Default, Debug)]
pub struct PayloadFieldParam;

impl PayloadFieldParam {
    pub fn new(attr: &MetaList) -> Result<Self> {
        let data = PayloadFieldParam::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })?;

        Ok(data)
    }
}

impl Parse for PayloadFieldParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let option: Ident = input.parse()?;
        match option.to_string().as_str() {
            "raw" => (),
            _ => {
                return Err(Error::new(
                    option.span(),
                    "Invalid payload parameter.\n= note: Expected [`raw`].\n",
                ))
            }
        };

        if !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "Unexpected tokens inside `sv::payload` attribute.\n= note: Expected parameters: [`raw`] `.\n",
            ));
        }

        Ok(Self)
    }
}
