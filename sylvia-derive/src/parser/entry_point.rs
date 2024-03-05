use syn::parse::{Error, Nothing, Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, GenericArgument, Path, Result, Token};

use super::attributes::custom::Custom;
use super::extract_generics_from_path;

/// Parsed arguments for `entry_points` macro
#[derive(Default)]
pub struct EntryPointArgs {
    /// Types used in place of contracts generics.
    pub generics: Option<Punctuated<GenericArgument, Token![,]>>,
    /// Concrete custom msg/query used in place of contracts generic ones.
    pub custom: Option<Custom>,
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
                entry_points_args.generics = Some(extract_generics_from_path(&generics))
            }
            _ => return Err(Error::new(generics.span(), "Expected `generics`.")),
        };

        let comma: Option<Token![,]> = input.parse().ok();
        if comma.is_none() {
            return Ok(entry_points_args);
        }

        let custom: Option<Path> = input.parse().ok();
        match custom {
            Some(custom)
                if custom.get_ident().map(|custom| custom.to_string())
                    == Some("custom".to_owned()) =>
            {
                let content;
                parenthesized!(content in input);
                entry_points_args.custom = Some(Custom::parse.parse2(content.parse()?)?);
            }
            Some(attr) => return Err(Error::new(attr.span(), "Expected `custom`.")),
            _ => (),
        };

        let _: Nothing = input.parse()?;

        Ok(entry_points_args)
    }
}
