//! A small program the tests of `ccad.process.run` run as an external tool
//! (not part of ConstructCraft). Its arguments are steps, run in order:
//!
//! - `--version`: print `cc_plugins_test_tool 1.0`
//! - `args <a>…`: print the remaining arguments as a JSON array, and stop
//! - `out <text>` / `err <text>`: print a line to standard output / error
//! - `write <name> <text>`: write a file (creating its folders)
//! - `copy <from> <to>`: copy a file
//! - `sleep <ms>`: wait
//! - `spam <n>`: write `n` bytes to standard output
//! - `big <name> <n>`: write a file of `n` bytes
//! - `lines <n> <ms>`: print `line 1` … `line n`, waiting `ms` after each
//! - `cwd`: print the working folder
//! - `env`: print the names of the environment variables, sorted, one a line
//! - `exit <code>`: end with that exit code

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

fn arg(it: &mut impl Iterator<Item = String>, step: &str) -> Result<String, String> {
    it.next().ok_or_else(|| format!("{step} needs an argument"))
}

fn number(it: &mut impl Iterator<Item = String>, step: &str) -> Result<u64, String> {
    arg(it, step)?.parse().map_err(|e| format!("{step}: {e}"))
}

fn run() -> Result<u8, String> {
    let mut it = std::env::args().skip(1);
    let mut stdout = std::io::stdout();
    let io = |e: std::io::Error| e.to_string();
    while let Some(step) = it.next() {
        match step.as_str() {
            "--version" => println!("cc_plugins_test_tool 1.0"),
            "args" => {
                let rest: Vec<String> = it.by_ref().collect();
                println!("{}", serde_json::to_string(&rest).map_err(|e| e.to_string())?);
            }
            "out" => println!("{}", arg(&mut it, "out")?),
            "err" => eprintln!("{}", arg(&mut it, "err")?),
            "write" => {
                let (name, text) = (arg(&mut it, "write")?, arg(&mut it, "write")?);
                let path = std::path::Path::new(&name);
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent).map_err(io)?;
                }
                std::fs::write(path, text).map_err(io)?;
            }
            "copy" => {
                let (from, to) = (arg(&mut it, "copy")?, arg(&mut it, "copy")?);
                std::fs::copy(from, to).map_err(io)?;
            }
            "sleep" => std::thread::sleep(Duration::from_millis(number(&mut it, "sleep")?)),
            "spam" => {
                let n = number(&mut it, "spam")?;
                let chunk = [b'x'; 4096];
                let mut left = n;
                while left > 0 {
                    let take = left.min(chunk.len() as u64) as usize;
                    stdout.write_all(chunk.get(..take).unwrap_or_default()).map_err(io)?;
                    left -= take as u64;
                }
                stdout.flush().map_err(io)?;
            }
            "big" => {
                let (name, n) = (arg(&mut it, "big")?, number(&mut it, "big")?);
                let file = std::fs::File::create(name).map_err(io)?;
                file.set_len(n).map_err(io)?;
            }
            "lines" => {
                let (n, ms) = (number(&mut it, "lines")?, number(&mut it, "lines")?);
                for i in 1..=n {
                    writeln!(stdout, "line {i}").map_err(io)?;
                    stdout.flush().map_err(io)?;
                    std::thread::sleep(Duration::from_millis(ms));
                }
            }
            "cwd" => println!("{}", std::env::current_dir().map_err(io)?.display()),
            "env" => {
                let mut names: Vec<String> =
                    std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
                names.sort();
                for name in names {
                    println!("{name}");
                }
            }
            "exit" => {
                let code = number(&mut it, "exit")?;
                return Ok(u8::try_from(code).unwrap_or(u8::MAX));
            }
            other => return Err(format!("unknown step {other}")),
        }
    }
    Ok(0)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("cc_plugins_test_tool: {message}");
            ExitCode::from(101)
        }
    }
}
