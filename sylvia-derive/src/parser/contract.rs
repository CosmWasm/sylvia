use syn::parse::{Error, Nothing, Parse, ParseStream};
use syn::{Ident, Path, Result, Token};

/// Parsed arguments for `contract` macro
pub struct ContractArgs {
    /// Module in which contract impl block is defined.
    /// Used only while implementing `Interface` on `Contract`.
    pub module: Option<Path>,
}

impl Parse for ContractArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut module = None;

        while !input.is_empty() {
            let attr: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            if attr == "module" {
                module = Some(input.parse()?);
            } else {
                return Err(Error::new(attr.span(), "expected `module`"));
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else if !input.is_empty() {
                return Err(input.error("Unexpected token, comma expected"));
            }
        }

        let _: Nothing = input.parse()?;

        Ok(ContractArgs { module })
    }
}
