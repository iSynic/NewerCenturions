//! Read a fork-preserving classic installation without generating or changing files.

use crate::{Content, ContentError, Fork, Manifest, Resource, digest, safe_path, span};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize)]
pub struct SourceFork {
    pub path: String,
    pub offset: usize,
    #[serde(skip)]
    pub(crate) cached: Option<Arc<[u8]>>,
}

pub struct Installation {
    pub content: Content,
    pub resource_root: PathBuf,
}

impl Installation {
    pub fn open(root: &Path) -> Result<Self, ContentError> {
        if root.join("Data Files").is_dir() && root.join("Scenarios").is_dir() {
            return Self::direct(root);
        }
        // Retain the prepared development layout for existing fixture workflows.
        let extracted = root.join("analysis/extracted");
        if extracted.join("manifest.json").is_file() {
            return Ok(Self {
                content: Content::load(&root.join("NewCenturions"), &extracted)?,
                resource_root: extracted,
            });
        }
        Err(ContentError::Invalid(
            "place the Windows binaries beside the original New Centurions Mac application, Data Files, Scenarios and Warriors folders; preserve resource forks as .rsrc or AppleDouble sidecars".into(),
        ))
    }

    fn direct(root: &Path) -> Result<Self, ContentError> {
        let mut files = BTreeSet::new();
        for name in ["Data Files", "Scenarios", "Warriors"] {
            collect(root, &root.join(name), &mut files)?;
        }
        for name in ["New Centurions", "New Centurions.rsrc", "._New Centurions"] {
            if root.join(name).is_file() {
                files.insert(name.to_string());
            }
        }
        let mut forks = vec![];
        let mut resources = vec![];
        let mut owners = BTreeSet::new();
        for path in files {
            let name = Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| ContentError::Invalid("non-Unicode installation filename".into()))?;
            let sidecar_name = name
                .strip_suffix(".rsrc")
                .or_else(|| name.strip_prefix("._"));
            let raw = read_file(&safe_path(root, &path)?)?;
            if let Some(base) = sidecar_name {
                // Finder-info-only AppleDouble files carry no resource payload.
                if wrapped(&raw) && !has_fork(&raw, 2)? {
                    continue;
                }
                let parent = Path::new(&path).parent().unwrap_or(Path::new(""));
                let owner = parent.join(base).to_string_lossy().replace('\\', "/");
                add_resources(&raw, &path, &owner, &mut owners, &mut resources)?;
                let payload = fork_payload(&raw, 2)?;
                forks.push(Fork {
                    path: format!("TheNewCenturions/{path}"),
                    kind: "resource".into(),
                    size: payload.len(),
                    sha256: digest(payload),
                });
            } else {
                let data = fork_payload(&raw, 1)?;
                forks.push(Fork {
                    path: format!("TheNewCenturions/{path}"),
                    kind: "data".into(),
                    size: data.len(),
                    sha256: digest(data),
                });
                if wrapped(&raw) && has_fork(&raw, 2)? {
                    add_resources(&raw, &path, &path, &mut owners, &mut resources)?;
                    let payload = fork_payload(&raw, 2)?;
                    forks.push(Fork {
                        path: format!("TheNewCenturions/{path}"),
                        kind: "resource".into(),
                        size: payload.len(),
                        sha256: digest(payload),
                    });
                }
            }
        }
        let identities: Vec<_> = forks
            .iter()
            .map(|f| (&f.path, &f.kind, &f.sha256))
            .collect();
        let identity = digest(&serde_json::to_vec(&identities)?);
        let manifest = Manifest {
            schema_version: 1,
            input_manifest_sha256: identity,
            forks,
            resources,
        };
        Ok(Self {
            content: Content::from_manifest(root, root, manifest)?,
            resource_root: root.to_path_buf(),
        })
    }
}

fn collect(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<String>,
) -> Result<(), ContentError> {
    if !directory.is_dir() {
        return Err(ContentError::Invalid(format!(
            "missing original folder {}",
            directory.display()
        )));
    }
    if fs::symlink_metadata(directory)?.file_type().is_symlink() {
        return Err(ContentError::Invalid(
            "installation directory symlinks are not supported".into(),
        ));
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && !name.starts_with("._") {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(ContentError::Invalid(
                "installation symlinks are not supported".into(),
            ));
        }
        if kind.is_dir() {
            // Classic scenario trees are shallow; refuse an unbounded directory walk.
            if entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .components()
                .count()
                > 16
            {
                return Err(ContentError::Invalid(
                    "installation directory nesting exceeds limit".into(),
                ));
            }
            collect(root, &entry.path(), files)?;
        } else if kind.is_file() {
            if files.len() >= 10000 {
                return Err(ContentError::Invalid(
                    "installation file count exceeds limit".into(),
                ));
            }
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

pub(crate) fn read_file(path: &Path) -> Result<Vec<u8>, ContentError> {
    if fs::metadata(path)?.len() > 64 * 1024 * 1024 {
        return Err(ContentError::Invalid(format!(
            "original file exceeds 64 MiB: {}",
            path.display()
        )));
    }
    Ok(fs::read(path)?)
}

pub(crate) fn read_data(root: &Path, relative: &str) -> Result<Vec<u8>, ContentError> {
    let relative = if root.join("Data Files").is_dir() {
        relative
            .strip_prefix("TheNewCenturions/")
            .unwrap_or(relative)
    } else {
        relative
    };
    let raw = read_file(&safe_path(root, relative)?)?;
    Ok(fork_payload(&raw, 1)?.to_vec())
}

fn wrapped(bytes: &[u8]) -> bool {
    matches!(bytes.get(..4), Some([0, 5, 0x16, 0] | [0, 5, 0x16, 7]))
}

fn descriptors(bytes: &[u8]) -> Result<&[u8], ContentError> {
    let count = usize::from(u16::from_be_bytes(span(bytes, 24, 2)?.try_into().unwrap()));
    span(bytes, 26, count * 12)
}

fn has_fork(bytes: &[u8], id: u32) -> Result<bool, ContentError> {
    Ok(descriptors(bytes)?
        .as_chunks::<12>()
        .0
        .iter()
        .any(|r| u32::from_be_bytes(r[..4].try_into().unwrap()) == id))
}

pub(crate) fn fork_payload(bytes: &[u8], id: u32) -> Result<&[u8], ContentError> {
    if !wrapped(bytes) {
        return Ok(bytes);
    }
    let mut result = None;
    for row in descriptors(bytes)?.as_chunks::<12>().0 {
        let kind = u32::from_be_bytes(row[..4].try_into().unwrap());
        let offset = u32::from_be_bytes(row[4..8].try_into().unwrap()) as usize;
        let size = u32::from_be_bytes(row[8..12].try_into().unwrap()) as usize;
        let payload = span(bytes, offset, size)?;
        if kind == id {
            if result.is_some() {
                return Err(ContentError::Invalid(
                    "duplicate AppleSingle/Double fork".into(),
                ));
            }
            result = Some(payload);
        }
    }
    result.ok_or_else(|| ContentError::Invalid(format!("AppleSingle/Double missing fork {id}")))
}

fn add_resources(
    raw: &[u8],
    path: &str,
    owner: &str,
    owners: &mut BTreeSet<String>,
    output: &mut Vec<Resource>,
) -> Result<(), ContentError> {
    if !owners.insert(owner.to_string()) {
        return Err(ContentError::Invalid(format!(
            "multiple resource forks for {owner}"
        )));
    }
    let fork = fork_payload(raw, 2)?;
    let cached: Arc<[u8]> = Arc::from(fork);
    let number = |at| -> Result<usize, ContentError> {
        Ok(u32::from_be_bytes(span(fork, at, 4)?.try_into().unwrap()) as usize)
    };
    let data_offset = number(0)?;
    let map_offset = number(4)?;
    let data = span(fork, data_offset, number(8)?)?;
    let map = span(fork, map_offset, number(12)?)?;
    let short = |at| -> Result<usize, ContentError> {
        Ok(usize::from(u16::from_be_bytes(
            span(map, at, 2)?.try_into().unwrap(),
        )))
    };
    let types = short(24)?;
    let names = short(26)?;
    let count = short(types)?;
    let count = if count == 0xffff { 0 } else { count + 1 };
    let mut seen = BTreeSet::new();
    for row in span(map, types + 2, count * 8)?.as_chunks::<8>().0 {
        let resource_type = encoding_rs::MACINTOSH.decode(&row[..4]).0.into_owned();
        let count = usize::from(u16::from_be_bytes(row[4..6].try_into().unwrap())) + 1;
        let refs = usize::from(u16::from_be_bytes(row[6..8].try_into().unwrap()));
        for reference in span(map, types + refs, count * 12)?.as_chunks::<12>().0 {
            if output.len() >= 100_000 {
                return Err(ContentError::Invalid(
                    "installation resource count exceeds limit".into(),
                ));
            }
            let id = i16::from_be_bytes(reference[..2].try_into().unwrap());
            if !seen.insert((resource_type.clone(), id)) {
                return Err(ContentError::Invalid(format!(
                    "duplicate resource {resource_type}:{id} in {owner}"
                )));
            }
            let name = usize::from(u16::from_be_bytes(reference[2..4].try_into().unwrap()));
            if name != 0xffff {
                let length = usize::from(span(map, names + name, 1)?[0]);
                span(map, names + name + 1, length)?;
            }
            let offset = (usize::from(reference[5]) << 16)
                | (usize::from(reference[6]) << 8)
                | usize::from(reference[7]);
            let size = u32::from_be_bytes(span(data, offset, 4)?.try_into().unwrap()) as usize;
            let payload = span(data, offset + 4, size)?;
            output.push(Resource {
                original: format!("TheNewCenturions/{owner}"),
                r#type: resource_type.clone(),
                id,
                size,
                sha256: digest(payload),
                output: String::new(),
                source_fork: Some(SourceFork {
                    path: path.into(),
                    offset: data_offset + offset + 4,
                    cached: Some(cached.clone()),
                }),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_map_and_appledouble_preserve_signed_ids_and_reject_truncation() {
        // One TEXT resource, ID -5, with a five-byte payload.
        let mut fork = vec![0; 256];
        fork[..16].copy_from_slice(&[0, 0, 1, 0, 0, 0, 1, 9, 0, 0, 0, 9, 0, 0, 0, 50]);
        fork.extend_from_slice(&[0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o']);
        let mut map = vec![0; 50];
        map[24..28].copy_from_slice(&[0, 28, 0, 50]);
        map[28..30].copy_from_slice(&[0, 0]);
        map[30..38].copy_from_slice(&[b'T', b'E', b'X', b'T', 0, 0, 0, 10]);
        map[38..42].copy_from_slice(&[0xff, 0xfb, 0xff, 0xff]);
        fork.extend_from_slice(&map);
        let mut wrapper = vec![0; 38];
        wrapper[..4].copy_from_slice(&[0, 5, 0x16, 7]);
        wrapper[24..26].copy_from_slice(&[0, 1]);
        wrapper[26..30].copy_from_slice(&2_u32.to_be_bytes());
        wrapper[30..34].copy_from_slice(&38_u32.to_be_bytes());
        wrapper[34..38].copy_from_slice(&(fork.len() as u32).to_be_bytes());
        wrapper.extend_from_slice(&fork);
        for bytes in [&fork, &wrapper] {
            let mut output = vec![];
            add_resources(
                bytes,
                "Scenario.rsrc",
                "Scenario",
                &mut BTreeSet::new(),
                &mut output,
            )
            .unwrap();
            assert_eq!(output.len(), 1);
            assert_eq!(output[0].id, -5);
            assert_eq!(output[0].sha256, digest(b"hello"));
            let source = output[0].source_fork.as_ref().unwrap();
            assert_eq!(
                span(
                    fork_payload(bytes, 2).unwrap(),
                    source.offset,
                    output[0].size
                )
                .unwrap(),
                b"hello"
            );
            assert!(
                add_resources(
                    &bytes[..bytes.len() - 1],
                    "Scenario.rsrc",
                    "Scenario",
                    &mut BTreeSet::new(),
                    &mut vec![]
                )
                .is_err()
            );
        }
    }
}
