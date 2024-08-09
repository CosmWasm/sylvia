use convert_case::{Case, Casing};
use proc_macro_error::emit_warning;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{parenthesized, Error, Ident, MetaList, Path, Result, Token};

use proc_macro_error::emit_error;

use crate::fold::StripGenerics;

/// Type wrapping data parsed from `sv::message` attribute.
#[derive(Debug)]
pub struct ContractMessageAttr {
    pub module: Path,
    pub variant: Ident,
    pub customs: Customs,
}

impl ContractMessageAttr {
    pub fn new(attr: &MetaList) -> Result<Self> {
        ContractMessageAttr::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })
    }
}

/// The `Customs` struct is used to represent the presence of `: custom(msg, query)` parameter
/// within a `sv::message` attribute.
///
/// It is used to determine if an interface implemented on the contract uses `Empty` in place
/// of custom types.
#[derive(Debug)]
pub struct Customs {
    pub has_msg: bool,
    pub has_query: bool,
}

fn interface_has_custom(content: ParseStream) -> Result<Customs> {
    let mut customs = Customs {
        has_msg: false,
        has_query: false,
    };

    if !content.peek(Token![:]) {
        return Ok(customs);
    }

    let _: Token![:] = content.parse()?;
    let attr: Ident = content.parse()?;
    if attr != "custom" {
        return Ok(customs);
    }

    let custom_content;
    parenthesized!(custom_content in content);

    while !custom_content.is_empty() {
        let custom = custom_content.parse::<Path>()?;
        match custom.get_ident() {
            Some(ident) if ident == "msg" => customs.has_msg = true,
            Some(ident) if ident == "query" => customs.has_query = true,
            _ => {
                return Err(Error::new(
                    custom.span(),
                    "Invalid custom attribute, expected one or both of: [`msg`, `query`]\n
  = note: Expected attribute to be in form `#[sv::messages(interface: custom(msg, query))]`.\n",
                ))
            }
        }
        if !custom_content.peek(Token![,]) {
            break;
        }
        let _: Token![,] = custom_content.parse()?;
    }
    Ok(customs)
}

impl Parse for ContractMessageAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse()?;
        let module = StripGenerics.fold_path(module);

        let variant = if input.parse::<Token![as]>().is_ok() {
            let variant: Ident = input.parse()?;
            if Some(variant.to_string())
                == module
                    .segments
                    .last()
                    .map(|name| name.ident.to_string().to_case(Case::UpperCamel))
            {
                emit_warning!(
                    variant.span(), "Redundant `as {}`.", variant;
                    note = "Interface name is a camel case version of the path and can be auto deduced.";
                    note = "Attribute can be simplified to: `#[sv::messages(interface_path)]`"
                )
            }
            variant
        } else if let Some(module_name) = &module.segments.last() {
            let interface_name = module_name.ident.to_string().to_case(Case::UpperCamel);
            syn::Ident::new(&interface_name, module.span())
        } else {
            Ident::new("", module.span())
        };
        let customs = interface_has_custom(input)?;
        if !input.is_empty() {
            return Err(Error::new(input.span(),
                "Unexpected tokens inside `sv::messages` attribtue.\n
  = note: Maximal supported form of attribute: `#[sv::messages(interface::path as InterfaceName: custom(msg, query))]`.\n"
            ));
        }
        Ok(Self {
            module,
            variant,
            customs,
        })
    }
}
