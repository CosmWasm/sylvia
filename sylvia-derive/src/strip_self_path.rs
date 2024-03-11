use syn::fold::Fold;
use syn::Path;

pub struct StripSelfPath;

impl Fold for StripSelfPath {
    fn fold_path(&mut self, path: Path) -> Path {
        let segments = path
            .segments
            .into_iter()
            .filter(|segment| segment.ident != "Self")
            .collect();
        syn::fold::fold_path(self, Path { segments, ..path })
    }
}
