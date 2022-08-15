//! Sylvia utlities to be used by buildscript to generate code

use cargo_toml::Manifest;
use std::env;
use std::path::{Path, PathBuf};
use syn::fold::Fold;
use syn_serde::Syn;

use mod_folder::ModFolder;

mod mod_folder;

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

        let mut folder = ModFolder::new(src_path);

        let file = syn::parse_file(&lib_file).expect("Cannot parse AST");
        let file = folder.fold_file(file);

        let out_dir: PathBuf = env::var_os("OUT_DIR").unwrap().into();
        let out_file = out_dir.join("lib.json");
        let out_file = std::fs::File::create(&out_file).expect("Cannot create output file");

        serde_json::to_writer_pretty(out_file, &file.to_adapter())
            .expect("Cannot write output file");
    }
}
