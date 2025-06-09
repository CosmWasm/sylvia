use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Error, Ident, MetaList, Result};

/// Type wrapping data parsed from `sv::features` attribute.
#[derive(Debug, Default)]
pub struct SylviaFeatures {}

impl SylviaFeatures {
    pub fn new(attr: &MetaList) -> Result<Self> {
        SylviaFeatures::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })
    }
}

impl Parse for SylviaFeatures {
    fn parse(input: ParseStream) -> Result<Self> {
        let features = Self::default();

        if !input.is_empty() {
            let feature: Ident = input.parse()?;
            return Err(Error::new(
                feature.span(),
                "Invalid feature.\n= note: No features supported currently.\n",
            ));
        }

        Ok(features)
    }
}
