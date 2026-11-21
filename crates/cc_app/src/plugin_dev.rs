//! Plugin developer mode (Preferences): every second the plugins folder and
//! the development folder are listed (the size and modification time of
//! their `*.lua`, `*.toml` and `*.svg` files, at most [`MAX_FILES`] files
//! [`MAX_DEPTH`] folders deep, hidden folders left out), and the plugins
//! load again when the listing changed, unless a plugin command is running
//! (then as soon as it ended). The listing is a plain read of the folders'
//! metadata on the UI thread: no watcher thread and no new dependency.

use crate::state::AppState;
use crate::t;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Time between two listings.
pub const INTERVAL: Duration = Duration::from_secs(1);
/// Most files listed (all folders together), and how deep.
pub const MAX_FILES: usize = 2000;
pub const MAX_DEPTH: usize = 6;
/// The files whose changes count.
const EXTENSIONS: [&str; 3] = ["lua", "toml", "svg"];

/// A file's size and modification time.
pub type Stamp = (u64, Option<SystemTime>);

/// The watched files of a listing, by path.
pub type Snapshot = BTreeMap<PathBuf, Stamp>;

/// Whether a file at `path` is watched (by its extension).
pub fn watched(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// The snapshot of a listing: the watched files, at most [`MAX_FILES`].
pub fn snapshot(listing: impl IntoIterator<Item = (PathBuf, Stamp)>) -> Snapshot {
    listing.into_iter().filter(|(p, _)| watched(p)).take(MAX_FILES).collect()
}

/// Whether the plugins should load again: a watched file came, went, or
/// changed size or time (`None`: nothing was listed yet).
pub fn changed(before: Option<&Snapshot>, now: &Snapshot) -> bool {
    before.is_some_and(|b| b != now)
}

/// The files under `folders`, depth first, bounded.
fn list(folders: &[PathBuf]) -> Vec<(PathBuf, Stamp)> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = folders.iter().rev().map(|f| (f.clone(), 0)).collect();
    // Folders read, also bounded (a tree of empty folders).
    let mut read = 0;
    while let Some((dir, depth)) = stack.pop() {
        read += 1;
        if read > MAX_FILES || out.len() >= MAX_FILES {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut paths: Vec<(PathBuf, std::fs::Metadata)> =
            entries.filter_map(|e| e.ok()).filter_map(|e| Some((e.path(), e.metadata().ok()?))).collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, meta) in paths {
            let hidden = path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if meta.is_dir() {
                if !hidden && depth < MAX_DEPTH {
                    stack.push((path, depth + 1));
                }
            } else if meta.is_file() && watched(&path) {
                out.push((path, (meta.len(), meta.modified().ok())));
                if out.len() >= MAX_FILES {
                    break;
                }
            }
        }
    }
    out
}

/// What developer mode keeps between frames.
#[derive(Debug, Default)]
pub struct Watcher {
    last: Option<Instant>,
    snapshot: Option<Snapshot>,
    /// The folders of the snapshot (a new set starts over).
    folders: Vec<PathBuf>,
    /// A change was seen; the plugins load again when nothing runs.
    pub pending: bool,
}

/// The folders developer mode watches.
fn folders(state: &AppState) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = state.prefs.plugins_dir().into_iter().collect();
    if let Some(dev) = state.prefs.plugin_dev_dir() {
        out.push(dev);
    }
    out
}

/// Every frame: in developer mode, list the folders once a second and load
/// the plugins again after a change (when no plugin command runs).
pub fn poll(ctx: &egui::Context, state: &mut AppState) {
    if !state.prefs.plugin_dev_mode {
        state.plugins.dev = Watcher::default();
        return;
    }
    ctx.request_repaint_after(INTERVAL);
    let now = Instant::now();
    let due = state.plugins.dev.last.is_none_or(|t| now.duration_since(t) >= INTERVAL);
    if due {
        let folders = folders(state);
        let snap = snapshot(list(&folders));
        let w = &mut state.plugins.dev;
        let same_folders = w.folders == folders;
        if same_folders && changed(w.snapshot.as_ref(), &snap) {
            w.pending = true;
        }
        w.snapshot = Some(snap);
        w.folders = folders;
        w.last = Some(now);
    }
    if state.plugins.dev.pending && !crate::plugins_ui::busy(state) {
        state.plugins.dev.pending = false;
        crate::plugins_ui::load(state);
        state.message = t!("plug-dev-reloaded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: u64) -> Option<SystemTime> {
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
    }

    #[test]
    fn changes_of_watched_files_are_seen() {
        let base = vec![
            (PathBuf::from("p/plugin.toml"), (100, at(1))),
            (PathBuf::from("p/main.lua"), (200, at(1))),
            (PathBuf::from("p/icons/a.svg"), (50, at(1))),
            (PathBuf::from("p/notes.txt"), (5, at(1))),
            (PathBuf::from("p/.settings/p.json"), (5, at(1))),
        ];
        let first = snapshot(base.clone());
        assert_eq!(first.len(), 3, "only .lua, .toml and .svg");
        assert!(!changed(None, &first), "the first listing is no change");
        assert!(!changed(Some(&first), &snapshot(base.clone())), "the same");
        let edit = |i: usize, stamp: Stamp| {
            let mut list = base.clone();
            list[i].1 = stamp;
            snapshot(list)
        };
        assert!(changed(Some(&first), &edit(1, (200, at(2)))), "a newer time");
        assert!(changed(Some(&first), &edit(1, (201, at(1)))), "another size");
        assert!(changed(Some(&first), &edit(2, (50, None))), "a time no longer read");
        assert!(!changed(Some(&first), &edit(3, (6, at(9)))), "a .txt file does not count");
        let mut added = base.clone();
        added.push((PathBuf::from("p/lib/x.lua"), (1, at(1))));
        assert!(changed(Some(&first), &snapshot(added)), "a file added");
        assert!(changed(Some(&first), &snapshot(base[1..].to_vec())), "a file removed");
        assert!(watched(Path::new("A/B.LUA")));
        let many = (0..MAX_FILES + 10).map(|i| (PathBuf::from(format!("f{i}.lua")), (1, at(1))));
        assert_eq!(snapshot(many).len(), MAX_FILES, "bounded");
    }

    #[test]
    fn listing_reads_watched_files_of_the_folders() {
        let root = std::env::temp_dir().join(format!("ccad-plugin-dev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("p/lib")).unwrap();
        std::fs::create_dir_all(root.join("p/.git")).unwrap();
        std::fs::write(root.join("p/main.lua"), "x").unwrap();
        std::fs::write(root.join("p/lib/a.lua"), "y").unwrap();
        std::fs::write(root.join("p/.git/b.lua"), "z").unwrap();
        std::fs::write(root.join("p/readme.md"), "z").unwrap();
        let mut files: Vec<PathBuf> = list(std::slice::from_ref(&root)).into_iter().map(|(p, _)| p).collect();
        files.sort();
        assert_eq!(files, [root.join("p/lib/a.lua"), root.join("p/main.lua")], "hidden folders left out");
        let _ = std::fs::remove_dir_all(&root);
    }
}
