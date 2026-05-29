//! Loading tunings, layouts, and the generation spec from RON config files.
//!
//! RON is deserialized into thin `Raw*` types (strings and ints only), then an
//! `into_*` / `load` pass converts them to validated domain types, parsing
//! interval names, checking table sizes, resolving color schemes, and verifying
//! every output's references.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::lumatone::{coverage, FillInfo, Generator, Layout};
use crate::tuning::{ColorScheme, Edo, Interval, IntervalDirection, IntervalStep, MidiNote, Tuning};

/// A fully loaded and validated configuration, ready to drive generation.
pub struct Config {
    tunings: BTreeMap<String, Edo>,
    layouts: BTreeMap<String, Layout>,
    fills: BTreeMap<String, Vec<FillInfo>>,
    /// The things to generate, in file order.
    pub outputs: Vec<Output>,
}

/// One unit of generation: a layout rendered with a tuning and a fill.
#[derive(Debug, Deserialize)]
pub struct Output {
    pub tuning: String,
    pub layout: String,
    pub fill: String,
    pub name: String,
}

impl Config {
    pub fn tuning(&self, name: &str) -> Option<&Edo> {
        self.tunings.get(name)
    }

    pub fn layout(&self, name: &str) -> Option<&Layout> {
        self.layouts.get(name)
    }

    pub fn fill(&self, name: &str) -> Option<&[FillInfo]> {
        self.fills.get(name).map(Vec::as_slice)
    }

    /// Load and validate `tunings.ron`, `layouts.ron`, and `generate.ron` from a
    /// directory.
    pub fn load(dir: &Path) -> Result<Config> {
        let raw_tunings: BTreeMap<String, RawTuning> = read_ron(&dir.join("tunings.ron"))?;
        let raw_layouts: BTreeMap<String, RawAxes> = read_ron(&dir.join("layouts.ron"))?;
        let spec: RawGenSpec = read_ron(&dir.join("generate.ron"))?;

        let mut tunings = BTreeMap::new();
        for (name, raw) in raw_tunings {
            let edo = raw.into_edo(&name)?;
            tunings.insert(name, edo);
        }

        let mut layouts = BTreeMap::new();
        for (name, raw) in raw_layouts {
            let layout = raw.into_layout()
                .with_context(|| format!("layout {:?}", name))?;
            layouts.insert(name, layout);
        }

        let fills = spec.fills;

        // Validate every output: references resolve, the layout's intervals
        // exist in the tuning, and warn about per-octave incompleteness.
        for out in &spec.outputs {
            let edo = tunings.get(&out.tuning)
                .ok_or_else(|| anyhow!("output {:?}: unknown tuning {:?}", out.name, out.tuning))?;
            let layout = layouts.get(&out.layout)
                .ok_or_else(|| anyhow!("output {:?}: unknown layout {:?}", out.name, out.layout))?;
            if !fills.contains_key(&out.fill) {
                bail!("output {:?}: unknown fill {:?}", out.name, out.fill);
            }

            let resolved = layout.try_resolve(edo).map_err(|step| {
                anyhow!("output {:?}: tuning {:?} does not define interval {:?} used by layout {:?}",
                        out.name, out.tuning, step, out.layout)
            })?;

            let cov = coverage(&resolved, edo.octave());
            if !cov.per_octave_complete {
                eprintln!(
                    "warning: {} is not per-octave complete (gcd={}): ~{} of {} notes per octave, closes every {} octaves",
                    out.name, cov.gcd, cov.notes_per_octave, edo.octave(), cov.octaves_to_close,
                );
            }
        }

        Ok(Config { tunings, layouts, fills, outputs: spec.outputs })
    }
}

/// Read and parse a RON file into `T`.
fn read_ron<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    ron::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))
}

// --- Raw (file-shaped) types -------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawTuning {
    octave: usize,
    #[serde(default)]
    channel_octaves: Option<usize>,
    middle_c: MidiNote,
    color: String,
    /// Named interval -> step count, e.g. `("M2", 5)`.
    intervals: Vec<(String, isize)>,
    sharp_names: Vec<String>,
    /// Defaults to `sharp_names` when omitted.
    #[serde(default)]
    flat_names: Option<Vec<String>>,
}

impl RawTuning {
    fn into_edo(self, name: &str) -> Result<Edo> {
        let mut intervals = BTreeMap::new();
        for (iname, steps) in self.intervals {
            let step: IntervalStep = iname.parse()
                .map_err(|e| anyhow!("tuning {:?}: {}", name, e))?;
            intervals.insert(step, steps);
        }

        if self.sharp_names.len() != self.octave {
            bail!("tuning {:?}: sharp_names has {} entries, expected {}",
                  name, self.sharp_names.len(), self.octave);
        }
        let flat_names = self.flat_names.unwrap_or_else(|| self.sharp_names.clone());
        if flat_names.len() != self.octave {
            bail!("tuning {:?}: flat_names has {} entries, expected {}",
                  name, flat_names.len(), self.octave);
        }

        let color = ColorScheme::by_name(&self.color)
            .ok_or_else(|| anyhow!("tuning {:?}: unknown color scheme {:?}", name, self.color))?;

        Ok(Edo::new(self.octave, self.channel_octaves, self.middle_c,
                    intervals, self.sharp_names, flat_names, color))
    }
}

#[derive(Debug, Deserialize)]
enum RawGenerator {
    /// A named interval, up unless `dir: Down` is given.
    Named { interval: String, #[serde(default)] dir: IntervalDirection },
    /// A raw signed step count.
    Steps(isize),
}

impl RawGenerator {
    fn into_generator(&self) -> Result<Generator> {
        Ok(match self {
            RawGenerator::Named { interval, dir } => {
                let step: IntervalStep = interval.parse().map_err(|e: String| anyhow!(e))?;
                Generator::Named(Interval::new(step, *dir))
            }
            RawGenerator::Steps(n) => Generator::Steps(*n),
        })
    }
}

#[derive(Debug, Deserialize)]
enum RawAxes {
    RightUpLeft { right: RawGenerator, up_left: RawGenerator },
    RightUpRight { right: RawGenerator, up_right: RawGenerator },
    UpLeftUpRight { up_left: RawGenerator, up_right: RawGenerator },
}

impl RawAxes {
    fn into_layout(&self) -> Result<Layout> {
        Ok(match self {
            RawAxes::RightUpLeft { right, up_left } => Layout::RightUpLeft {
                right: right.into_generator()?,
                up_left: up_left.into_generator()?,
            },
            RawAxes::RightUpRight { right, up_right } => Layout::RightUpRight {
                right: right.into_generator()?,
                up_right: up_right.into_generator()?,
            },
            RawAxes::UpLeftUpRight { up_left, up_right } => Layout::UpLeftUpRight {
                up_left: up_left.into_generator()?,
                up_right: up_right.into_generator()?,
            },
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawGenSpec {
    fills: BTreeMap<String, Vec<FillInfo>>,
    outputs: Vec<Output>,
}

#[cfg(test)]
mod test {
    use super::*;

    const NAMES12: &str =
        r#"["C","C♯","D","D♯","E","F","F♯","G","G♯","A","A♯","B"]"#;

    fn raw_tuning(intervals: &str, names: &str) -> RawTuning {
        ron::from_str(&format!(
            r#"(octave: 12, middle_c: (channel: 1, note: 60), color: "ups_downs",
                intervals: {intervals}, sharp_names: {names})"#
        )).expect("valid RON")
    }

    #[test]
    fn rejects_unknown_interval() {
        assert!(raw_tuning(r#"[("P6", 9)]"#, NAMES12).into_edo("edo12").is_err());
    }

    #[test]
    fn rejects_wrong_name_count() {
        assert!(raw_tuning(r#"[("M2", 2)]"#, r#"["C","D"]"#).into_edo("edo12").is_err());
    }

    #[test]
    fn rejects_unknown_color() {
        let raw: RawTuning = ron::from_str(
            r#"(octave: 1, middle_c: (channel: 1, note: 60), color: "rainbow",
                intervals: [], sharp_names: ["C"])"#,
        ).unwrap();
        assert!(raw.into_edo("x").is_err());
    }

    #[test]
    fn loads_real_config() {
        let cfg = Config::load("config".as_ref()).expect("load config");
        assert_eq!(cfg.outputs.len(), 22);
        assert!(cfg.tuning("edo31").is_some());
        assert!(cfg.layout("wicki-hayden").is_some());
        assert!(cfg.fill("split").is_some());
    }
}
