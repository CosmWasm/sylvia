//! Sylvia utlities to be used by buildscript to generate code

use cargo_toml::Manifest;
use std::env;
use std::path::{Path, PathBuf};
use syn::fold::Fold;
use syn::parse::{Nothing, ParseStream, Parser};
use syn::token::Brace;
use syn::{Item, ItemMod, LitStr, Token};

pub struct Build;

impl Build {
    pub fn build(self) {
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
        let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");
        let manifest = Manifest::from_path(&manifest_path).unwrap();

        let lib = manifest
            .lib
            .expect("Missing library entry in crate manifest. Smart contract has to be cdylib");
        let lib_path = lib
            .path
            .map(PathBuf::from)
            .unwrap_or_else(|| Path::new(&manifest_dir).join("src/lib.rs"));
        let src_path = lib_path.parent().expect("Source directory doesn't exist");
        let lib_file = std::fs::read_to_string(&lib_path).expect("Cannot load root library file");

        let mut folder = ModFolder { path: src_path };

        let items: Vec<_> = syn::parse_file(&lib_file)
            .expect("Cannot parse AST")
            .items
            .into_iter()
            .map(|item| folder.fold_item(item))
            .collect();

        dbg!(items);
        panic!();

        let _out_dir = env::var_os("OUT_DIR").unwrap();
    }
}

struct ModFolder<'p> {
    path: &'p Path,
}

impl<'p> Fold for ModFolder<'p> {
    fn fold_item_mod(&mut self, i: ItemMod) -> ItemMod {
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

            // Module cannot be build for some reason - for now ignoring the module silently
            let content = match Self::fold_from_path(&path) {
                Ok(items) => items,
                Err(_) => return i,
            };
            let brace = Brace::default();

            ItemMod {
                content: Some((brace, content)),
                semi: None,
                ..i
            }
        } else {
            syn::fold::fold_item_mod(self, i)
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
        let file = std::fs::read_to_string(path).unwrap();
        let file = syn::parse_file(&file)?;
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
