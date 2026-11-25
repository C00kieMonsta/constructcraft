# `cc_rvt` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Revit project files (.rvt), read by the crates.io `rvt` crate (rvt-rs,
Apache-2.0, pure Rust, `forbid(unsafe_code)`) and written as IFC4 text,
which `cc_ifc` imports as it imports any IFC file.
`rvt` is not written to our robustness rules (it may panic on damaged
files), so it runs on its own thread with a large stack, inside
`catch_unwind`; a panic becomes an error and the program goes on. Its
default open and walker limits bound what it reads; a file they keep from
being read whole is refused, as `rvt-ifc` refuses it, since the model
would silently miss what those streams hold.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_rvt`](#mod-cc_rvt) | 4 | Revit project files (.rvt), read by the crates.io `rvt` crate (rvt-rs, Apache-2.0, pure Rust, `forbid(unsafe_code)`) and written as IFC4 text, which `cc_ifc` imports as it imports any IFC file. |

<a id="mod-cc_rvt"></a>

## `cc_rvt`

<sub>`crates/cc_rvt/src/lib.rs`</sub>

**Items:** [`MAX_RVT_BYTES`](#cc_rvt-max_rvt_bytes) · [`RvtError`](#cc_rvt-rvterror) · [`Converted`](#cc_rvt-converted) · [`to_ifc`](#cc_rvt-to_ifc)

<a id="cc_rvt-max_rvt_bytes"></a>

### const `MAX_RVT_BYTES`

```rust
pub const MAX_RVT_BYTES: u64
```

The largest file read: the reader holds the whole file in memory.

<a id="cc_rvt-rvterror"></a>

### enum `RvtError`

```rust
pub enum RvtError
```

**Traits:** `Debug`, `Error`

**Variants**

- `Read(String)`

  The reader refused the file: not a Revit file, damaged, or a part it cannot read.

- `Partial { streams: usize }`

  A resource limit kept this many streams from being read whole.

- `Internal(String)`

  The reader stopped on an internal error (a panic).


<a id="cc_rvt-converted"></a>

### struct `Converted`

```rust
pub struct Converted
```

A Revit file written as IFC4, with what the conversion recovered.

**Traits:** `Debug`, `Clone`

**Fields**

- `ifc: String`

  The IFC4 file, as text.

- `revit_version: Option<u32>`

  The Revit release (year) that saved the file, when read.

- `elements: usize`

  Building elements written.

- `with_geometry: usize`

  Of those, the elements with a body.

- `boxes: usize`

  Bodies that are their element's bounding box: placed and sized, shape not decoded.

- `notes: Vec<String>`

  The converter's notes and warnings, in order.


<a id="cc_rvt-to_ifc"></a>

### fn `to_ifc`

```rust
pub fn to_ifc(data: Vec<u8>) -> Result<Converted, RvtError>
```

Convert the bytes of a Revit file to IFC4.

{% endraw %}
