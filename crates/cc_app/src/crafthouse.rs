//! 3D CraftHouse in the Component Library: the open library of components
//! published at <https://3dcrafthouse.constructcraft.net>, browsed from its
//! catalog and downloaded one component at a time into the app's data
//! folder, from where the engine loads it like any component file.
//!
//! The network is used only here and only once the user picks this source:
//! the catalog, then the file of the component picked, over HTTPS in
//! background threads with bounded sizes (see [`crate::plugins_ui::get`]).
//! The catalog is hostile input: only files under `components/<folder>/`
//! with plain names are listed, so a download never leaves the cache folder.

use crate::library::Entry;
use crate::plugins_ui::{get, spawn};
use egui::Context;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

/// The site, opened in the browser from the library window.
pub const SITE: &str = "https://3dcrafthouse.constructcraft.net/";
/// Where the catalog and the component files are published.
const LIBRARY: &str = "https://3dcrafthouse.constructcraft.net/library/";
/// The catalog is not read past this many bytes.
const MAX_CATALOG: usize = 8 << 20;
/// A component file is not read past this many bytes.
const MAX_COMPONENT: usize = 16 << 20;
/// Most components listed from a catalog.
const MAX_ENTRIES: usize = 10_000;
/// Longest text taken from the catalog (a name, a description).
const MAX_TEXT: usize = 200;

/// The catalog as the window knows it.
#[derive(Debug, Default)]
enum Catalog {
    #[default]
    NotRead,
    Reading(Receiver<Result<Vec<Entry>, String>>),
    Read(Vec<Entry>),
    Failed(String),
}

/// A component file being downloaded: its catalog path and the local file.
#[derive(Debug)]
struct Download {
    file: PathBuf,
    result: Receiver<Result<PathBuf, String>>,
}

/// What the window shows of the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Reading,
    Failed(String),
    /// Read, with this many components.
    Ready(usize),
}

/// What the window shows of the component picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Downloading,
    Failed(String),
    /// Downloaded, to this file.
    Ready(PathBuf),
}

/// 3D CraftHouse state of the app (kept while the app runs, so the catalog
/// is read once and a component is downloaded once).
#[derive(Debug, Default)]
pub struct CraftHouse {
    catalog: Catalog,
    download: Option<Download>,
    /// The components downloaded: catalog path → local file.
    downloaded: BTreeMap<PathBuf, PathBuf>,
    /// The download that failed last, and why.
    failed: Option<(PathBuf, String)>,
}

impl CraftHouse {
    pub fn status(&self) -> Status {
        match &self.catalog {
            Catalog::NotRead | Catalog::Reading(_) => Status::Reading,
            Catalog::Failed(message) => Status::Failed(message.clone()),
            Catalog::Read(entries) => Status::Ready(entries.len()),
        }
    }

    /// The catalog's components (none until it is read).
    pub fn entries(&self) -> Vec<Entry> {
        match &self.catalog {
            Catalog::Read(entries) => entries.clone(),
            _ => Vec::new(),
        }
    }

    /// The local file of the catalog's `file`, once downloaded.
    pub fn local(&self, file: &Path) -> Option<&PathBuf> {
        self.downloaded.get(file)
    }

    /// How the download of the catalog's `file` stands.
    pub fn file_status(&self, file: &Path) -> FileStatus {
        match (self.local(file), &self.failed) {
            (Some(path), _) => FileStatus::Ready(path.clone()),
            (None, Some((failed, message))) if failed == file => FileStatus::Failed(message.clone()),
            _ => FileStatus::Downloading,
        }
    }

    /// Forget the catalog and the downloads, to read them again.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check the background work and start what is missing: the catalog,
    /// then the file of `selected`. Returns the catalog's components when
    /// it has just been read, and the local file of `selected` when it has
    /// just arrived.
    pub fn sync(&mut self, ctx: &Context, selected: Option<&Path>) -> (Option<Vec<Entry>>, Option<PathBuf>) {
        let mut read = None;
        self.catalog = match std::mem::take(&mut self.catalog) {
            Catalog::NotRead => Catalog::Reading(spawn(ctx, "crafthouse-catalog", || {
                parse_catalog(&get(&format!("{LIBRARY}catalog.json"), MAX_CATALOG)?)
            })),
            Catalog::Reading(rx) => match rx.try_recv() {
                Ok(Ok(entries)) => {
                    read = Some(entries.clone());
                    Catalog::Read(entries)
                }
                Ok(Err(message)) => Catalog::Failed(message),
                Err(TryRecvError::Empty) => Catalog::Reading(rx),
                Err(TryRecvError::Disconnected) => Catalog::Failed(String::new()),
            },
            other => other,
        };
        let mut arrived = None;
        if let Some(download) = self.download.take() {
            match download.result.try_recv() {
                Ok(Ok(path)) => {
                    if selected == Some(download.file.as_path()) {
                        arrived = Some(path.clone());
                    }
                    self.downloaded.insert(download.file, path);
                }
                Ok(Err(message)) => self.failed = Some((download.file, message)),
                Err(TryRecvError::Empty) => self.download = Some(download),
                Err(TryRecvError::Disconnected) => self.failed = Some((download.file, String::new())),
            }
        }
        if let (None, Some(file)) = (&self.download, selected)
            && self.file_status(file) == FileStatus::Downloading
            && let Some(url) = file_url(file)
        {
            let result = spawn(ctx, "crafthouse-download", move || download(&url.0, &url.1));
            self.download = Some(Download { file: file.to_path_buf(), result });
        }
        (read, arrived)
    }

    /// The catalog as if it had been read.
    #[cfg(test)]
    pub fn set_catalog(&mut self, entries: Vec<Entry>) {
        self.catalog = Catalog::Read(entries);
    }

    /// The catalog's `file` as if it had been downloaded to `path`.
    #[cfg(test)]
    pub fn set_downloaded(&mut self, file: &Path, path: PathBuf) {
        self.downloaded.insert(file.to_path_buf(), path);
    }
}

/// Where downloaded components are kept.
fn cache_dir() -> Option<PathBuf> {
    crate::recovery::data_dir().map(|d| d.join("crafthouse"))
}

/// The address of the catalog's `file` and its path under the cache
/// folder, when the catalog path is one the library publishes.
fn file_url(file: &Path) -> Option<(String, PathBuf)> {
    let (folder, slug) = file_parts(file.to_str()?)?;
    Some((format!("{LIBRARY}components/{folder}/{slug}.ccomp"), Path::new(folder).join(format!("{slug}.ccomp"))))
}

/// The folder and the name of `components/<folder>/<name>.ccomp`, both
/// of lowercase letters, digits and hyphens only.
fn file_parts(file: &str) -> Option<(&str, &str)> {
    let plain = |s: &str| {
        !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    let (folder, slug) = file.strip_prefix("components/")?.strip_suffix(".ccomp")?.split_once('/')?;
    (plain(folder) && plain(slug)).then_some((folder, slug))
}

/// Download `url` to `file` of the cache folder, kept only when it is a
/// component file the app reads.
fn download(url: &str, file: &Path) -> Result<PathBuf, String> {
    let bytes = get(url, MAX_COMPONENT)?;
    let path = cache_dir().ok_or("no folder to keep the download in")?.join(file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    if let Err(e) = cc_core::component_file::load(&path) {
        let _ = std::fs::remove_file(&path);
        return Err(e.to_string());
    }
    Ok(path)
}

/// The components of a `catalog.json`: those with a name and a file the
/// library publishes (see [`file_parts`]); the others are left out.
pub fn parse_catalog(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let json: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let list = json["components"].as_array().ok_or("the catalog has no component list")?;
    let text = |v: &serde_json::Value| v.as_str().unwrap_or_default().chars().take(MAX_TEXT).collect::<String>();
    let entries = list.iter().take(MAX_ENTRIES).filter_map(|c| {
        let file = c["file"].as_str().filter(|f| file_parts(f).is_some())?;
        let component = text(&c["component"]);
        if component.trim().is_empty() {
            return None;
        }
        let types = c["types"].as_array().into_iter().flatten().take(MAX_TEXT);
        let parameters = c["parameters"].as_array().into_iter().flatten().take(MAX_TEXT);
        Some(Entry {
            path: PathBuf::from(file),
            component,
            kind: text(&c["kind"]),
            author: text(&c["author"]),
            types: types.map(|t| (text(&t["name"]), text(&t["description"]))).collect(),
            parameters: parameters.map(text).collect(),
            connectors: Vec::new(),
        })
    });
    Ok(entries.collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOG: &str = r#"{
        "format_version": 1,
        "components": [
            {"file": "components/casework/armario-aereo.ccomp", "component": "Armário aéreo", "author": "Ana",
             "kind": "Casework", "mep": null, "parameters": ["Largura", "Altura"],
             "types": [{"name": "Aéreo 60 cm", "values": {"Largura": 0.6}, "description": "Cozinha"}]},
            {"file": "components/../../evil.ccomp", "component": "Up"},
            {"file": "/etc/passwd", "component": "Absolute"},
            {"file": "components/doors/Porta.ccomp", "component": "Uppercase"},
            {"file": "components/doors/a/b.ccomp", "component": "Too deep"},
            {"file": "components/doors/porta.skp", "component": "Not a component file"},
            {"file": "components/doors/sem-nome.ccomp", "component": " "},
            {"file": 3, "component": "No file"},
            {"file": "components/doors/porta.ccomp", "component": "Porta"}
        ]
    }"#;

    #[test]
    fn the_catalog_lists_only_files_the_library_publishes() {
        let entries = parse_catalog(CATALOG.as_bytes()).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.component.as_str()).collect();
        assert_eq!(names, ["Armário aéreo", "Porta"]);
        let first = &entries[0];
        assert_eq!(first.path, PathBuf::from("components/casework/armario-aereo.ccomp"));
        assert_eq!((first.kind.as_str(), first.author.as_str()), ("Casework", "Ana"));
        assert_eq!(first.types, [("Aéreo 60 cm".to_string(), "Cozinha".to_string())]);
        assert_eq!(first.parameters, ["Largura", "Altura"]);
        // A component with only its file and name still lists.
        assert!(entries[1].types.is_empty() && entries[1].author.is_empty());
    }

    #[test]
    fn a_catalog_that_is_not_one_is_refused() {
        assert!(parse_catalog(b"<html>").is_err());
        assert!(parse_catalog(b"{}").is_err());
        assert!(parse_catalog(br#"{"components": {}}"#).is_err());
        assert_eq!(parse_catalog(br#"{"components": []}"#).unwrap().len(), 0);
        // Long texts are cut.
        let long =
            format!(r#"{{"components": [{{"file": "components/a/b.ccomp", "component": "{}"}}]}}"#, "x".repeat(900));
        assert_eq!(parse_catalog(long.as_bytes()).unwrap()[0].component.len(), MAX_TEXT);
    }

    #[test]
    fn a_component_downloads_from_the_library_into_its_folder() {
        let (url, path) = file_url(Path::new("components/doors/porta-de-abrir.ccomp")).unwrap();
        assert_eq!(url, "https://3dcrafthouse.constructcraft.net/library/components/doors/porta-de-abrir.ccomp");
        assert_eq!(path, Path::new("doors").join("porta-de-abrir.ccomp"));
        assert!(path.is_relative());
        for bad in ["components/../x.ccomp", "components/doors/..ccomp", "components/doors/a b.ccomp", "x.ccomp"] {
            assert!(file_url(Path::new(bad)).is_none(), "{bad}");
        }
    }

    /// Offline (as tests are), the catalog fails to read and the window says so.
    #[test]
    fn offline_the_catalog_fails() {
        let ctx = Context::default();
        let mut house = CraftHouse::default();
        assert_eq!(house.status(), Status::Reading);
        house.sync(&ctx, None);
        for _ in 0..200 {
            if house.status() != Status::Reading {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            house.sync(&ctx, None);
        }
        assert_eq!(house.status(), Status::Failed("offline".into()));
        house.reset();
        assert_eq!(house.status(), Status::Reading);
    }
}
