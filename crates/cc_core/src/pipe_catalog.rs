//! The built-in pipe series: Brazilian, European and US product lines as
//! series files (`assets/pipes/*.json`, see [`crate::pipe_file`]) compiled
//! into the binary and read once. The catalog is not stored in projects:
//! a series is copied into [`Project::pipe_series`](crate::Project) when it
//! is first used, so a project keeps working when the catalog changes.

use crate::pipe_file::read_series;
use crate::pipe_series::PipeSeries;
use std::sync::OnceLock;

/// Where a built-in series is sold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Brazil,
    Europe,
    Us,
}

impl Region {
    pub const ALL: [Region; 3] = [Self::Brazil, Self::Europe, Self::Us];

    /// `"brazil"`, `"europe"`, `"us"`.
    pub fn code(self) -> &'static str {
        match self {
            Self::Brazil => "brazil",
            Self::Europe => "europe",
            Self::Us => "us",
        }
    }

    /// The i18n key of the region's name.
    pub fn key(self) -> &'static str {
        match self {
            Self::Brazil => "pipe-region-brazil",
            Self::Europe => "pipe-region-europe",
            Self::Us => "pipe-region-us",
        }
    }
}

/// A built-in series file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltIn {
    /// The series' code, as in the file.
    pub code: &'static str,
    /// The i18n key of the series' name: `pipe-series-<code lowercased>`.
    pub name_key: &'static str,
    pub region: Region,
    /// The file's name in `assets/pipes`.
    pub file: &'static str,
    /// The file's text.
    pub text: &'static str,
}

const fn built_in(
    code: &'static str,
    name_key: &'static str,
    region: Region,
    file: &'static str,
    text: &'static str,
) -> BuiltIn {
    BuiltIn { code, name_key, region, file, text }
}

use Region::{Brazil, Europe, Us};

/// Every built-in series file.
pub const BUILT_IN: &[BuiltIn] = &[
    built_in(
        "BR-COBRE-A",
        "pipe-series-br-cobre-a",
        Brazil,
        "br-cobre-a.json",
        include_str!("../assets/pipes/br-cobre-a.json"),
    ),
    built_in(
        "BR-COBRE-E",
        "pipe-series-br-cobre-e",
        Brazil,
        "br-cobre-e.json",
        include_str!("../assets/pipes/br-cobre-e.json"),
    ),
    built_in("BR-CPVC", "pipe-series-br-cpvc", Brazil, "br-cpvc.json", include_str!("../assets/pipes/br-cpvc.json")),
    built_in(
        "BR-PPR-PN20",
        "pipe-series-br-ppr-pn20",
        Brazil,
        "br-ppr-pn20.json",
        include_str!("../assets/pipes/br-ppr-pn20.json"),
    ),
    built_in(
        "BR-PVC-ESG-N",
        "pipe-series-br-pvc-esg-n",
        Brazil,
        "br-pvc-esg-n.json",
        include_str!("../assets/pipes/br-pvc-esg-n.json"),
    ),
    built_in(
        "BR-PVC-ESG-R",
        "pipe-series-br-pvc-esg-r",
        Brazil,
        "br-pvc-esg-r.json",
        include_str!("../assets/pipes/br-pvc-esg-r.json"),
    ),
    built_in(
        "BR-PVC-SOLD",
        "pipe-series-br-pvc-sold",
        Brazil,
        "br-pvc-sold.json",
        include_str!("../assets/pipes/br-pvc-sold.json"),
    ),
    built_in(
        "EU-PE100-SDR11",
        "pipe-series-eu-pe100-sdr11",
        Europe,
        "eu-pe100-sdr11.json",
        include_str!("../assets/pipes/eu-pe100-sdr11.json"),
    ),
    built_in(
        "EU-PELD-PN4",
        "pipe-series-eu-peld-pn4",
        Europe,
        "eu-peld-pn4.json",
        include_str!("../assets/pipes/eu-peld-pn4.json"),
    ),
    built_in(
        "EU-PELD-PN6",
        "pipe-series-eu-peld-pn6",
        Europe,
        "eu-peld-pn6.json",
        include_str!("../assets/pipes/eu-peld-pn6.json"),
    ),
    built_in(
        "EU-PVCU-1329",
        "pipe-series-eu-pvcu-1329",
        Europe,
        "eu-pvcu-1329.json",
        include_str!("../assets/pipes/eu-pvcu-1329.json"),
    ),
    built_in(
        "EU-PVCU-1401-SN4",
        "pipe-series-eu-pvcu-1401-sn4",
        Europe,
        "eu-pvcu-1401-sn4.json",
        include_str!("../assets/pipes/eu-pvcu-1401-sn4.json"),
    ),
    built_in(
        "EU-PVCU-1401-SN8",
        "pipe-series-eu-pvcu-1401-sn8",
        Europe,
        "eu-pvcu-1401-sn8.json",
        include_str!("../assets/pipes/eu-pvcu-1401-sn8.json"),
    ),
    built_in(
        "EU-PVCU-1452-PN10",
        "pipe-series-eu-pvcu-1452-pn10",
        Europe,
        "eu-pvcu-1452-pn10.json",
        include_str!("../assets/pipes/eu-pvcu-1452-pn10.json"),
    ),
    built_in(
        "EU-PVCU-1452-PN16",
        "pipe-series-eu-pvcu-1452-pn16",
        Europe,
        "eu-pvcu-1452-pn16.json",
        include_str!("../assets/pipes/eu-pvcu-1452-pn16.json"),
    ),
    built_in(
        "US-CPVC-CTS",
        "pipe-series-us-cpvc-cts",
        Us,
        "us-cpvc-cts.json",
        include_str!("../assets/pipes/us-cpvc-cts.json"),
    ),
    built_in("US-CU-L", "pipe-series-us-cu-l", Us, "us-cu-l.json", include_str!("../assets/pipes/us-cu-l.json")),
    built_in("US-CU-M", "pipe-series-us-cu-m", Us, "us-cu-m.json", include_str!("../assets/pipes/us-cu-m.json")),
    built_in("US-PEX", "pipe-series-us-pex", Us, "us-pex.json", include_str!("../assets/pipes/us-pex.json")),
    built_in(
        "US-PVC-DWV",
        "pipe-series-us-pvc-dwv",
        Us,
        "us-pvc-dwv.json",
        include_str!("../assets/pipes/us-pvc-dwv.json"),
    ),
    built_in(
        "US-PVC-SCH40",
        "pipe-series-us-pvc-sch40",
        Us,
        "us-pvc-sch40.json",
        include_str!("../assets/pipes/us-pvc-sch40.json"),
    ),
    built_in(
        "US-PVC-SCH80",
        "pipe-series-us-pvc-sch80",
        Us,
        "us-pvc-sch80.json",
        include_str!("../assets/pipes/us-pvc-sch80.json"),
    ),
];

/// Every built-in series that reads, with its file, parsed on first use; a
/// file that does not read is left out (a test makes sure none is).
pub fn all() -> &'static [(&'static BuiltIn, PipeSeries)] {
    static ALL: OnceLock<Vec<(&'static BuiltIn, PipeSeries)>> = OnceLock::new();
    ALL.get_or_init(|| BUILT_IN.iter().filter_map(|b| Some((b, read_series(b.text).ok()?))).collect())
}

/// The built-in file with this code (case ignored).
pub fn built_in_of(code: &str) -> Option<&'static BuiltIn> {
    all().iter().map(|(b, _)| *b).find(|b| b.code.eq_ignore_ascii_case(code))
}

/// The built-in series with this code (case ignored).
pub fn series(code: &str) -> Option<&'static PipeSeries> {
    all().iter().find(|(b, _)| b.code.eq_ignore_ascii_case(code)).map(|(_, s)| s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe_file::write_series;
    use crate::pipe_series::FittingRowKind;
    use std::collections::BTreeSet;

    #[test]
    fn every_built_in_series_reads() {
        let failures: Vec<String> =
            BUILT_IN.iter().filter_map(|b| read_series(b.text).err().map(|e| format!("{}: {e}", b.file))).collect();
        assert!(failures.is_empty(), "{failures:#?}");
        assert_eq!(all().len(), 22);
    }

    #[test]
    fn codes_are_unique_and_match_the_files() {
        let mut codes = BTreeSet::new();
        for (b, s) in all() {
            assert!(codes.insert(b.code), "{} repeats", b.code);
            assert_eq!(s.code, b.code);
            assert_eq!(format!("{}.json", b.code.to_lowercase()), b.file);
            assert_eq!(format!("pipe-series-{}", b.code.to_lowercase()), b.name_key);
            let prefix = match b.region {
                Region::Brazil => "BR-",
                Region::Europe => "EU-",
                Region::Us => "US-",
            };
            assert!(b.code.starts_with(prefix), "{}", b.code);
        }
        assert!(series("br-pvc-sold").is_some());
        assert!(series("nope").is_none());
    }

    #[test]
    fn inside_diameters_are_the_outside_less_two_walls() {
        for (b, s) in all() {
            for p in &s.pipes {
                let off = (p.di - (p.de - 2.0 * p.e)).abs();
                assert!(off <= 0.000_150_000_1, "{} {}: di is {:.3} mm off", b.code, p.size, off * 1000.0);
            }
        }
    }

    #[test]
    fn fitting_sizes_exist_and_rows_do_not_repeat() {
        for (b, s) in all() {
            let sizes: BTreeSet<&str> = s.pipes.iter().map(|p| p.size.as_str()).collect();
            let mut seen = BTreeSet::new();
            for f in &s.fittings {
                assert!(sizes.contains(f.size.as_str()), "{} {:?}", b.code, f.size);
                if let Some(s2) = &f.size2 {
                    assert!(sizes.contains(s2.as_str()), "{} {s2:?}", b.code);
                }
                if f.kind == FittingRowKind::Reducer {
                    assert!(f.size2.is_some(), "{} reducer {}", b.code, f.size);
                }
                assert!(seen.insert(f.key()), "{} repeats {:?}", b.code, f.key());
            }
        }
    }

    #[test]
    fn written_series_read_back_equal() {
        for (b, s) in all() {
            let back = read_series(&write_series(s));
            assert_eq!(back.as_ref(), Ok(s), "{}", b.code);
        }
    }
}
