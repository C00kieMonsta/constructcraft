//! Running an external program for a plugin (`ccad.process.run`), on a
//! background thread.
//!
//! Each run gets a fresh folder under the system's temporary folder
//! (`constructcraft-<plugin>-<random>`), created only for it: the input files
//! are written and copied there, the program runs there (its working
//! folder), the outputs are read back from there, and the folder is deleted
//! afterwards, also when the run fails, times out or is cancelled.
//!
//! - **No shell:** the program is started directly with its arguments, each
//!   passed as it is (`&`, `;`, `$(…)` and quotes mean nothing).
//! - **Environment:** only `PATH`, `SystemRoot`, `TEMP`, `TMP`, `HOME`,
//!   `USERPROFILE`, `LANG` and `LC_*` are passed on ([`ENV_ALLOWED`]); the
//!   rest of the app's environment is not.
//! - **Names** of input and output files are relative, `/`-separated plain
//!   names inside the run folder ([`check_name`]): no `..`, no absolute
//!   paths or drives, no device names such as `CON`.
//! - **Limits:** inputs at most [`MAX_INPUTS`] in all; each output at most
//!   [`MAX_OUTPUT`], all together [`MAX_OUTPUTS`]; standard output and error
//!   kept up to [`MAX_STREAM`] each (`truncated` beyond); a timeout of at
//!   most [`MAX_TIMEOUT`].
//! - **Stopping:** on timeout or cancel the program is killed
//!   ([`std::process::Child::kill`]); programs it started itself may go on
//!   running (no job objects or process groups are used).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Most bytes written and copied into the run folder, all inputs together.
pub const MAX_INPUTS: u64 = 256 * 1024 * 1024;
/// Most bytes of one output file read back.
pub const MAX_OUTPUT: u64 = 64 * 1024 * 1024;
/// Most bytes of all output files read back.
pub const MAX_OUTPUTS: u64 = 256 * 1024 * 1024;
/// Bytes of standard output, and of standard error, kept.
pub const MAX_STREAM: usize = 1024 * 1024;
/// The timeout when the plugin gives none.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
/// The longest timeout.
pub const MAX_TIMEOUT: Duration = Duration::from_secs(7200);
/// The environment variables passed on to the program (`LC_*` too).
pub const ENV_ALLOWED: [&str; 7] = ["PATH", "SystemRoot", "TEMP", "TMP", "HOME", "USERPROFILE", "LANG"];
/// Longest name of a file in the run folder, and of one of its parts.
const MAX_NAME: usize = 512;
const MAX_PART: usize = 128;
/// Deepest folder of the run folder looked into, and most entries looked at.
const MAX_DEPTH: usize = 8;
const MAX_ENTRIES: usize = 10_000;
/// How long the readers of the program's output get after it ended (a
/// program it started may keep the pipes open).
const READER_GRACE: Duration = Duration::from_secs(2);
/// Time between two checks of the running program.
const TICK: Duration = Duration::from_millis(20);
/// Longest last line kept for the progress dialog, in characters.
const MAX_LAST_LINE: usize = 200;

/// A program to run, checked ([`check_name`], sizes) by the caller.
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The plugin's id (names the run folder).
    pub plugin: String,
    /// The program's absolute path.
    pub program: PathBuf,
    pub args: Vec<String>,
    /// Files to write into the run folder first: name, content.
    pub files: Vec<(String, Vec<u8>)>,
    /// Files to copy into the run folder: name, source.
    pub copies: Vec<(String, PathBuf)>,
    /// Names of files to read back, with `*` wildcards.
    pub outputs: Vec<String>,
    pub timeout: Duration,
}

/// How a run ended.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Finished {
    /// The exit code (`None` when the program was killed or ended by a signal).
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// Standard output or error went past [`MAX_STREAM`].
    pub truncated: bool,
    /// The timeout ended it.
    pub timed_out: bool,
    /// The output files found: name (`/`-separated), content.
    pub files: Vec<(String, Vec<u8>)>,
}

/// A program running on a background thread. Dropping it stops the program.
#[derive(Debug)]
pub struct Running {
    rx: Receiver<Result<Finished, String>>,
    cancel: Arc<AtomicBool>,
    last_line: Arc<Mutex<String>>,
}

impl Running {
    /// Start `request` on a background thread (an error that keeps it from
    /// starting comes back from [`Running::wait`]).
    pub fn start(request: Request) -> Running {
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let last_line = Arc::new(Mutex::new(String::new()));
        let (stop, line) = (cancel.clone(), last_line.clone());
        let started = std::thread::Builder::new().name("constructcraft-process".into()).spawn(move || {
            let outcome = run(&request, &stop, &line);
            let _ = tx.send(outcome);
        });
        if let Err(e) = started {
            let (tx, failed) = mpsc::channel();
            let _ = tx.send(Err(format!("could not start a thread for the program: {e}")));
            return Running { rx: failed, cancel, last_line };
        }
        Running { rx, cancel, last_line }
    }

    /// How it ended, waiting at most `wait`; `None` while it runs.
    pub fn wait(&self, wait: Duration) -> Option<Result<Finished, String>> {
        match self.rx.recv_timeout(wait) {
            Ok(outcome) => Some(outcome),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => Some(Err("the program's thread ended without a result".into())),
        }
    }

    /// Stop the program and wait (a few seconds at most) until its run
    /// folder is deleted.
    pub fn cancel(self) {
        self.cancel.store(true, Ordering::Relaxed);
        let _ = self.rx.recv_timeout(READER_GRACE + Duration::from_secs(8));
    }

    /// The last line the program wrote (standard output or error).
    pub fn last_line(&self) -> String {
        self.last_line.lock().map(|l| l.clone()).unwrap_or_default()
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        // A run nobody waits for any more is stopped (its thread cleans up).
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Run `program` with `args` (no files), waiting at most `timeout`: for a
/// version check.
pub fn run_now(plugin: &str, program: &Path, args: &[String], timeout: Duration) -> Result<Finished, String> {
    let request = Request {
        plugin: plugin.to_string(),
        program: program.to_path_buf(),
        args: args.to_vec(),
        timeout,
        ..Request::default()
    };
    let running = Running::start(request);
    // The run's own timeout ends it; this only guards against a lost thread.
    running.wait(timeout + READER_GRACE + Duration::from_secs(10)).unwrap_or_else(|| Err("no answer".into()))
}

/// Whether `name` is a device name of Windows (`CON`, `COM1`…), with or
/// without an extension, in any case.
fn device_name(part: &str) -> bool {
    let stem = part.split('.').next().unwrap_or(part).trim_end().to_ascii_uppercase();
    match stem.as_str() {
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" => true,
        s if s.len() == 4 && (s.starts_with("COM") || s.starts_with("LPT")) => {
            s.chars().nth(3).is_some_and(|c| c.is_ascii_digit())
        }
        // COM¹, LPT² …: also devices.
        s => (s.starts_with("COM") || s.starts_with("LPT")) && s.chars().count() == 4 && s.ends_with(['¹', '²', '³']),
    }
}

/// Check the name of a file in the run folder: relative, made of plain
/// names joined by `/` (no `.` or `..`, no `\`, `:`, `<`, `>`, `"`, `|`, `?`
/// or control characters, not ending in a dot or space, no Windows device
/// name), at most [`MAX_DEPTH`] deep; `*` only when `wildcard`. Returns its
/// parts.
pub fn check_name(name: &str, wildcard: bool) -> Result<Vec<String>, String> {
    let bad = |why: &str| Err(format!("\"{name}\": {why}"));
    if name.is_empty() {
        return bad("an empty name");
    }
    if name.len() > MAX_NAME {
        return bad(&format!("longer than {MAX_NAME} bytes"));
    }
    if name.starts_with('/') {
        return bad("an absolute path (names are relative to the run folder)");
    }
    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() > MAX_DEPTH {
        return bad(&format!("more than {MAX_DEPTH} levels deep"));
    }
    for part in &parts {
        match *part {
            "" => return bad("an empty part (two `/` in a row, or one at the end)"),
            "." | ".." => return bad("`.` and `..` are not allowed"),
            _ => {}
        }
        if part.len() > MAX_PART {
            return bad(&format!("a part longer than {MAX_PART} bytes"));
        }
        if let Some(c) = part.chars().find(|c| {
            c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?') || (*c == '*' && !wildcard)
        }) {
            return bad(&format!("the character {c:?} is not allowed"));
        }
        if part.ends_with('.') || part.ends_with(' ') {
            return bad("a part ends with a dot or a space");
        }
        if device_name(part) {
            return bad("a device name (CON, NUL, COM1…)");
        }
    }
    Ok(parts.into_iter().map(str::to_string).collect())
}

/// Whether `text` matches `pattern`, whose `*` stands for any run of
/// characters (case-insensitive on Windows).
fn matches(pattern: &str, text: &str) -> bool {
    let fold = |s: &str| if cfg!(windows) { s.to_lowercase() } else { s.to_string() };
    let (pattern, text) = (fold(pattern), fold(text));
    let pieces: Vec<&str> = pattern.split('*').collect();
    let [first, middle @ .., last] = pieces.as_slice() else { return pattern == text };
    let Some(mut rest) = text.strip_prefix(first) else { return false };
    for piece in middle {
        match rest.find(piece) {
            Some(i) => rest = rest.get(i + piece.len()..).unwrap_or_default(),
            None => return false,
        }
    }
    rest.len() >= last.len() && rest.ends_with(last)
}

/// A random suffix for the run folder.
fn random_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut h = RandomState::new().build_hasher();
    h.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    h.write_u32(std::process::id());
    if let Ok(t) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        h.write_u128(t.as_nanos());
    }
    format!("{:016x}", h.finish())
}

/// The folder of a run: created here, deleted when dropped.
struct RunFolder(PathBuf);

impl RunFolder {
    fn create(plugin: &str) -> Result<RunFolder, String> {
        let temp = std::env::temp_dir();
        let mut last = String::new();
        for _ in 0..16 {
            let dir = temp.join(format!("constructcraft-{plugin}-{}", random_suffix()));
            // `create_dir`, not `create_dir_all`: a folder that exists is not taken.
            match std::fs::create_dir(&dir) {
                Ok(()) => return Ok(RunFolder(dir)),
                Err(e) => last = e.to_string(),
            }
        }
        Err(format!("could not create a run folder in {}: {last}", temp.display()))
    }
}

impl Drop for RunFolder {
    fn drop(&mut self) {
        // A killed program may hold its files for a moment (Windows).
        for _ in 0..20 {
            if std::fs::remove_dir_all(&self.0).is_ok() || !self.0.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // Left behind only when something (a program the program started)
        // still holds it; the system's temporary folder is cleaned in time.
    }
}

/// `parts` under `root`, creating its folders.
fn inside(root: &Path, name: &str) -> Result<PathBuf, String> {
    let parts = check_name(name, false)?;
    let mut path = root.to_path_buf();
    for part in &parts {
        path.push(part);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(path)
}

/// Write and copy the inputs into the run folder, at most [`MAX_INPUTS`].
fn write_inputs(root: &Path, request: &Request) -> Result<(), String> {
    let mut total: u64 = 0;
    let too_much = || format!("the input files take more than {} MiB", MAX_INPUTS >> 20);
    for (name, bytes) in &request.files {
        total = total.saturating_add(bytes.len() as u64);
        if total > MAX_INPUTS {
            return Err(too_much());
        }
        let path = inside(root, name)?;
        std::fs::write(&path, bytes).map_err(|e| format!("{name}: {e}"))?;
    }
    for (name, source) in &request.copies {
        let path = inside(root, name)?;
        let shown = source.display();
        let from = std::fs::File::open(source).map_err(|e| format!("{shown}: {e}"))?;
        let room = MAX_INPUTS.saturating_sub(total);
        let mut to = std::fs::File::create(&path).map_err(|e| format!("{name}: {e}"))?;
        let copied =
            std::io::copy(&mut from.take(room.saturating_add(1)), &mut to).map_err(|e| format!("{shown}: {e}"))?;
        total = total.saturating_add(copied);
        if total > MAX_INPUTS {
            return Err(too_much());
        }
        to.flush().map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

/// The environment passed on to the program.
fn allowed_env() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    std::env::vars_os()
        .filter(|(key, _)| {
            let Some(key) = key.to_str() else { return false };
            let same = |a: &str| if cfg!(windows) { key.eq_ignore_ascii_case(a) } else { key == a };
            ENV_ALLOWED.iter().any(|a| same(a)) || key.starts_with("LC_")
        })
        .collect()
}

/// What a reader thread keeps of one stream.
#[derive(Debug, Default)]
struct Capture {
    bytes: Vec<u8>,
    truncated: bool,
    done: bool,
}

/// Read `stream` into `capture` (at most [`MAX_STREAM`] bytes, the rest
/// read and dropped), noting its last line in `last_line`.
fn read_stream(mut stream: impl Read, capture: &Mutex<Capture>, last_line: &Mutex<String>) {
    let mut buf = [0u8; 8192];
    let mut line = Vec::new();
    loop {
        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let chunk = buf.get(..n).unwrap_or_default();
        if let Ok(mut c) = capture.lock() {
            let room = MAX_STREAM.saturating_sub(c.bytes.len());
            c.bytes.extend_from_slice(chunk.get(..room.min(n)).unwrap_or_default());
            if n > room {
                c.truncated = true;
            }
        }
        for &b in chunk {
            if b == b'\n' || b == b'\r' {
                let text = String::from_utf8_lossy(&line).trim().to_string();
                if !text.is_empty()
                    && let Ok(mut last) = last_line.lock()
                {
                    *last = text.chars().take(MAX_LAST_LINE).collect();
                }
                line.clear();
            } else if line.len() < MAX_LAST_LINE * 4 {
                line.push(b);
            }
        }
    }
    let text = String::from_utf8_lossy(&line).trim().to_string();
    if !text.is_empty()
        && let Ok(mut last) = last_line.lock()
    {
        *last = text.chars().take(MAX_LAST_LINE).collect();
    }
    if let Ok(mut c) = capture.lock() {
        c.done = true;
    }
}

/// Take what a reader kept, waiting until `until` for it to finish.
fn take_capture(capture: &Mutex<Capture>, until: Instant) -> (Vec<u8>, bool) {
    loop {
        let done = capture.lock().map(|c| c.done).unwrap_or(true);
        if done || Instant::now() >= until {
            break;
        }
        std::thread::sleep(TICK);
    }
    match capture.lock() {
        Ok(mut c) => (std::mem::take(&mut c.bytes), c.truncated),
        Err(_) => (Vec::new(), false),
    }
}

/// The files of the run folder whose names match `patterns`, read.
fn read_outputs(root: &Path, patterns: &[String]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let patterns: Vec<Vec<String>> = patterns.iter().map(|p| check_name(p, true)).collect::<Result<_, _>>()?;
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    let mut entries = 0usize;
    walk(root, &mut Vec::new(), &mut entries, &mut |parts, path| {
        let hit = patterns
            .iter()
            .any(|p| p.len() == parts.len() && p.iter().zip(parts).all(|(pat, part)| matches(pat, part)));
        if hit {
            found.push((parts.join("/"), path.to_path_buf()));
        }
    });
    found.sort();
    let mut total: u64 = 0;
    let mut out = Vec::new();
    for (name, path) in found {
        let file = std::fs::File::open(&path).map_err(|e| format!("output {name}: {e}"))?;
        let mut bytes = Vec::new();
        file.take(MAX_OUTPUT + 1).read_to_end(&mut bytes).map_err(|e| format!("output {name}: {e}"))?;
        if bytes.len() as u64 > MAX_OUTPUT {
            return Err(format!("the output {name} is larger than {} MiB", MAX_OUTPUT >> 20));
        }
        total = total.saturating_add(bytes.len() as u64);
        if total > MAX_OUTPUTS {
            return Err(format!("the outputs take more than {} MiB", MAX_OUTPUTS >> 20));
        }
        out.push((name, bytes));
    }
    Ok(out)
}

/// Call `f` with each regular file under `dir` (not following links), at
/// most [`MAX_DEPTH`] deep and [`MAX_ENTRIES`] entries in all.
fn walk(dir: &Path, parts: &mut Vec<String>, entries: &mut usize, f: &mut dyn FnMut(&[String], &Path)) {
    if parts.len() >= MAX_DEPTH {
        return;
    }
    let Ok(list) = std::fs::read_dir(dir) else { return };
    for entry in list.flatten() {
        *entries += 1;
        if *entries > MAX_ENTRIES {
            return;
        }
        let Ok(kind) = entry.file_type() else { continue };
        let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
        parts.push(name);
        if kind.is_file() {
            f(parts, &entry.path());
        } else if kind.is_dir() {
            walk(&entry.path(), parts, entries, f);
        }
        parts.pop();
    }
}

/// Run `request`: create its folder, write the inputs, run the program
/// until it ends, times out or `cancel` is set, read the outputs, delete the
/// folder.
fn run(request: &Request, cancel: &AtomicBool, last_line: &Arc<Mutex<String>>) -> Result<Finished, String> {
    let folder = RunFolder::create(&request.plugin)?;
    let root = folder.0.clone();
    write_inputs(&root, request)?;
    let mut command = Command::new(&request.program);
    command
        .args(&request.args)
        .current_dir(&root)
        .env_clear()
        .envs(allowed_env())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        // No console window over the app.
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|e| format!("{} could not start: {e}", request.program.display()))?;
    let out = Arc::new(Mutex::new(Capture::default()));
    let err = Arc::new(Mutex::new(Capture::default()));
    if let Some(stream) = child.stdout.take() {
        let (capture, line) = (out.clone(), last_line.clone());
        let _ = std::thread::Builder::new()
            .name("process-stdout".into())
            .spawn(move || read_stream(stream, &capture, &line));
    } else if let Ok(mut c) = out.lock() {
        c.done = true;
    }
    if let Some(stream) = child.stderr.take() {
        let (capture, line) = (err.clone(), last_line.clone());
        let _ = std::thread::Builder::new()
            .name("process-stderr".into())
            .spawn(move || read_stream(stream, &capture, &line));
    } else if let Ok(mut c) = err.lock() {
        c.done = true;
    }
    let started = Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(_) => break None,
        }
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
        } else if started.elapsed() >= request.timeout {
            timed_out = true;
        }
        if cancelled || timed_out {
            let _ = child.kill();
            break child.wait().ok();
        }
        std::thread::sleep(TICK);
    };
    let until = Instant::now() + READER_GRACE;
    let (stdout, out_cut) = take_capture(&out, until);
    let (stderr, err_cut) = take_capture(&err, until);
    if cancelled {
        return Err("cancelled".into());
    }
    let files = read_outputs(&root, &request.outputs)?;
    drop(folder);
    Ok(Finished {
        code: if timed_out { None } else { status.and_then(|s| s.code()) },
        stdout,
        stderr,
        truncated: out_cut || err_cut,
        timed_out,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_stay_inside_the_run_folder() {
        for good in ["in.idf", "sub/in.epw", "a-b_c.1.txt", "deep/er/file"] {
            assert!(check_name(good, false).is_ok(), "{good}");
        }
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "a/../b",
            "./a",
            "a//b",
            "a/",
            "C:/x",
            "C:x",
            "a\\b",
            "CON",
            "con.txt",
            "nul",
            "COM1",
            "lpt9.log",
            "x?",
            "a*",
            "trail.",
            "trail ",
            "\u{1}",
        ] {
            assert!(check_name(bad, false).is_err(), "{bad:?}");
        }
        assert!(check_name("*.err", true).is_ok());
        assert!(check_name("a/".repeat(9).trim_end_matches('/'), false).is_err(), "too deep");
        assert!(check_name("COM10", false).is_ok() && check_name("console", false).is_ok());
    }

    #[test]
    fn wildcards_match_runs_of_characters() {
        assert!(matches("*.err", "eplusout.err"));
        assert!(matches("eplus*.c*v", "eplusout.csv"));
        assert!(matches("*", "x"));
        assert!(matches("a*a", "aa"));
        assert!(!matches("a*a", "a"));
        assert!(!matches("*.err", "eplusout.err.txt"));
        assert!(matches("exact.txt", "exact.txt"));
        assert!(!matches("exact.txt", "exact.txt2"));
    }

    #[test]
    fn the_environment_is_an_allowlist() {
        let env = allowed_env();
        let names: Vec<String> = env.iter().map(|(k, _)| k.to_string_lossy().to_uppercase()).collect();
        assert!(names.iter().any(|n| n == "PATH"), "{names:?}");
        assert!(names.iter().all(|n| ENV_ALLOWED.iter().any(|a| a.to_uppercase() == *n) || n.starts_with("LC_")));
    }
}
