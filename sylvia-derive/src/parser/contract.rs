use proc_macro_error::abort;
use syn::parse::{Error, Nothing, Parse, ParseStream};
use syn::{Ident, Path, Result, Token};

/// Parsed arguments for `contract` macro
pub struct ContractArgs {
    /// Module in which contract impl block is defined.
    /// Used only while implementing `Interface` on `Contract`.
    pub module: Path,
}

impl Parse for ContractArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_module = input.parse().and_then(|attr: Ident| -> Result<Path> {
            let _: Token![=] = input.parse()?;
            if attr == "module" {
                input.parse()
            } else {
                Err(Error::new(attr.span(), "Missing `module` attribute"))
            }
        });
        let module: Path = match maybe_module {
            Ok(module) => module,
            Err(e) => abort!(
                e.span(), "The module path needs to be provided `#[contract(module=path::to::contract)`.";
                note = "Implementing interface on a contract requires to point the path to the contract structure.";
                note = "Parsing error: {}", e
            ),
        };
        let _: Nothing = input.parse()?;
        Ok(ContractArgs { module })
    }
}
