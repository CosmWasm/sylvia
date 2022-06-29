//! Sylvia utlities to be used by buildscript to generate code

use cargo_toml::Manifest;
use std::env;
use std::path::{Path, PathBuf};
use syn::fold::Fold;
use syn::parse::{Nothing, ParseStream, Parser};
use syn::{Item, ItemMod, LitStr, Token};

pub struct Build;

impl Build {
    pub fn build(self) {
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
        let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");
        let manifest = Manifest::from_path(&manifest_path).unwrap();

        let lib = manifest.lib.expect("Missing library entry in crate manifest. Smart contract has to be either rlib or cdylib");
        let lib_path = lib
            .path
            .map(PathBuf::from)
            .unwrap_or_else(|| Path::new(&manifest_dir).join("src/lib.rs"));

        let _items = ModFolder::fold_from_path(&lib_path).expect("Error building library AST");

        let _out_dir = env::var_os("OUT_DIR").unwrap();
    }
}

struct ModFolder<'p> {
    path: &'p Path,
}

impl<'p> Fold for ModFolder<'p> {
    fn fold_item_mod(&mut self, mod i: ItemMod) -> ItemMod {
        if i.semi.is_some() {
            let path = i
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("path"))
                .map(|attr| ModFolder::parse_path_attr.parse2(attr.tokens.clone()))
                .transpose();

            // Path unparseable but exits - failing silently by ignoring the module, will be
            // hanlded by the compiler
            let path = match path {
                Ok(path) => path,
                Err(_) => return i,
            };

            let fallback_paths = [
                self.path.join(format!("{}.rs", i.ident)),
                self.path.join(format!("{}/mod.rs", i.ident)),
            ];

            let path = path.or_else(move || fallback_paths.into_iter().find(|path| path.exists()));

            // No valid path - failing silently by ignoring the module, will be handled by the
            // compiler
            let path = match path {
                Some(path) => path,
                None => return i,
            };

            // Load mod from path
            todo!()
        } else {
            self.fold_item_mod(i)
        }
    }
}

impl<'p> ModFolder<'p> {
    fn parse_path_attr(tokens: ParseStream) -> Result<PathBuf, syn::parse::Error> {
        let _: Token![=] = tokens.parse()?;
        let path: LitStr = tokens.parse()?;
        let _: Nothing = tokens.parse()?;

        Ok(PathBuf::from(path.value()))
    }

    fn fold_from_path(path: &Path) -> Result<Vec<Item>, syn::parse::Error> {
        let file = syn::parse_file(path.to_str().unwrap())?;
        let stem = path.file_stem().unwrap();
        let path = path.parent().unwrap().join(stem);

        let mut folder = ModFolder { path: &path };

        let items = file
            .items
            .into_iter()
            .map(|item| folder.fold_item(item))
            .collect();

        Ok(items)
    }
}
