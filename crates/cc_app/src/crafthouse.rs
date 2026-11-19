//! 3D CraftHouse in the Component Library: the open library of components
//! published at <https://3dcrafthouse.constructcraft.net>, browsed from its
//! catalog and downloaded one component at a time into the app's data
//! folder, from where the engine loads it like any component file.
//!
//! The network is used only here and only once the user picks this source:
//! the catalog, the pictures the catalog gives for the components the list
//! shows (a few at a time), and the file of the component picked, over HTTPS
//! in background threads with bounded sizes (see [`crate::plugins_ui::get`]).
//! A component's file is downloaded only when it is picked.
//! The catalog is hostile input: only files under `components/<folder>/`
//! and pictures under `thumbnails/<folder>/` with plain names are taken, so
//! a download never leaves the cache folder, and a picture is kept only
//! when it is a PNG of a bounded size.

use crate::library::Entry;
use crate::plugins_ui::{get, spawn};
use egui::Context;
use std::collections::{BTreeMap, BTreeSet};
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
/// A picture is not read past this many bytes.
const MAX_PICTURE: usize = 1 << 20;
/// Most pixels on each side of a picture.
const MAX_PICTURE_SIDE: u32 = 2048;
/// Most pictures downloaded at once.
const PICTURE_DOWNLOADS: usize = 4;
/// Most pictures asked for in one frame.
const MAX_PICTURES_WANTED: usize = 256;
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

/// A file being downloaded: its catalog path and the local file.
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
    /// Not there yet: being downloaded, or not asked for.
    Downloading,
    Failed(String),
    /// Downloaded, to this file.
    Ready(PathBuf),
}

/// 3D CraftHouse state of the app (kept while the app runs, so the catalog
/// is read once and a component or a picture is downloaded once).
#[derive(Debug, Default)]
pub struct CraftHouse {
    catalog: Catalog,
    /// The download of the component picked.
    download: Option<Download>,
    /// The components downloaded: catalog path → local file.
    downloaded: BTreeMap<PathBuf, PathBuf>,
    /// The downloads that failed, and why.
    failed: BTreeMap<PathBuf, String>,
    /// The catalog paths whose copy kept from an earlier run was looked for.
    looked: BTreeSet<PathBuf>,
    /// The pictures downloaded: catalog path → local file.
    pictures: BTreeMap<PathBuf, PathBuf>,
    /// The pictures being downloaded, at most [`PICTURE_DOWNLOADS`].
    picture_downloads: Vec<Download>,
    /// The pictures that did not download or are not ones the window shows
    /// (asked for again only after Refresh).
    pictures_failed: BTreeSet<PathBuf>,
    /// The pictures asked for since the last [`CraftHouse::sync`], in the
    /// order asked: those of the cards in sight.
    pictures_wanted: Vec<PathBuf>,
    /// Files kept from earlier runs are downloaded again (after Refresh).
    fresh: bool,
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

    /// The local file of the catalog's `file`: downloaded, or a copy kept
    /// from an earlier run (looked for once). Nothing is downloaded for it.
    pub fn local_or_kept(&mut self, file: &Path) -> Option<PathBuf> {
        if !self.downloaded.contains_key(file)
            && self.looked.insert(file.to_path_buf())
            && let Some(path) = file_url(file).and_then(|(_, relative)| self.kept(&relative))
        {
            self.downloaded.insert(file.to_path_buf(), path);
        }
        self.downloaded.get(file).cloned()
    }

    /// How the download of the catalog's `file` stands.
    pub fn file_status(&self, file: &Path) -> FileStatus {
        match (self.local(file), self.failed.get(file)) {
            (Some(path), _) => FileStatus::Ready(path.clone()),
            (None, Some(message)) => FileStatus::Failed(message.clone()),
            _ => FileStatus::Downloading,
        }
    }

    /// The local file of the catalog's `picture` (an entry's
    /// [`Entry::thumbnail`]), once it is there.
    pub fn picture(&self, picture: &Path) -> Option<&PathBuf> {
        self.pictures.get(picture)
    }

    /// Ask for the catalog's `picture`, to show it on a card in sight: a
    /// copy kept from an earlier run is taken as it is; else the picture is
    /// downloaded by the next [`CraftHouse::sync`], a few at a time. Asked
    /// again each frame while it is wanted.
    pub fn want_picture(&mut self, picture: &Path) {
        if self.pictures.contains_key(picture)
            || self.pictures_failed.contains(picture)
            || self.picture_downloads.iter().any(|d| d.file == picture)
            || self.pictures_wanted.iter().any(|w| w == picture)
        {
            return;
        }
        let Some((_, relative)) = picture_url(picture) else {
            self.pictures_failed.insert(picture.to_path_buf());
            return;
        };
        if self.looked.insert(picture.to_path_buf())
            && let Some(path) = self.kept(&relative)
        {
            self.pictures.insert(picture.to_path_buf(), path);
        } else if self.pictures_wanted.len() < MAX_PICTURES_WANTED {
            self.pictures_wanted.push(picture.to_path_buf());
        }
    }

    /// The copy of `relative` (a path under the cache folder) kept from an
    /// earlier run, unless the catalog was read again to bring the library
    /// up to date. Tests see no kept copies.
    fn kept(&self, relative: &Path) -> Option<PathBuf> {
        if self.fresh || cfg!(test) {
            return None;
        }
        let path = cache_dir()?.join(relative);
        path.is_file().then_some(path)
    }

    /// Forget the catalog and the downloads, to read them again from the site.
    pub fn reset(&mut self) {
        *self = Self { fresh: true, ..Self::default() };
    }

    /// Check the background work and start what is missing: the catalog,
    /// the file of `selected`, and the pictures asked for since the last
    /// call. Returns the catalog's components when it has just been read,
    /// and the local file of `selected` when it has just arrived.
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
        // A copy of the component picked, kept from an earlier run, is there at once.
        if let Some(file) = selected
            && !self.downloaded.contains_key(file)
            && let Some(path) = self.local_or_kept(file)
        {
            arrived = Some(path);
        }
        if let Some(download) = self.download.take() {
            match download.result.try_recv() {
                Ok(Ok(path)) => {
                    if selected == Some(download.file.as_path()) {
                        arrived = Some(path.clone());
                    }
                    self.downloaded.insert(download.file, path);
                }
                Ok(Err(message)) => _ = self.failed.insert(download.file, message),
                Err(TryRecvError::Empty) => self.download = Some(download),
                Err(TryRecvError::Disconnected) => _ = self.failed.insert(download.file, String::new()),
            }
        }
        // Only the component picked is downloaded, one at a time.
        if self.download.is_none()
            && let Some(file) = selected
            && self.file_status(file) == FileStatus::Downloading
            && let Some((url, relative)) = file_url(file)
        {
            let result = spawn(ctx, "crafthouse-download", move || download(&url, &relative));
            self.download = Some(Download { file: file.to_path_buf(), result });
        }
        self.sync_pictures(ctx);
        (read, arrived)
    }

    /// Take the pictures that arrived and start downloading those asked for
    /// since the last call, up to [`PICTURE_DOWNLOADS`] at once. The others
    /// are forgotten: the cards still in sight ask for them again.
    fn sync_pictures(&mut self, ctx: &Context) {
        let mut running = Vec::new();
        for download in std::mem::take(&mut self.picture_downloads) {
            match download.result.try_recv() {
                Ok(Ok(path)) => _ = self.pictures.insert(download.file, path),
                Ok(Err(message)) => {
                    log::debug!("picture {} not downloaded: {message}", download.file.display());
                    self.pictures_failed.insert(download.file);
                }
                Err(TryRecvError::Empty) => running.push(download),
                Err(TryRecvError::Disconnected) => _ = self.pictures_failed.insert(download.file),
            }
        }
        self.picture_downloads = running;
        for picture in std::mem::take(&mut self.pictures_wanted) {
            if self.picture_downloads.len() >= PICTURE_DOWNLOADS {
                break;
            }
            if self.pictures.contains_key(&picture)
                || self.pictures_failed.contains(&picture)
                || self.picture_downloads.iter().any(|d| d.file == picture)
            {
                continue;
            }
            let Some((url, relative)) = picture_url(&picture) else {
                self.pictures_failed.insert(picture);
                continue;
            };
            let result = spawn(ctx, "crafthouse-picture", move || download_picture(&url, &relative));
            self.picture_downloads.push(Download { file: picture, result });
        }
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

    /// The catalog's `picture` as if it had been downloaded to `path`.
    #[cfg(test)]
    pub fn set_picture(&mut self, picture: &Path, path: PathBuf) {
        self.pictures.insert(picture.to_path_buf(), path);
    }
}

/// Where downloaded components and pictures are kept.
fn cache_dir() -> Option<PathBuf> {
    crate::recovery::data_dir().map(|d| d.join("crafthouse"))
}

/// The address of the catalog's `file` and its path under the cache
/// folder, when the catalog path is one the library publishes.
fn file_url(file: &Path) -> Option<(String, PathBuf)> {
    let (folder, slug) = plain_parts(file.to_str()?, "components/", ".ccomp")?;
    Some((format!("{LIBRARY}components/{folder}/{slug}.ccomp"), Path::new(folder).join(format!("{slug}.ccomp"))))
}

/// The address of the catalog's `picture` and its path under the cache
/// folder (in `thumbnails/`), when the catalog path is one the library
/// publishes.
fn picture_url(picture: &Path) -> Option<(String, PathBuf)> {
    let (folder, name) = plain_parts(picture.to_str()?, "thumbnails/", ".png")?;
    let local = Path::new("thumbnails").join(folder).join(format!("{name}.png"));
    Some((format!("{LIBRARY}thumbnails/{folder}/{name}.png"), local))
}

/// The folder and the name of `<root><folder>/<name><extension>`, both of
/// lowercase letters, digits and hyphens only, 1 to 80 of them.
fn plain_parts<'a>(file: &'a str, root: &str, extension: &str) -> Option<(&'a str, &'a str)> {
    let plain = |s: &str| {
        !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    let (folder, name) = file.strip_prefix(root)?.strip_suffix(extension)?.split_once('/')?;
    (plain(folder) && plain(name)).then_some((folder, name))
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

/// Download the picture at `url` to `file` of the cache folder, kept only
/// when [`check_picture`] takes it (a bad copy there is deleted).
fn download_picture(url: &str, file: &Path) -> Result<PathBuf, String> {
    let bytes = get(url, MAX_PICTURE)?;
    let path = cache_dir().ok_or("no folder to keep the download in")?.join(file);
    if let Err(e) = check_picture(&bytes) {
        let _ = std::fs::remove_file(&path);
        return Err(e);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Whether `bytes` are a picture the window shows: at most [`MAX_PICTURE`]
/// bytes of a PNG that decodes whole, 1 to [`MAX_PICTURE_SIDE`] pixels on
/// each side.
fn check_picture(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_PICTURE {
        return Err("the picture is too large".into());
    }
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().map_err(|e| e.to_string())?;
    let (width, height) = (reader.info().width, reader.info().height);
    let side = 1..=MAX_PICTURE_SIDE;
    if !side.contains(&width) || !side.contains(&height) {
        return Err(format!("a picture of {width} by {height} pixels"));
    }
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("the picture is too large")?];
    reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    Ok(())
}

/// The components of a `catalog.json`: those with a name and a file the
/// library publishes (see [`plain_parts`]); the others are left out. A
/// `thumbnail` that is not a picture the library publishes is dropped.
pub fn parse_catalog(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let json: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let list = json["components"].as_array().ok_or("the catalog has no component list")?;
    let text = |v: &serde_json::Value| v.as_str().unwrap_or_default().chars().take(MAX_TEXT).collect::<String>();
    let entries = list.iter().take(MAX_ENTRIES).filter_map(|c| {
        let file = c["file"].as_str().filter(|f| file_url(Path::new(f)).is_some())?;
        let component = text(&c["component"]);
        if component.trim().is_empty() {
            return None;
        }
        let thumbnail = c["thumbnail"].as_str().map(PathBuf::from).filter(|t| picture_url(t).is_some());
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
            thumbnail,
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
        assert!(entries.iter().all(|e| e.thumbnail.is_none()));
    }

    /// A component keeps the picture the catalog gives when it is one the
    /// library publishes; with any other, it lists with no picture.
    #[test]
    fn the_catalog_keeps_only_pictures_the_library_publishes() {
        let good = ["thumbnails/doors/porta-de-abrir.png", "thumbnails/a/1.png"];
        let bad = [
            r#""thumbnails/../evil.png""#,
            r#""thumbnails/doors/../../evil.png""#,
            r#""/thumbnails/doors/porta.png""#,
            r#""C:/thumbnails/doors/porta.png""#,
            r#""https://example.com/thumbnails/doors/porta.png""#,
            r#""thumbnails/Doors/porta.png""#,
            r#""thumbnails/doors/Porta.png""#,
            r#""thumbnails/doors/porta.PNG""#,
            r#""thumbnails/doors/porta.jpg""#,
            r#""thumbnails/doors/a/porta.png""#,
            r#""thumbnails/porta.png""#,
            r#""components/doors/porta.png""#,
            r#""thumbnails/doors/.png""#,
            r#""thumbnails/doors/a b.png""#,
            r#""""#,
            "3",
            "null",
            r#"["thumbnails/doors/porta.png"]"#,
            r#"{"path": "thumbnails/doors/porta.png"}"#,
        ];
        let json = |thumbnail: &str| {
            format!(
                r#"{{"components": [{{"file": "components/doors/porta.ccomp", "component": "Porta", "thumbnail": {thumbnail}}}]}}"#
            )
        };
        for value in good {
            let entries = parse_catalog(json(&format!("\"{value}\"")).as_bytes()).unwrap();
            assert_eq!(entries[0].thumbnail.as_deref(), Some(Path::new(value)), "{value}");
        }
        let long = format!(r#""thumbnails/doors/{}.png""#, "a".repeat(81));
        for value in bad.iter().copied().chain([long.as_str()]) {
            let entries = parse_catalog(json(value).as_bytes()).unwrap();
            assert_eq!(entries.len(), 1, "the component still lists: {value}");
            assert!(entries[0].thumbnail.is_none(), "{value}");
        }
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

    #[test]
    fn a_picture_downloads_from_the_library_into_its_folder() {
        let (url, path) = picture_url(Path::new("thumbnails/doors/porta-de-abrir.png")).unwrap();
        assert_eq!(url, "https://3dcrafthouse.constructcraft.net/library/thumbnails/doors/porta-de-abrir.png");
        assert_eq!(path, Path::new("thumbnails").join("doors").join("porta-de-abrir.png"));
        assert!(path.is_relative());
        for bad in ["thumbnails/../x.png", "thumbnails/doors/..png", "components/doors/porta.png", "x.png"] {
            assert!(picture_url(Path::new(bad)).is_none(), "{bad}");
        }
    }

    /// A PNG of `width` × `height` pixels.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let pixels = vec![128; (width * height * 4) as usize];
        encoder.write_header().unwrap().write_image_data(&pixels).unwrap();
        bytes
    }

    /// Only a whole PNG of a bounded size is taken as a picture.
    #[test]
    fn a_picture_is_kept_only_when_it_is_a_png_of_a_bounded_size() {
        assert_eq!(check_picture(&png(360, 240)), Ok(()));
        assert_eq!(check_picture(&png(1, MAX_PICTURE_SIDE)), Ok(()));
        assert!(check_picture(&png(MAX_PICTURE_SIDE + 1, 1)).is_err(), "too wide");
        assert!(check_picture(&png(1, MAX_PICTURE_SIDE + 1)).is_err(), "too high");
        assert!(check_picture(b"<html>not found</html>").is_err());
        assert!(check_picture(b"").is_err());
        let picture = png(36, 24);
        assert!(check_picture(&picture[..picture.len() / 2]).is_err(), "cut short");
        // A JPEG's first bytes.
        assert!(check_picture(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 16, b'J', b'F', b'I', b'F', 0]).is_err());
        let mut large = png(4, 4);
        large.resize(MAX_PICTURE + 1, 0);
        assert!(check_picture(&large).is_err(), "too many bytes");
        // From a file, as a picture kept in the cache folder is.
        let dir = std::env::temp_dir().join(format!("ccad-crafthouse-pictures-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (good, bad) = (dir.join("good.png"), dir.join("bad.png"));
        std::fs::write(&good, png(360, 240)).unwrap();
        std::fs::write(&bad, png(4000, 2)).unwrap();
        assert!(check_picture(&std::fs::read(&good).unwrap()).is_ok());
        assert!(check_picture(&std::fs::read(&bad).unwrap()).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Offline, a picture asked for is downloaded once, fails, and is not
    /// asked for again until Refresh; no component file is downloaded for it.
    #[test]
    fn a_picture_that_does_not_download_is_not_asked_for_again() {
        let ctx = Context::default();
        let mut house = CraftHouse::default();
        let picture = Path::new("thumbnails/doors/porta.png");
        house.want_picture(picture);
        house.sync(&ctx, None);
        assert_eq!(house.picture_downloads.len(), 1);
        for _ in 0..200 {
            if house.pictures_failed.contains(picture) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            house.sync(&ctx, None);
        }
        assert!(house.pictures_failed.contains(picture));
        house.want_picture(picture);
        assert!(house.pictures_wanted.is_empty() && house.picture(picture).is_none());
        assert!(house.download.is_none() && house.failed.is_empty());
        house.reset();
        house.want_picture(picture);
        assert_eq!(house.pictures_wanted.len(), 1);
    }

    /// At most a few pictures download at once; the others wait for a frame
    /// in which their cards ask again.
    #[test]
    fn pictures_download_a_few_at_a_time() {
        let ctx = Context::default();
        let mut house = CraftHouse::default();
        let pictures: Vec<PathBuf> = (0..10).map(|i| PathBuf::from(format!("thumbnails/misc/c-{i}.png"))).collect();
        for p in &pictures {
            house.want_picture(p);
        }
        house.want_picture(&pictures[0]);
        assert_eq!(house.pictures_wanted.len(), pictures.len(), "each asked once");
        house.sync(&ctx, None);
        assert_eq!(house.picture_downloads.len(), PICTURE_DOWNLOADS);
        assert!(house.pictures_wanted.is_empty());
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
