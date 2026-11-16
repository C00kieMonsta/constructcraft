//! Writes the API reference of the workspace's crates as Markdown pages for the site.
//!
//! `cargo run -p api_doc` reads every crate under `crates/` (from its `lib.rs` or
//! `main.rs`, following `mod` declarations, leaving `#[cfg(test)]` modules out) and writes
//! `docs/api/<crate>.md`: per module its documentation, then each item that is not private
//! (functions, structs, enums, traits, type aliases, constants, statics, exported macros and
//! re-exports) with its signature and doc comment. A struct lists its visible fields, an enum
//! its variants, a type the methods of its inherent impls (wherever in the crate they are) and
//! the traits it derives or implements, and a trait its implementors across the workspace.
//! Rust has no classes or inheritance: types plus traits are what this reference describes.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use proc_macro2::LineColumn;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Fields, ImplItem, Item, Lit, Meta, TraitItem, Visibility};

/// A documented member of an item: a field, a variant or a method.
struct Member {
    sig: String,
    doc: String,
    /// The module of the impl block holding a method, when it is not the type's own module.
    from: Option<String>,
}

#[derive(Default)]
struct Entry {
    kind: &'static str,
    name: String,
    sig: String,
    doc: String,
    /// Derived and implemented traits.
    traits: Vec<String>,
    fields: Vec<Member>,
    hidden_fields: bool,
    variants: Vec<Member>,
    methods: Vec<Member>,
    /// For traits: the types implementing them.
    implementors: Vec<String>,
}

struct Module {
    path: String,
    file: String,
    doc: String,
    entries: Vec<Entry>,
    reexports: Vec<String>,
    /// Inherent methods of types this reference does not show (private or not found).
    other: BTreeMap<String, Vec<Member>>,
}

struct Impl {
    module: usize,
    self_ty: String,
    trait_name: Option<String>,
    trait_text: Option<String>,
    methods: Vec<Member>,
    /// The implementing type as shown in lists: its path, linked to its entry when shown.
    owner: String,
}

struct Crate {
    name: String,
    doc: String,
    modules: Vec<Module>,
    impls: Vec<Impl>,
}

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(e) = run() {
        eprintln!("api_doc: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = args.first().map(PathBuf::from).unwrap_or_else(|| root.join("docs/api"));
    let mut names: Vec<String> = std::fs::read_dir(root.join("crates"))
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut crates = Vec::new();
    for name in names {
        let src = root.join("crates").join(&name).join("src");
        let main = ["lib.rs", "main.rs"].iter().map(|f| src.join(f)).find(|p| p.is_file());
        let Some(main) = main else { continue };
        let mut krate = Crate { name: name.clone(), doc: String::new(), modules: Vec::new(), impls: Vec::new() };
        walk_file(&mut krate, &root, &main, name.clone(), &src)?;
        krate.doc = krate.modules.first().map(|m| m.doc.clone()).unwrap_or_default();
        resolve(&mut krate);
        crates.push(krate);
    }
    link_traits(&mut crates);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for krate in &crates {
        let page = render(krate);
        let path = out.join(format!("{}.md", krate.name));
        std::fs::write(&path, page).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let index = render_index(&crates);
    std::fs::write(out.join("index.md"), index).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------- reading

/// The source of one file, sliced by the line and column of spans.
struct Src {
    lines: Vec<Vec<char>>,
}

impl Src {
    fn new(text: &str) -> Self {
        Src { lines: text.lines().map(|l| l.chars().collect()).collect() }
    }

    /// The text between two positions (lines from 1, columns in chars), comments dropped
    /// and whitespace folded into one line.
    fn slice(&self, start: LineColumn, end: LineColumn) -> String {
        let mut parts = Vec::new();
        for line in start.line..=end.line {
            let Some(chars) = self.lines.get(line.saturating_sub(1)) else { break };
            let from = if line == start.line { start.column } else { 0 };
            let to = if line == end.line { end.column.min(chars.len()) } else { chars.len() };
            let piece: String = chars.get(from..to.max(from)).map(|c| c.iter().collect()).unwrap_or_default();
            let piece = match piece.find("//") {
                Some(i) => piece[..i].to_string(),
                None => piece,
            };
            parts.push(piece);
        }
        tidy(&parts.join(" "))
    }
}

/// One line of folded whitespace, with the spaces and trailing commas of rustfmt's
/// multi-line layout removed.
fn tidy(text: &str) -> String {
    let mut s = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for (from, to) in [("( ", "("), (", )", ")"), (" )", ")"), ("< ", "<"), (", >", ">"), (",)", ")"), (",>", ">")] {
        while s.contains(from) {
            s = s.replace(from, to);
        }
    }
    s.trim_end_matches(',').trim().to_string()
}

fn later(a: LineColumn, b: LineColumn) -> LineColumn {
    if (b.line, b.column) > (a.line, a.column) { b } else { a }
}

fn walk_file(krate: &mut Crate, root: &Path, path: &Path, module: String, dir: &Path) -> Result<()> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let src = Src::new(&text);
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    let rel = rel.trim_start_matches("./").to_string();
    let idx = krate.modules.len();
    krate.modules.push(Module {
        path: module.clone(),
        file: rel.clone(),
        doc: doc_of(&file.attrs),
        entries: Vec::new(),
        reexports: Vec::new(),
        other: BTreeMap::new(),
    });
    items(krate, root, &src, &file.items, idx, dir, &rel)
}

fn items(krate: &mut Crate, root: &Path, src: &Src, list: &[Item], m: usize, dir: &Path, file: &str) -> Result<()> {
    let module = krate.modules.get(m).map(|x| x.path.clone()).unwrap_or_default();
    for item in list {
        match item {
            Item::Mod(im) => {
                if is_test(&im.attrs) {
                    continue;
                }
                let name = im.ident.to_string();
                let path = format!("{module}::{name}");
                let child_dir = dir.join(&name);
                if let Some((_, content)) = &im.content {
                    let idx = krate.modules.len();
                    krate.modules.push(Module {
                        path,
                        file: file.to_string(),
                        doc: doc_of(&im.attrs),
                        entries: Vec::new(),
                        reexports: Vec::new(),
                        other: BTreeMap::new(),
                    });
                    items(krate, root, src, content, idx, &child_dir, file)?;
                } else {
                    let flat = dir.join(format!("{name}.rs"));
                    let nested = child_dir.join("mod.rs");
                    let target = if flat.is_file() { flat } else { nested };
                    if target.is_file() {
                        walk_file(krate, root, &target, path, &child_dir)?;
                    }
                }
            }
            Item::Fn(f) if shown(&f.vis) && !is_test(&f.attrs) => {
                let sig = src.slice(f.vis.span().start(), f.sig.span().end());
                push(
                    krate,
                    m,
                    Entry {
                        kind: "fn",
                        name: f.sig.ident.to_string(),
                        sig,
                        doc: doc_of(&f.attrs),
                        ..Default::default()
                    },
                );
            }
            Item::Struct(s) if shown(&s.vis) => {
                let mut end = s.ident.span().end();
                if s.generics.lt_token.is_some() {
                    end = later(end, s.generics.span().end());
                }
                if let Fields::Unnamed(u) = &s.fields {
                    end = later(end, u.span().end());
                }
                if let Some(w) = &s.generics.where_clause {
                    end = later(end, w.span().end());
                }
                let mut entry = Entry {
                    kind: "struct",
                    name: s.ident.to_string(),
                    sig: src.slice(s.vis.span().start(), end),
                    doc: doc_of(&s.attrs),
                    traits: derives(&s.attrs),
                    ..Default::default()
                };
                if let Fields::Named(named) = &s.fields {
                    for f in &named.named {
                        if !shown(&f.vis) {
                            entry.hidden_fields = true;
                            continue;
                        }
                        let Some(ident) = &f.ident else { continue };
                        entry.fields.push(Member {
                            sig: src.slice(ident.span().start(), f.ty.span().end()),
                            doc: doc_of(&f.attrs),
                            from: None,
                        });
                    }
                }
                push(krate, m, entry);
            }
            Item::Enum(e) if shown(&e.vis) => {
                let mut end = e.ident.span().end();
                if e.generics.lt_token.is_some() {
                    end = later(end, e.generics.span().end());
                }
                if let Some(w) = &e.generics.where_clause {
                    end = later(end, w.span().end());
                }
                let mut entry = Entry {
                    kind: "enum",
                    name: e.ident.to_string(),
                    sig: src.slice(e.vis.span().start(), end),
                    doc: doc_of(&e.attrs),
                    traits: derives(&e.attrs),
                    ..Default::default()
                };
                for v in &e.variants {
                    let mut end = v.ident.span().end();
                    if !matches!(v.fields, Fields::Unit) {
                        end = later(end, v.fields.span().end());
                    }
                    if let Some((_, expr)) = &v.discriminant {
                        end = later(end, expr.span().end());
                    }
                    entry.variants.push(Member {
                        sig: src.slice(v.ident.span().start(), end),
                        doc: doc_of(&v.attrs),
                        from: None,
                    });
                }
                push(krate, m, entry);
            }
            Item::Union(u) if shown(&u.vis) => {
                let sig = src.slice(u.vis.span().start(), u.ident.span().end());
                push(
                    krate,
                    m,
                    Entry {
                        kind: "union",
                        name: u.ident.to_string(),
                        sig,
                        doc: doc_of(&u.attrs),
                        ..Default::default()
                    },
                );
            }
            Item::Trait(t) if shown(&t.vis) => {
                let mut end = t.ident.span().end();
                if t.generics.lt_token.is_some() {
                    end = later(end, t.generics.span().end());
                }
                if !t.supertraits.is_empty() {
                    end = later(end, t.supertraits.span().end());
                }
                if let Some(w) = &t.generics.where_clause {
                    end = later(end, w.span().end());
                }
                let mut entry = Entry {
                    kind: "trait",
                    name: t.ident.to_string(),
                    sig: src.slice(t.vis.span().start(), end),
                    doc: doc_of(&t.attrs),
                    ..Default::default()
                };
                for ti in &t.items {
                    let member = match ti {
                        TraitItem::Fn(f) => Some((src.slice(f.sig.span().start(), f.sig.span().end()), &f.attrs)),
                        TraitItem::Const(c) => {
                            Some((src.slice(c.const_token.span().start(), c.ty.span().end()), &c.attrs))
                        }
                        TraitItem::Type(ty) => {
                            let mut end = ty.ident.span().end();
                            if !ty.bounds.is_empty() {
                                end = later(end, ty.bounds.span().end());
                            }
                            Some((src.slice(ty.type_token.span().start(), end), &ty.attrs))
                        }
                        _ => None,
                    };
                    if let Some((sig, attrs)) = member {
                        entry.methods.push(Member { sig, doc: doc_of(attrs), from: None });
                    }
                }
                push(krate, m, entry);
            }
            Item::Type(t) if shown(&t.vis) => {
                let sig = src.slice(t.vis.span().start(), t.ty.span().end());
                push(
                    krate,
                    m,
                    Entry { kind: "type", name: t.ident.to_string(), sig, doc: doc_of(&t.attrs), ..Default::default() },
                );
            }
            Item::Const(c) if shown(&c.vis) => {
                let sig = src.slice(c.vis.span().start(), c.ty.span().end());
                push(
                    krate,
                    m,
                    Entry {
                        kind: "const",
                        name: c.ident.to_string(),
                        sig,
                        doc: doc_of(&c.attrs),
                        ..Default::default()
                    },
                );
            }
            Item::Static(s) if shown(&s.vis) => {
                let sig = src.slice(s.vis.span().start(), s.ty.span().end());
                push(
                    krate,
                    m,
                    Entry {
                        kind: "static",
                        name: s.ident.to_string(),
                        sig,
                        doc: doc_of(&s.attrs),
                        ..Default::default()
                    },
                );
            }
            Item::Macro(mac) => {
                let exported = mac.attrs.iter().any(|a| a.path().is_ident("macro_export"));
                if let (true, Some(ident)) = (exported, &mac.ident) {
                    let sig = format!("macro_rules! {ident}");
                    push(
                        krate,
                        m,
                        Entry {
                            kind: "macro",
                            name: ident.to_string(),
                            sig,
                            doc: doc_of(&mac.attrs),
                            ..Default::default()
                        },
                    );
                }
            }
            Item::Use(u) if shown(&u.vis) => {
                let text = src.slice(u.vis.span().start(), u.tree.span().end());
                if let Some(module) = krate.modules.get_mut(m) {
                    module.reexports.push(text);
                }
            }
            Item::Impl(imp) if !is_test(&imp.attrs) => {
                let self_ty = match &*imp.self_ty {
                    syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
                    other => Some(src.slice(other.span().start(), other.span().end())),
                };
                let Some(self_ty) = self_ty else { continue };
                let (trait_name, trait_text) = match &imp.trait_ {
                    Some((bang, path, _)) => {
                        let name = path.segments.last().map(|s| s.ident.to_string());
                        let text = src.slice(path.span().start(), path.span().end());
                        let text = if bang.is_some() { format!("!{text}") } else { text };
                        (name, Some(text))
                    }
                    None => (None, None),
                };
                let mut methods = Vec::new();
                if trait_text.is_none() {
                    for ii in &imp.items {
                        match ii {
                            ImplItem::Fn(f) if shown(&f.vis) => methods.push(Member {
                                sig: src.slice(f.vis.span().start(), f.sig.span().end()),
                                doc: doc_of(&f.attrs),
                                from: None,
                            }),
                            ImplItem::Const(c) if shown(&c.vis) => methods.push(Member {
                                sig: src.slice(c.vis.span().start(), c.ty.span().end()),
                                doc: doc_of(&c.attrs),
                                from: None,
                            }),
                            _ => {}
                        }
                    }
                }
                let owner = format!("`{module}::{self_ty}`");
                krate.impls.push(Impl { module: m, self_ty, trait_name, trait_text, methods, owner });
            }
            _ => {}
        }
    }
    Ok(())
}

fn push(krate: &mut Crate, m: usize, entry: Entry) {
    if let Some(module) = krate.modules.get_mut(m) {
        module.entries.push(entry);
    }
}

fn shown(vis: &Visibility) -> bool {
    !matches!(vis, Visibility::Inherited)
}

/// Whether the attributes put the item under `cfg(test)`.
fn is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && match &a.meta {
                Meta::List(l) => {
                    l.tokens.to_string().split(|c: char| !c.is_alphanumeric() && c != '_').any(|w| w == "test")
                }
                _ => false,
            }
    })
}

fn derives(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.path().is_ident("derive")) {
        if let Meta::List(l) = &a.meta {
            for part in l.tokens.to_string().split(',') {
                let name = part.rsplit("::").next().unwrap_or(part).trim().to_string();
                if !name.is_empty() {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// The doc comment of an item, with the one space after `///` removed, the hidden lines of
/// Rust code blocks dropped and intra-doc links turned into plain code.
fn doc_of(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for a in attrs.iter().filter(|a| a.path().is_ident("doc")) {
        if let Meta::NameValue(nv) = &a.meta
            && let Expr::Lit(lit) = &nv.value
            && let Lit::Str(s) = &lit.lit
        {
            for line in s.value().lines() {
                lines.push(line.strip_prefix(' ').unwrap_or(line).to_string());
            }
        }
    }
    let mut out = Vec::new();
    let mut in_rust = false;
    for line in lines {
        let t = line.trim_start();
        if let Some(lang) = t.strip_prefix("```") {
            if in_rust {
                in_rust = false;
            } else {
                let lang = lang.trim();
                in_rust = lang.is_empty()
                    || lang.starts_with("rust")
                    || lang.starts_with("no_run")
                    || lang.starts_with("ignore");
                if in_rust {
                    out.push("```rust".to_string());
                    continue;
                }
            }
            out.push(line);
            continue;
        }
        if in_rust && (t == "#" || t.starts_with("# ")) {
            continue;
        }
        if !in_rust && starts_list(&line) {
            // Markdown on the site needs a blank line between a paragraph and a list.
            let after_text =
                out.last().is_some_and(|p: &String| !p.trim().is_empty() && !starts_list(p) && !p.starts_with(' '));
            if after_text {
                out.push(String::new());
            }
        }
        out.push(if in_rust { line } else { plain_links(&line) });
    }
    out.join("\n").trim().to_string()
}

/// Whether a line starts a Markdown list item.
fn starts_list(line: &str) -> bool {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    line.starts_with("- ") || line.starts_with("* ") || (digits > 0 && line[digits..].starts_with(". "))
}

/// `[`Foo`]` (an intra-doc link) becomes `` `Foo` ``; links with a target are kept.
fn plain_links(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find("[`") {
        let after = &rest[i + 2..];
        let Some(j) = after.find("`]") else { break };
        let tail = &after[j + 2..];
        if tail.starts_with('(') || tail.starts_with('[') {
            out.push_str(&rest[..i + 2 + j + 2]);
        } else {
            out.push_str(&rest[..i]);
            out.push('`');
            out.push_str(&after[..j]);
            out.push('`');
        }
        rest = tail;
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------- linking

const TYPE_KINDS: [&str; 5] = ["struct", "enum", "union", "trait", "type"];

/// Attaches each impl block's methods and trait to the type it is for.
fn resolve(krate: &mut Crate) {
    let mut by_name: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (mi, module) in krate.modules.iter().enumerate() {
        for (ei, e) in module.entries.iter().enumerate() {
            if TYPE_KINDS.contains(&e.kind) {
                by_name.entry(e.name.clone()).or_default().push((mi, ei));
            }
        }
    }
    let impls = std::mem::take(&mut krate.impls);
    for mut imp in impls {
        let found = by_name.get(&imp.self_ty).and_then(|c| {
            c.iter().find(|(mi, _)| *mi == imp.module).or(if c.len() == 1 { c.first() } else { None }).copied()
        });
        let here = krate.modules.get(imp.module).map(|m| m.path.clone()).unwrap_or_default();
        match found {
            Some((mi, ei)) => {
                let same = mi == imp.module;
                if let Some(module) = krate.modules.get(mi) {
                    let path = format!("{}::{}", module.path, imp.self_ty);
                    imp.owner = format!("[`{path}`]({}.md#{})", krate.name, anchor(&path));
                }
                let Some(entry) = krate.modules.get_mut(mi).and_then(|m| m.entries.get_mut(ei)) else { continue };
                if let Some(t) = &imp.trait_text
                    && !entry.traits.contains(t)
                {
                    entry.traits.push(t.clone());
                }
                for mut method in imp.methods.drain(..) {
                    if !same {
                        method.from = Some(here.clone());
                    }
                    entry.methods.push(method);
                }
            }
            None if !imp.methods.is_empty() => {
                if let Some(module) = krate.modules.get_mut(imp.module) {
                    module.other.entry(imp.self_ty.clone()).or_default().append(&mut imp.methods);
                }
            }
            None => {}
        }
        if imp.trait_name.is_some() {
            krate.impls.push(imp);
        }
    }
}

/// Lists the implementors of each trait defined in the workspace.
fn link_traits(crates: &mut [Crate]) {
    let mut found: Vec<(String, String, String)> = Vec::new(); // (trait crate pick, trait, implementor)
    for krate in crates.iter() {
        for imp in &krate.impls {
            if let Some(t) = &imp.trait_name {
                found.push((krate.name.clone(), t.clone(), imp.owner.clone()));
            }
        }
    }
    for (from, trait_name, implementor) in found {
        let mut spots = Vec::new();
        for (ci, krate) in crates.iter().enumerate() {
            for (mi, module) in krate.modules.iter().enumerate() {
                for (ei, e) in module.entries.iter().enumerate() {
                    if e.kind == "trait" && e.name == trait_name {
                        spots.push((ci, mi, ei, krate.name == from));
                    }
                }
            }
        }
        let pick = spots.iter().find(|s| s.3).or(if spots.len() == 1 { spots.first() } else { None }).copied();
        if let Some((ci, mi, ei, _)) = pick
            && let Some(e) = crates.get_mut(ci).and_then(|c| c.modules.get_mut(mi)).and_then(|m| m.entries.get_mut(ei))
            && !e.implementors.contains(&implementor)
        {
            e.implementors.push(implementor);
        }
    }
}

// ---------------------------------------------------------------- writing

const KIND_LABELS: [(&str, &str); 9] = [
    ("struct", "struct"),
    ("enum", "enum"),
    ("union", "union"),
    ("trait", "trait"),
    ("type", "type alias"),
    ("fn", "fn"),
    ("const", "const"),
    ("static", "static"),
    ("macro", "macro"),
];

fn label(kind: &str) -> &str {
    KIND_LABELS.iter().find(|(k, _)| *k == kind).map(|(_, l)| *l).unwrap_or(kind)
}

fn anchor(path: &str) -> String {
    path.replace("::", "-").to_lowercase()
}

/// The first sentence of a doc comment, on one line, safe in a table cell.
fn summary(doc: &str) -> String {
    let para = doc.split("\n\n").next().unwrap_or("").replace('\n', " ");
    let cut = para.find(". ").map(|i| i + 1).unwrap_or(para.len());
    para[..cut].replace('|', "\\|").trim().to_string()
}

/// Text indented to continue a list item.
fn indent(text: &str) -> String {
    text.lines().map(|l| if l.is_empty() { String::new() } else { format!("  {l}") }).collect::<Vec<_>>().join("\n")
}

fn members(out: &mut String, title: &str, list: &[Member]) {
    if list.is_empty() {
        return;
    }
    let _ = writeln!(out, "**{title}**\n");
    for m in list {
        let from = m.from.as_ref().map(|f| format!(" *(in `{f}`)*")).unwrap_or_default();
        let _ = writeln!(out, "- `{}`{from}", m.sig);
        if !m.doc.is_empty() {
            let _ = writeln!(out, "\n{}\n", indent(&m.doc));
        }
    }
    out.push('\n');
}

fn render(krate: &Crate) -> String {
    let mut out = String::new();
    // The title stays outside the raw block so the site takes it as the page's title.
    let _ = writeln!(out, "# `{}` API\n\n{{% raw %}}", krate.name);
    let _ = writeln!(
        out,
        "[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.\n"
    );
    if !krate.doc.is_empty() {
        let _ = writeln!(out, "{}\n", krate.doc);
    }
    let mut modules: Vec<&Module> = krate.modules.iter().filter(|m| has_content(m)).collect();
    modules.sort_by(|a, b| a.path.cmp(&b.path));
    let _ = writeln!(out, "## Modules\n\n| Module | Items | Summary |\n|---|---|---|");
    for m in &modules {
        let _ =
            writeln!(out, "| [`{}`](#mod-{}) | {} | {} |", m.path, anchor(&m.path), m.entries.len(), summary(&m.doc));
    }
    out.push('\n');
    for m in modules {
        let _ = writeln!(out, "<a id=\"mod-{}\"></a>\n\n## `{}`\n", anchor(&m.path), m.path);
        let _ = writeln!(out, "<sub>`{}`</sub>\n", m.file);
        if !m.doc.is_empty() && m.path != krate.name {
            let _ = writeln!(out, "{}\n", m.doc);
        }
        if !m.reexports.is_empty() {
            let _ = writeln!(out, "**Re-exports**\n");
            for r in &m.reexports {
                let _ = writeln!(out, "- `{r}`");
            }
            out.push('\n');
        }
        if !m.entries.is_empty() {
            let list: Vec<String> = m
                .entries
                .iter()
                .map(|e| format!("[`{}`](#{})", e.name, anchor(&format!("{}::{}", m.path, e.name))))
                .collect();
            let _ = writeln!(out, "**Items:** {}\n", list.join(" · "));
        }
        for e in &m.entries {
            let _ = writeln!(out, "<a id=\"{}\"></a>\n", anchor(&format!("{}::{}", m.path, e.name)));
            let _ = writeln!(out, "### {} `{}`\n", label(e.kind), e.name);
            let _ = writeln!(out, "```rust\n{}\n```\n", e.sig);
            if !e.doc.is_empty() {
                let _ = writeln!(out, "{}\n", e.doc);
            }
            if !e.traits.is_empty() {
                let list: Vec<String> = e.traits.iter().map(|t| format!("`{t}`")).collect();
                let _ = writeln!(out, "**Traits:** {}\n", list.join(", "));
            }
            if !e.implementors.is_empty() {
                let mut list = e.implementors.clone();
                list.sort();
                let _ = writeln!(out, "**Implemented by ({}):** {}\n", list.len(), list.join(", "));
            }
            members(&mut out, "Fields", &e.fields);
            if e.hidden_fields {
                let _ = writeln!(out, "*Some fields are private.*\n");
            }
            members(&mut out, "Variants", &e.variants);
            let title = if e.kind == "trait" { "Required and provided items" } else { "Methods" };
            members(&mut out, title, &e.methods);
        }
        for (ty, methods) in &m.other {
            let _ = writeln!(out, "### impl `{ty}`\n");
            members(&mut out, "Methods", methods);
        }
    }
    out.push_str("{% endraw %}\n");
    out
}

fn has_content(m: &Module) -> bool {
    !m.entries.is_empty() || !m.reexports.is_empty() || !m.other.is_empty() || !m.doc.is_empty()
}

fn render_index(crates: &[Crate]) -> String {
    let mut out = String::new();
    out.push_str("# API reference\n\n");
    out.push_str(
        "[Overview of the architecture](../api.md) · [Visão geral em português](../api.pt-BR.md) · Generated from the source by `cargo run -p api_doc`.\n\n",
    );
    out.push_str("| Crate | Modules | Items | Methods | Summary |\n|---|---|---|---|---|\n");
    for k in crates {
        let modules = k.modules.iter().filter(|m| has_content(m)).count();
        let items: usize = k.modules.iter().map(|m| m.entries.len()).sum();
        let methods: usize = k
            .modules
            .iter()
            .map(|m| {
                m.entries.iter().map(|e| e.methods.len()).sum::<usize>() + m.other.values().map(Vec::len).sum::<usize>()
            })
            .sum();
        let _ = writeln!(out, "| [`{0}`]({0}.md) | {modules} | {items} | {methods} | {1} |", k.name, summary(&k.doc));
    }
    out
}
