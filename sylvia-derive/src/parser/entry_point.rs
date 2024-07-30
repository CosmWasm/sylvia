use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::emit_error;
use syn::parse::{Error, Nothing, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse2, GenericArgument, Path, Result, Token};

use super::extract_generics_from_path;

/// Parsed arguments for `entry_points` macro
#[derive(Default)]
pub struct EntryPointArgs {
    /// Types used in place of contracts generics.
    pub generics: Punctuated<GenericArgument, Token![,]>,
}

impl EntryPointArgs {
    pub fn new(attr: &TokenStream2) -> Result<Self> {
        let args: Self = parse2(attr.clone()).map_err(|err| {
            emit_error!(attr, err);
            err
        })?;

        Ok(args)
    }
}

impl Parse for EntryPointArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut entry_points_args = Self::default();
        if input.is_empty() {
            return Ok(entry_points_args);
        }

        let generics: Path = input.parse()?;
        match generics.segments.last() {
            Some(segment) if segment.ident == "generics" => {
                entry_points_args.generics = extract_generics_from_path(&generics)
            }
            _ => return Err(Error::new(generics.span(), "Expected `generics`.")),
        };

        let _: Nothing = input.parse()?;

        Ok(entry_points_args)
    }
}
