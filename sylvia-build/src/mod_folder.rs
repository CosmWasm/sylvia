use proc_macro2::Span;
use std::path::{Path, PathBuf};
use syn::fold::Fold;
use syn::parse::{Nothing, ParseStream, Parser};
use syn::token::{Brace, Impl};
use syn::{parse_quote, Ident, Item, ItemImpl, ItemMod, LitStr, Token};

/// Wrapper struct keeping absolute names of sylvia attributes and sylvia crate itself
struct Paths {
    sylvia: syn::Path,
    contract: syn::Path,
    interface: syn::Path,
}

impl Paths {
    fn new(sylvia_crate: &str) -> Self {
        use proc_macro_crate::{crate_name, FoundCrate};

        let sylvia = match crate_name(sylvia_crate).expect("silvia is not found in Cargo.toml") {
            FoundCrate::Itself => parse_quote!(sylvia),
            FoundCrate::Name(name) => {
                let name = Ident::new(&name, Span::call_site());
                parse_quote!(#name)
            }
        };

        Self {
            contract: parse_quote!(#sylvia::contract),
            interface: parse_quote!(#sylvia::interface),
            sylvia,
        }
    }

    fn is_sylvia_attribute(&self, path: &syn::Path) -> bool {
        [&self.contract, &self.interface].contains(&path)
    }
}

/// Utility keeping alive aliases for sylvia attribtues basing on `use` cluases
#[derive(Default)]
struct Aliases {
    sylvia: Vec<syn::Path>,
    contract: Vec<syn::Path>,
    interface: Vec<syn::Path>,
}

impl Aliases {
    fn new() -> Self {
        Self::default()
    }

    fn is_sylvia_attribute(&self, path: &syn::Path) -> bool {
        [&self.contract, &self.interface]
            .into_iter()
            .any(|paths| paths.contains(path))
    }
}

/// The `Fold` type going through all the modules in syntax tree and tries to load its content from
/// a file merging all the module tree in single AST.
///
/// Additionally it would tract any uses and renames of the `sylvia` attributes, to properly figure
/// out which of them are sylvia attributes and which are from other crates. It would use this
/// information to get rid of all the `impl` block not marked by `contract` or `interface`
/// attribute.
pub struct ModFolder<'p> {
    path: &'p Path,

    sylvia_paths: Paths,
    aliases: Aliases,
}

impl<'p> Fold for ModFolder<'p> {
    fn fold_item_mod(&mut self, i: ItemMod) -> ItemMod {
        let i = self.load_module(i);

        syn::fold::fold_item_mod(self, i)
    }
}

impl<'p> ModFolder<'p> {
    pub fn new(path: &'p Path) -> Self {
        ModFolder {
            path,
            sylvia_paths: Paths::new("sylvia"),
            aliases: Aliases::new(),
        }
    }

    fn parse_path_attr(tokens: ParseStream) -> Result<PathBuf, syn::parse::Error> {
        let _: Token![=] = tokens.parse()?;
        let path: LitStr = tokens.parse()?;
        let _: Nothing = tokens.parse()?;

        Ok(PathBuf::from(path.value()))
    }

    fn fold_from_path(path: &Path) -> Result<Vec<Item>, syn::parse::Error> {
        let file = std::fs::read_to_string(path).unwrap();
        let file = syn::parse_file(&file)?;
        let stem = path.file_stem().unwrap();
        let path = path.parent().unwrap().join(stem);

        let mut folder = ModFolder::new(&path);

        let items = file
            .items
            .into_iter()
            .map(|item| folder.fold_item(item))
            .collect();

        Ok(items)
    }

    fn load_module(&self, item: ItemMod) -> ItemMod {
        if item.semi.is_some() {
            let path = item
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("path"))
                .map(|attr| ModFolder::parse_path_attr.parse2(attr.tokens.clone()))
                .transpose();

            // Path unparseable but exits - failing silently by ignoring the module, will be
            // hanlded by the compiler
            let path = match path {
                Ok(path) => path,
                Err(_) => return item,
            };

            let fallback_paths = [
                self.path.join(format!("{}.rs", item.ident)),
                self.path.join(format!("{}/mod.rs", item.ident)),
            ];

            let path = path.or_else(move || fallback_paths.into_iter().find(|path| path.exists()));

            // No valid path - failing silently by ignoring the module, will be handled by the
            // compiler
            let path = match path {
                Some(path) => path,
                None => return item,
            };

            // Module cannot be build for some reason - for now ignoring the module silently
            let content = match Self::fold_from_path(&path) {
                Ok(items) => items,
                Err(_) => return item,
            };
            let brace = Brace::default();

            ItemMod {
                content: Some((brace, content)),
                semi: None,
                ..item
            }
        } else {
            item
        }
    }

    fn is_sylvia_attribute(&self, path: &syn::Path) -> bool {
        self.sylvia_paths.is_sylvia_attribute(path) || self.aliases.is_sylvia_attribute(path)
    }

    fn process_impl_items(&mut self, items: Vec<syn::Item>) -> Vec<syn::Item> {
        items
            .into_iter()
            .filter(|item| {
                if let Item::Impl(ItemImpl { attrs, .. }) = &item {
                    attrs
                        .iter()
                        .any(|attr| self.is_sylvia_attribute(&attr.path))
                } else {
                    false
                }
            })
            .collect()
    }
}
