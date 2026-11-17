//! Pipe series: the sizes of one pipe product line (outside diameter, wall
//! and inside diameter per size) and the fittings sold for it, with their
//! dimensions. Series live in [`Project::pipe_series`](crate::Project) and
//! are copied there from the built-in catalog
//! ([`crate::pipe_catalog`]) or from series files ([`crate::pipe_file`]).
//! Lengths are meters; files hold millimeters.

use crate::Uid;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub use crate::ids::PipeSeriesId;

/// Most series in a project.
pub const MAX_SERIES: usize = 200;
/// Most sizes in a series.
pub const MAX_SIZES: usize = 100;
/// Most fitting rows in a series.
pub const MAX_FITTINGS: usize = 2000;
/// Longest code, name, label or note, in characters.
pub const MAX_TEXT: usize = 200;
/// Longest source url, title or note of what it covers, in characters.
pub const MAX_SOURCE_TEXT: usize = 1000;
/// Most sources of a series.
pub const MAX_SOURCES: usize = 50;
/// Largest series file read, in bytes.
pub const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
/// Smallest outside diameter, m.
pub const MIN_DE: f64 = 0.001;
/// Largest outside diameter, m.
pub const MAX_DE: f64 = 2.0;
/// Largest fitting length (socket, laying length, body diameter), m.
pub const MAX_FITTING_LENGTH: f64 = 2.0;
/// The range of fitting angles, degrees.
pub const MIN_ANGLE: f64 = 1.0;
pub const MAX_ANGLE: f64 = 180.0;
/// How far a size's inside diameter may be from `de − 2e`, m.
pub const DI_TOLERANCE: f64 = 0.001;

/// What a pipe is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipeMaterial {
    #[serde(rename = "PVC")]
    Pvc,
    #[serde(rename = "CPVC")]
    Cpvc,
    #[serde(rename = "PPR")]
    Ppr,
    #[serde(rename = "PE")]
    Pe,
    #[serde(rename = "PEX")]
    Pex,
    Copper,
    Steel,
    CastIron,
}

impl PipeMaterial {
    pub const ALL: [PipeMaterial; 8] =
        [Self::Pvc, Self::Cpvc, Self::Ppr, Self::Pe, Self::Pex, Self::Copper, Self::Steel, Self::CastIron];

    /// The code files use (`"PVC"`, `"Copper"`).
    pub fn code(self) -> &'static str {
        match self {
            Self::Pvc => "PVC",
            Self::Cpvc => "CPVC",
            Self::Ppr => "PPR",
            Self::Pe => "PE",
            Self::Pex => "PEX",
            Self::Copper => "Copper",
            Self::Steel => "Steel",
            Self::CastIron => "CastIron",
        }
    }
}

/// What a series carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipeUse {
    /// Water supply, under pressure.
    Supply,
    /// Drainage, by gravity.
    Drain,
    #[default]
    Any,
}

impl PipeUse {
    pub const ALL: [PipeUse; 3] = [Self::Supply, Self::Drain, Self::Any];

    pub fn code(self) -> &'static str {
        match self {
            Self::Supply => "supply",
            Self::Drain => "drain",
            Self::Any => "any",
        }
    }
}

/// How pipes and fittings of a series are joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipeJoint {
    /// Solvent cement in a socket.
    Solvent,
    /// A rubber ring in a socket.
    Ring,
    /// Soldered (or brazed) in a socket.
    Solder,
    Compression,
    Crimp,
    /// Heat fusion (socket or butt).
    Fusion,
    Threaded,
    Push,
    Flanged,
}

impl PipeJoint {
    pub const ALL: [PipeJoint; 9] = [
        Self::Solvent,
        Self::Ring,
        Self::Solder,
        Self::Compression,
        Self::Crimp,
        Self::Fusion,
        Self::Threaded,
        Self::Push,
        Self::Flanged,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Solvent => "solvent",
            Self::Ring => "ring",
            Self::Solder => "solder",
            Self::Compression => "compression",
            Self::Crimp => "crimp",
            Self::Fusion => "fusion",
            Self::Threaded => "threaded",
            Self::Push => "push",
            Self::Flanged => "flanged",
        }
    }
}

/// The kind of a catalog fitting row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FittingRowKind {
    Elbow,
    Tee,
    Wye,
    Cross,
    Coupling,
    Reducer,
}

impl FittingRowKind {
    pub const ALL: [FittingRowKind; 6] =
        [Self::Elbow, Self::Tee, Self::Wye, Self::Cross, Self::Coupling, Self::Reducer];

    pub fn code(self) -> &'static str {
        match self {
            Self::Elbow => "elbow",
            Self::Tee => "tee",
            Self::Wye => "wye",
            Self::Cross => "cross",
            Self::Coupling => "coupling",
            Self::Reducer => "reducer",
        }
    }
}

/// A document a series' values come from.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SeriesSource {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub title: String,
    /// What the document gives (pipe dimensions, socket depths…).
    #[serde(default)]
    pub covers: String,
}

/// One size of a series. Lengths in meters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipeSize {
    /// The label the market uses (`"25 mm"`, `"DN 100"`, `"1/2\""`), unique in the series.
    pub size: String,
    /// Outside diameter.
    pub de: f64,
    /// Wall thickness.
    pub e: f64,
    /// Inside diameter.
    pub di: f64,
}

/// One fitting sold for a series. Lengths in meters, `None` where no source
/// gives the value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogFitting {
    pub kind: FittingRowKind,
    /// Degrees: an elbow's bend, a wye's branch angle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    /// The size of the main run (a reducer's larger end).
    pub size: String,
    /// The branch's size (tees, wyes, crosses) or a reducer's other end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size2: Option<String>,
    /// Socket depth of the main ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket: Option<f64>,
    /// Socket depth of the branch or the other end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket2: Option<f64>,
    /// Laying length (center, or the other end, to the socket's bottom) of the main ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
    /// Laying length of the branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z2: Option<f64>,
    /// Laying length of the main run's second end, where it differs from `z`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z3: Option<f64>,
    /// Outside diameter of the fitting's body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub od: Option<f64>,
    /// How values that came from a rule were derived (empty when all were published).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub derived: String,
}

impl CatalogFitting {
    /// What makes two rows the same fitting: kind, angle (to a thousandth of
    /// a degree), sizes.
    pub fn key(&self) -> (FittingRowKind, Option<i64>, &str, Option<&str>) {
        (self.kind, self.angle.map(|a| (a * 1000.0).round() as i64), &self.size, self.size2.as_deref())
    }

    /// The lengths by field name, for checks.
    pub fn lengths(&self) -> [(&'static str, Option<f64>); 6] {
        [
            ("socket", self.socket),
            ("socket2", self.socket2),
            ("z", self.z),
            ("z2", self.z2),
            ("z3", self.z3),
            ("od", self.od),
        ]
    }
}

/// A pipe series of the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipeSeries {
    pub id: PipeSeriesId,
    #[serde(default)]
    pub uid: Uid,
    /// Stable code, unique in the project (`"BR-PVC-SOLD"`).
    pub code: String,
    /// The pipe type name prefix (`"Tubo PVC soldável"`).
    pub name: String,
    /// The short name fitting names use (`"PVC soldável"`).
    #[serde(default)]
    pub short: String,
    /// The standard (`"ABNT NBR 5648"`).
    #[serde(default)]
    pub standard: String,
    pub material: PipeMaterial,
    #[serde(default, rename = "use")]
    pub use_: PipeUse,
    pub joint: PipeJoint,
    /// The pressure class or series, as text (`"PN 7,5"`, `"Sch 40"`).
    #[serde(default)]
    pub pressure: String,
    #[serde(default)]
    pub sources: Vec<SeriesSource>,
    #[serde(default)]
    pub pipes: Vec<PipeSize>,
    #[serde(default)]
    pub fittings: Vec<CatalogFitting>,
}

/// Why a series (or a series file) is refused. Row indices count from 0.
#[derive(Debug, Clone, PartialEq)]
pub enum PipeFileError {
    /// A file format this build does not read.
    Format(u64),
    /// The text is larger than [`MAX_FILE_BYTES`].
    TooLarge(usize),
    /// The text is not JSON of a series.
    Json(String),
    /// A header field (code, name, material, use, joint, sources…) is wrong.
    Header(String),
    /// A pipe size row is wrong.
    Pipe { row: usize, message: String },
    /// A fitting row is wrong.
    Fitting { row: usize, message: String },
}

impl fmt::Display for PipeFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format(v) => write!(f, "unsupported pipe series format {v}"),
            Self::TooLarge(n) => write!(f, "pipe series file too large ({n} bytes, at most {MAX_FILE_BYTES})"),
            Self::Json(m) => write!(f, "not a pipe series file: {m}"),
            Self::Header(m) => write!(f, "pipe series: {m}"),
            Self::Pipe { row, message } => write!(f, "pipe series: pipes[{row}]: {message}"),
            Self::Fitting { row, message } => write!(f, "pipe series: fittings[{row}]: {message}"),
        }
    }
}

impl std::error::Error for PipeFileError {}

fn text(name: &str, v: &str, max: usize, required: bool) -> Result<(), String> {
    if required && v.trim().is_empty() {
        return Err(format!("{name} is empty"));
    }
    if v.chars().count() > max {
        return Err(format!("{name} is longer than {max} characters"));
    }
    Ok(())
}

impl PipeSize {
    /// Whether the row's numbers make a pipe: `de` within
    /// [`MIN_DE`]..=[`MAX_DE`], `0 < e < de/2`, `di` within
    /// [`DI_TOLERANCE`] of `de − 2e`, and a label.
    pub fn check(&self) -> Result<(), String> {
        text("size", &self.size, MAX_TEXT, true)?;
        if !(self.de.is_finite() && (MIN_DE..=MAX_DE).contains(&self.de)) {
            return Err(format!("de {} is out of range", self.de));
        }
        if !(self.e.is_finite() && self.e > 0.0 && self.e < self.de / 2.0) {
            return Err(format!("e {} is not between 0 and de/2", self.e));
        }
        if !(self.di.is_finite() && self.di > 0.0 && (self.di - (self.de - 2.0 * self.e)).abs() <= DI_TOLERANCE) {
            return Err(format!("di {} is not de - 2e", self.di));
        }
        Ok(())
    }
}

impl CatalogFitting {
    /// Whether the row is sound against the series' size labels: its sizes
    /// are rows (a reducer has two different ones), elbows have an angle,
    /// angles and lengths are in range.
    pub fn check(&self, sizes: &BTreeSet<&str>) -> Result<(), String> {
        if !sizes.contains(self.size.as_str()) {
            return Err(format!("size {:?} is not a pipe size of the series", self.size));
        }
        if let Some(s2) = &self.size2
            && !sizes.contains(s2.as_str())
        {
            return Err(format!("size2 {s2:?} is not a pipe size of the series"));
        }
        if self.kind == FittingRowKind::Reducer {
            match &self.size2 {
                None => return Err("a reducer needs size2".into()),
                Some(s2) if *s2 == self.size => return Err("a reducer's size2 equals its size".into()),
                Some(_) => {}
            }
        }
        match self.angle {
            Some(a) if !(a.is_finite() && (MIN_ANGLE..=MAX_ANGLE).contains(&a)) => {
                return Err(format!("angle {a} is out of range"));
            }
            None if self.kind == FittingRowKind::Elbow => return Err("an elbow needs an angle".into()),
            _ => {}
        }
        for (name, v) in self.lengths() {
            if let Some(v) = v
                && !(v.is_finite() && (0.0..=MAX_FITTING_LENGTH).contains(&v))
            {
                return Err(format!("{name} {v} is out of range"));
            }
        }
        text("derived", &self.derived, MAX_TEXT, false)
    }
}

impl PipeSeries {
    /// The size row labeled `size`.
    pub fn row(&self, size: &str) -> Option<&PipeSize> {
        self.pipes.iter().find(|p| p.size == size)
    }

    /// The header's texts and the counts.
    fn check_header(&self) -> Result<(), String> {
        text("code", &self.code, MAX_TEXT, true)?;
        text("name", &self.name, MAX_TEXT, true)?;
        text("short", &self.short, MAX_TEXT, false)?;
        text("standard", &self.standard, MAX_TEXT, false)?;
        text("pressure", &self.pressure, MAX_TEXT, false)?;
        if self.sources.len() > MAX_SOURCES {
            return Err(format!("more than {MAX_SOURCES} sources"));
        }
        for s in &self.sources {
            text("source url", &s.url, MAX_SOURCE_TEXT, false)?;
            text("source title", &s.title, MAX_SOURCE_TEXT, false)?;
            text("source covers", &s.covers, MAX_SOURCE_TEXT, false)?;
        }
        if self.pipes.is_empty() {
            return Err("no pipe sizes".into());
        }
        if self.pipes.len() > MAX_SIZES {
            return Err(format!("more than {MAX_SIZES} pipe sizes"));
        }
        if self.fittings.len() > MAX_FITTINGS {
            return Err(format!("more than {MAX_FITTINGS} fittings"));
        }
        Ok(())
    }

    /// Whether the series is sound: header texts and counts within bounds,
    /// each size a pipe ([`PipeSize::check`]) with a unique label, each
    /// fitting sound ([`CatalogFitting::check`]) and no fitting repeated.
    /// The first problem is returned, with its row.
    pub fn check(&self) -> Result<(), PipeFileError> {
        self.check_header().map_err(PipeFileError::Header)?;
        let mut sizes = BTreeSet::new();
        for (row, p) in self.pipes.iter().enumerate() {
            p.check().map_err(|message| PipeFileError::Pipe { row, message })?;
            if !sizes.insert(p.size.as_str()) {
                return Err(PipeFileError::Pipe { row, message: format!("size {:?} repeats", p.size) });
            }
        }
        let mut seen = BTreeSet::new();
        for (row, f) in self.fittings.iter().enumerate() {
            f.check(&sizes).map_err(|message| PipeFileError::Fitting { row, message })?;
            if !seen.insert(f.key()) {
                return Err(PipeFileError::Fitting { row, message: "the fitting repeats".into() });
            }
        }
        Ok(())
    }

    /// Bring a series loaded from a project file back to a sound one: texts
    /// cut to their bounds, extra sources, bad and repeated sizes, and bad,
    /// repeated or orphan fittings dropped. Returns the number of fixes; the
    /// series may still have no sizes (the caller drops it then).
    pub fn repair(&mut self) -> usize {
        let mut fixes = 0;
        let cut = |s: &mut String, max: usize, fixes: &mut usize| {
            if s.chars().count() > max {
                *s = s.chars().take(max).collect();
                *fixes += 1;
            }
        };
        cut(&mut self.code, MAX_TEXT, &mut fixes);
        cut(&mut self.name, MAX_TEXT, &mut fixes);
        cut(&mut self.short, MAX_TEXT, &mut fixes);
        cut(&mut self.standard, MAX_TEXT, &mut fixes);
        cut(&mut self.pressure, MAX_TEXT, &mut fixes);
        if self.code.trim().is_empty() {
            self.code = format!("SERIES-{}", self.id.0);
            fixes += 1;
        }
        if self.name.trim().is_empty() {
            self.name.clone_from(&self.code);
            fixes += 1;
        }
        if self.sources.len() > MAX_SOURCES {
            fixes += self.sources.len() - MAX_SOURCES;
            self.sources.truncate(MAX_SOURCES);
        }
        for s in &mut self.sources {
            cut(&mut s.url, MAX_SOURCE_TEXT, &mut fixes);
            cut(&mut s.title, MAX_SOURCE_TEXT, &mut fixes);
            cut(&mut s.covers, MAX_SOURCE_TEXT, &mut fixes);
        }
        let before = self.pipes.len();
        let mut labels = BTreeSet::new();
        self.pipes.retain(|p| p.check().is_ok() && labels.insert(p.size.clone()));
        self.pipes.truncate(MAX_SIZES);
        fixes += before - self.pipes.len();
        let before = self.fittings.len();
        let sizes: BTreeSet<&str> = self.pipes.iter().map(|p| p.size.as_str()).collect();
        let mut seen = BTreeSet::new();
        let fittings = std::mem::take(&mut self.fittings);
        self.fittings = fittings
            .into_iter()
            .filter(|f| {
                f.check(&sizes).is_ok()
                    && seen.insert((
                        f.kind,
                        f.angle.map(|a| (a * 1000.0).round() as i64),
                        f.size.clone(),
                        f.size2.clone(),
                    ))
            })
            .take(MAX_FITTINGS)
            .collect();
        fixes += before - self.fittings.len();
        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series() -> PipeSeries {
        PipeSeries {
            id: PipeSeriesId(1),
            uid: Uid::generate(),
            code: "T".into(),
            name: "Test".into(),
            short: String::new(),
            standard: String::new(),
            material: PipeMaterial::Pvc,
            use_: PipeUse::Supply,
            joint: PipeJoint::Solvent,
            pressure: String::new(),
            sources: Vec::new(),
            pipes: vec![
                PipeSize { size: "20".into(), de: 0.020, e: 0.0015, di: 0.017 },
                PipeSize { size: "25".into(), de: 0.025, e: 0.0017, di: 0.0216 },
            ],
            fittings: vec![CatalogFitting {
                kind: FittingRowKind::Reducer,
                angle: None,
                size: "25".into(),
                size2: Some("20".into()),
                socket: Some(0.019),
                socket2: None,
                z: None,
                z2: None,
                z3: None,
                od: None,
                derived: String::new(),
            }],
        }
    }

    #[test]
    fn a_sound_series_passes_and_repair_keeps_it() {
        let mut s = series();
        assert_eq!(s.check(), Ok(()));
        assert_eq!(s.repair(), 0);
        assert_eq!(s.row("25").map(|r| r.de), Some(0.025));
    }

    #[test]
    fn repair_drops_bad_sizes_and_their_fittings() {
        let mut s = series();
        s.pipes[1].e = f64::NAN;
        assert!(matches!(s.check(), Err(PipeFileError::Pipe { row: 1, .. })));
        assert_eq!(s.repair(), 2);
        assert_eq!(s.pipes.len(), 1);
        assert!(s.fittings.is_empty());
        assert_eq!(s.check(), Ok(()));
    }
}
