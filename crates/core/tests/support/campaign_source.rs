use centurions_content::digest;
use std::path::{Path, PathBuf};

pub fn source_digest(root: &Path) -> String {
    fn collect(path: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    let mut files = vec![root.join("Cargo.lock"), root.join("Cargo.toml")];
    for package in ["core", "content"] {
        files.push(root.join(format!("crates/{package}/Cargo.toml")));
        collect(&root.join(format!("crates/{package}/src")), &mut files);
    }
    files.sort();
    let inputs: Vec<_> = files
        .iter()
        .map(|path| {
            (
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                digest(&std::fs::read(path).unwrap()),
            )
        })
        .collect();
    digest(&serde_json::to_vec(&inputs).unwrap())
}
