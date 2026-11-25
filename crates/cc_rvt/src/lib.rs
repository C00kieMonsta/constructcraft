//! Revit project files (.rvt), read by the crates.io `rvt` crate (rvt-rs,
//! Apache-2.0, pure Rust, `forbid(unsafe_code)`) and written as IFC4 text,
//! which `cc_ifc` imports as it imports any IFC file.
//!
//! `rvt` is not written to our robustness rules (it may panic on damaged
//! files), so it runs on its own thread with a large stack, inside
//! `catch_unwind`; a panic becomes an error and the program goes on. Its
//! default open and walker limits bound what it reads; a file they keep from
//! being read whole is refused, as `rvt-ifc` refuses it, since the model
//! would silently miss what those streams hold.

use rvt::RevitFile;
use rvt::ifc::step_writer::{StepOptions, write_step_to};
use rvt::ifc::{ExportOptions, run_export_with_limits};
use rvt::reader::OpenLimits;
use rvt::walker::WalkerLimits;

/// Stack of the reading thread (the reader recurses over nested records).
const STACK_BYTES: usize = 128 << 20;

/// The largest file read: the reader holds the whole file in memory.
pub const MAX_RVT_BYTES: u64 = rvt::reader::DEFAULT_MAX_FILE_BYTES;

#[derive(Debug, thiserror::Error)]
pub enum RvtError {
    /// The reader refused the file: not a Revit file, damaged, or a part it cannot read.
    #[error("{0}")]
    Read(String),
    /// A resource limit kept this many streams from being read whole.
    #[error("{streams} stream(s) of the file are too large to read whole")]
    Partial { streams: usize },
    /// The reader stopped on an internal error (a panic).
    #[error("{0}")]
    Internal(String),
}

/// A Revit file written as IFC4, with what the conversion recovered.
#[derive(Debug, Clone)]
pub struct Converted {
    /// The IFC4 file, as text.
    pub ifc: String,
    /// The Revit release (year) that saved the file, when read.
    pub revit_version: Option<u32>,
    /// Building elements written.
    pub elements: usize,
    /// Of those, the elements with a body.
    pub with_geometry: usize,
    /// Bodies that are their element's bounding box: placed and sized, shape not decoded.
    pub boxes: usize,
    /// The converter's notes and warnings, in order.
    pub notes: Vec<String>,
}

/// Convert the bytes of a Revit file to IFC4.
pub fn to_ifc(data: Vec<u8>) -> Result<Converted, RvtError> {
    let worker = std::thread::Builder::new()
        .name("rvt-reader".into())
        .stack_size(STACK_BYTES)
        .spawn(move || std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| convert(data))))
        .map_err(|e| RvtError::Internal(e.to_string()))?;
    match worker.join() {
        Ok(Ok(result)) => result,
        Ok(Err(payload)) | Err(payload) => Err(RvtError::Internal(panic_message(payload.as_ref()))),
    }
}

fn convert(data: Vec<u8>) -> Result<Converted, RvtError> {
    let read = |e: rvt::Error| RvtError::Read(e.to_string());
    let mut file = RevitFile::open_bytes_with_limits(data, OpenLimits::default()).map_err(read)?;
    let run = run_export_with_limits(&mut file, &ExportOptions::default(), WalkerLimits::default()).map_err(read)?;
    if run.diagnostics.is_partial() {
        return Err(RvtError::Partial { streams: run.unread.len().max(1) });
    }
    // The model is complete: release the file and the partitions it inflated before writing.
    drop(file);
    let mut bytes = Vec::new();
    write_step_to(&run.model, &StepOptions::default(), &mut bytes).map_err(|e| RvtError::Read(e.to_string()))?;
    let d = &run.diagnostics;
    let notes = run.notes.iter().map(ToString::to_string).chain(d.warnings.iter().cloned()).collect();
    Ok(Converted {
        ifc: String::from_utf8_lossy(&bytes).into_owned(),
        revit_version: d.input.revit_version,
        elements: d.exported.building_elements,
        with_geometry: d.exported.building_elements_with_geometry,
        boxes: d.exported.bounding_box_bodies,
        notes,
    })
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "internal error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic Revit file written by rvt-rs's `gen-fixture` (no Autodesk content):
    /// `gen-fixture synthetic -o synthetic.rvt` with rvt 0.25.0.
    const SYNTHETIC: &[u8] = include_bytes!("../tests/fixtures/synthetic.rvt");

    #[test]
    fn a_revit_file_becomes_an_ifc4_project() {
        let out = to_ifc(SYNTHETIC.to_vec()).unwrap();
        assert!(out.ifc.contains("FILE_SCHEMA(('IFC4'))"), "{}", &out.ifc[..out.ifc.len().min(400)]);
        assert!(out.ifc.contains("IFCPROJECT("));
        assert_eq!(out.revit_version, Some(2024));
    }

    #[test]
    fn other_files_are_errors() {
        for data in [Vec::new(), b"ISO-10303-21;".to_vec(), vec![0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]] {
            assert!(matches!(to_ifc(data), Err(RvtError::Read(_))));
        }
    }

    /// Damaged files never panic: every prefix and a few flipped bytes of a real file.
    #[test]
    fn damaged_files_are_errors_or_conversions() {
        for len in (0..SYNTHETIC.len()).step_by(997) {
            let _ = to_ifc(SYNTHETIC[..len].to_vec());
        }
        for at in (512..SYNTHETIC.len()).step_by(1499) {
            let mut data = SYNTHETIC.to_vec();
            data[at] ^= 0xff;
            let _ = to_ifc(data);
        }
    }
}
