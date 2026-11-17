//! Pipe series files: one series as JSON, lengths in millimeters, the
//! format of the built-in catalog (`assets/pipes/*.json`) and of the files
//! users import and export. Reading is bounded and refuses anything that
//! does not make a sound series ([`PipeSeries::check`]), naming the row.

use crate::Uid;
use crate::pipe_series::{
    CatalogFitting, FittingRowKind, MAX_FILE_BYTES, MAX_FITTINGS, MAX_SIZES, PipeJoint, PipeMaterial, PipeSeries,
    PipeSeriesId, PipeSize, PipeUse, SeriesSource,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::pipe_series::PipeFileError;

/// The file format this build reads and writes.
pub const FILE_FORMAT: u64 = 1;

/// The serde form of a series file (millimeters). Rows stay raw so each one
/// is read on its own and an error names it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesFile {
    pub format: u64,
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub short: String,
    #[serde(default)]
    pub standard: String,
    pub material: PipeMaterial,
    #[serde(rename = "use")]
    pub use_: PipeUse,
    pub joint: PipeJoint,
    #[serde(default)]
    pub pressure: String,
    #[serde(default)]
    pub sources: Vec<SeriesSource>,
    pub pipes: Vec<Value>,
    #[serde(default)]
    pub fittings: Vec<Value>,
}

/// A pipe row of a file, millimeters.
#[derive(Debug, Clone, Deserialize)]
struct PipeRow {
    size: String,
    de: f64,
    e: f64,
    di: f64,
}

/// A fitting row of a file, millimeters.
#[derive(Debug, Clone, Deserialize)]
struct FittingRow {
    kind: FittingRowKind,
    #[serde(default)]
    angle: Option<f64>,
    size: String,
    #[serde(default)]
    size2: Option<String>,
    #[serde(default)]
    socket: Option<f64>,
    #[serde(default)]
    socket2: Option<f64>,
    #[serde(default)]
    z: Option<f64>,
    #[serde(default)]
    z2: Option<f64>,
    #[serde(default)]
    z3: Option<f64>,
    #[serde(default)]
    od: Option<f64>,
    #[serde(default)]
    derived: String,
}

const MM: f64 = 0.001;

/// Read a series file. The series gets id 0 and no uid; the project that
/// takes it in gives both.
pub fn read_series(text: &str) -> Result<PipeSeries, PipeFileError> {
    if text.len() > MAX_FILE_BYTES {
        return Err(PipeFileError::TooLarge(text.len()));
    }
    let value: Value = serde_json::from_str(text).map_err(|e| PipeFileError::Json(e.to_string()))?;
    let Some(format) = value.get("format") else {
        return Err(PipeFileError::Json("no format".into()));
    };
    match format.as_u64() {
        Some(FILE_FORMAT) => {}
        Some(other) => return Err(PipeFileError::Format(other)),
        None => return Err(PipeFileError::Header(format!("format {format} is not a number"))),
    }
    let file: SeriesFile = serde_json::from_value(value).map_err(|e| PipeFileError::Header(e.to_string()))?;
    if file.pipes.len() > MAX_SIZES {
        return Err(PipeFileError::Header(format!("more than {MAX_SIZES} pipe sizes")));
    }
    if file.fittings.len() > MAX_FITTINGS {
        return Err(PipeFileError::Header(format!("more than {MAX_FITTINGS} fittings")));
    }
    let mut pipes = Vec::with_capacity(file.pipes.len());
    for (row, v) in file.pipes.into_iter().enumerate() {
        let r: PipeRow = serde_json::from_value(v).map_err(|e| PipeFileError::Pipe { row, message: e.to_string() })?;
        pipes.push(PipeSize { size: r.size, de: r.de * MM, e: r.e * MM, di: r.di * MM });
    }
    let mut fittings = Vec::with_capacity(file.fittings.len());
    for (row, v) in file.fittings.into_iter().enumerate() {
        let r: FittingRow =
            serde_json::from_value(v).map_err(|e| PipeFileError::Fitting { row, message: e.to_string() })?;
        let mm = |v: Option<f64>| v.map(|v| v * MM);
        fittings.push(CatalogFitting {
            kind: r.kind,
            angle: r.angle,
            size: r.size,
            size2: r.size2,
            socket: mm(r.socket),
            socket2: mm(r.socket2),
            z: mm(r.z),
            z2: mm(r.z2),
            z3: mm(r.z3),
            od: mm(r.od),
            derived: r.derived,
        });
    }
    let series = PipeSeries {
        id: PipeSeriesId(0),
        uid: Uid::default(),
        code: file.code,
        name: file.name,
        short: file.short,
        standard: file.standard,
        material: file.material,
        use_: file.use_,
        joint: file.joint,
        pressure: file.pressure,
        sources: file.sources,
        pipes,
        fittings,
    };
    series.check()?;
    Ok(series)
}

/// A number for a file: meters as millimeters to a millionth, without
/// trailing zeros (`20`, `1.5`, `21.34`).
fn mm(v: f64) -> String {
    number(v / MM)
}

fn number(v: f64) -> String {
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() { "0".into() } else { s.into() }
}

fn quoted(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// Write a series in the file format, in the style of the built-in files:
/// the header one field per line, then one source, size or fitting per line.
pub fn write_series(series: &PipeSeries) -> String {
    let mut out = String::from("{\n");
    out += &format!("  \"format\": {FILE_FORMAT},\n");
    out += &format!("  \"code\": {},\n", quoted(&series.code));
    out += &format!("  \"name\": {},\n", quoted(&series.name));
    out += &format!("  \"short\": {},\n", quoted(&series.short));
    out += &format!("  \"standard\": {},\n", quoted(&series.standard));
    out += &format!("  \"material\": {},\n", quoted(series.material.code()));
    out += &format!("  \"use\": {},\n", quoted(series.use_.code()));
    out += &format!("  \"joint\": {},\n", quoted(series.joint.code()));
    out += &format!("  \"pressure\": {},\n", quoted(&series.pressure));
    let list = |out: &mut String, name: &str, rows: Vec<String>, last: bool| {
        if rows.is_empty() {
            *out += &format!("  \"{name}\": []");
        } else {
            *out += &format!("  \"{name}\": [\n    {}\n  ]", rows.join(",\n    "));
        }
        *out += if last { "\n" } else { ",\n" };
    };
    let sources = series
        .sources
        .iter()
        .map(|s| {
            format!(
                "{{\"url\": {}, \"title\": {}, \"covers\": {}}}",
                quoted(&s.url),
                quoted(&s.title),
                quoted(&s.covers)
            )
        })
        .collect();
    list(&mut out, "sources", sources, false);
    let pipes = series
        .pipes
        .iter()
        .map(|p| {
            format!("{{\"size\": {}, \"de\": {}, \"e\": {}, \"di\": {}}}", quoted(&p.size), mm(p.de), mm(p.e), mm(p.di))
        })
        .collect();
    list(&mut out, "pipes", pipes, false);
    let fittings = series
        .fittings
        .iter()
        .map(|f| {
            let mut fields = vec![format!("\"kind\": {}", quoted(f.kind.code()))];
            if let Some(a) = f.angle {
                fields.push(format!("\"angle\": {}", number(a)));
            }
            fields.push(format!("\"size\": {}", quoted(&f.size)));
            if let Some(s2) = &f.size2 {
                fields.push(format!("\"size2\": {}", quoted(s2)));
            }
            for (name, v) in f.lengths() {
                if let Some(v) = v {
                    fields.push(format!("\"{name}\": {}", mm(v)));
                }
            }
            if !f.derived.is_empty() {
                fields.push(format!("\"derived\": {}", quoted(&f.derived)));
            }
            format!("{{{}}}", fields.join(", "))
        })
        .collect();
    list(&mut out, "fittings", fittings, true);
    out += "}\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"{
  "format": 1,
  "code": "X",
  "name": "Pipe X",
  "material": "PVC",
  "use": "supply",
  "joint": "solvent",
  "pipes": [
    {"size": "20 mm", "de": 20, "e": 1.5, "di": 17},
    {"size": "25 mm", "de": 25, "e": 1.7, "di": 21.6}
  ],
  "fittings": [
    {"kind": "elbow", "angle": 90, "size": "20 mm", "socket": 16.5, "z": 11.4, "od": 28},
    {"kind": "reducer", "size": "25 mm", "size2": "20 mm", "socket": 19, "z": null}
  ]
}"#;

    #[test]
    fn reads_millimeters_as_meters() {
        let s = read_series(GOOD).unwrap();
        assert_eq!(s.code, "X");
        assert!((s.pipes[1].di - 0.0216).abs() < 1e-12);
        assert_eq!(s.fittings[0].angle, Some(90.0));
        assert!((s.fittings[0].od.unwrap() - 0.028).abs() < 1e-12);
        assert_eq!(s.fittings[1].z, None);
        assert_eq!(read_series(&write_series(&s)).unwrap(), s);
    }

    #[test]
    fn refuses_a_newer_format() {
        let text = GOOD.replace("\"format\": 1", "\"format\": 2");
        assert_eq!(read_series(&text), Err(PipeFileError::Format(2)));
    }

    #[test]
    fn refuses_a_file_too_large() {
        let text = format!("{GOOD}{}", " ".repeat(3 * 1024 * 1024));
        assert!(matches!(read_series(&text), Err(PipeFileError::TooLarge(_))));
    }

    #[test]
    fn refuses_a_number_as_a_string() {
        let text = GOOD.replace("\"de\": 25", "\"de\": \"NaN\"");
        assert!(matches!(read_series(&text), Err(PipeFileError::Pipe { row: 1, .. })));
    }

    #[test]
    fn refuses_an_unknown_kind() {
        let text = GOOD.replace("\"kind\": \"reducer\"", "\"kind\": \"bend\"");
        assert!(matches!(read_series(&text), Err(PipeFileError::Fitting { row: 1, .. })));
    }

    #[test]
    fn refuses_an_unknown_material() {
        let text = GOOD.replace("\"PVC\"", "\"Wood\"");
        assert!(matches!(read_series(&text), Err(PipeFileError::Header(_))));
    }

    #[test]
    fn refuses_a_reducer_without_size2() {
        let text = GOOD.replace("\"size2\": \"20 mm\", ", "");
        assert!(matches!(read_series(&text), Err(PipeFileError::Fitting { row: 1, .. })));
    }

    #[test]
    fn refuses_a_wall_of_half_the_diameter() {
        let text = GOOD.replace("\"e\": 1.5, \"di\": 17", "\"e\": 10, \"di\": 0");
        assert!(matches!(read_series(&text), Err(PipeFileError::Pipe { row: 0, .. })));
    }

    #[test]
    fn refuses_an_inside_diameter_off_the_wall() {
        let text = GOOD.replace("\"di\": 21.6", "\"di\": 19");
        assert!(matches!(read_series(&text), Err(PipeFileError::Pipe { row: 1, .. })));
    }

    #[test]
    fn refuses_a_fitting_of_a_missing_size() {
        let text = GOOD.replace("\"angle\": 90, \"size\": \"20 mm\"", "\"angle\": 90, \"size\": \"32 mm\"");
        assert!(matches!(read_series(&text), Err(PipeFileError::Fitting { row: 0, .. })));
    }
}
