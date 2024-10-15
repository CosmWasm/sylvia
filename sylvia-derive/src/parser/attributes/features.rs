use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Error, Ident, MetaList, Result, Token};

/// Type wrapping data parsed from `sv::features` attribute.
#[derive(Debug, Default)]
pub struct SylviaFeatures {
    /// Enables better dispatching and deserialization for replies.
    pub replies: bool,
}

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
        let mut features = Self::default();

        while !input.is_empty() {
            let feature: Ident = input.parse()?;
            match feature.to_string().as_str() {
                "replies" => features.replies = true,
                _ => {
                    return Err(Error::new(
                        feature.span(),
                        "Invalid feature.\n= note: Expected `#[sv::features(replies)];`.\n",
                    ))
                }
            }
            if !input.peek(Token![,]) {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(features)
    }
}
