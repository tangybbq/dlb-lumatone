use std::fs::create_dir;

use anyhow::Result;
use microtone::{lumatone::{FillInfo, KeyIndex, Keyboard, Layout, BOSANQUET, HARMONIC_TABLE, WICKI_HAYDEN}, tuning::{Tuning, EDO12, EDO17, EDO19, EDO31, EDO41, EDO53}};

// An ltn to generate.  For each, we generate an ltn, and a svg showing the
// layout.
struct Ltn {
    /// The tuning to use for this Ltn.
    tuning: &'static (dyn Tuning + Sync),
    /// Layout
    layout: &'static Layout,
    /// How to fill in the keys.
    fills: &'static [FillInfo],
    /// The base of the filename.
    name: &'static str,
}

// Various fills.
static SPLIT_FILL: &'static [FillInfo] = &[
    FillInfo {
        left: 8,
        right: 9,
        start: KeyIndex { group: 3, key: 47 },
    },
    FillInfo {
        left: 9,
        right: 9,
        start: KeyIndex { group: 1, key: 14 },
    },
];

// Similar to the SPLIT_FILL above, but with everything shifted to the left so
// that sharps are easier to get to.
static SPLIT_FILL_SHARP: &'static [FillInfo] = &[
    FillInfo {
        left: 6,
        right: 10,
        start: KeyIndex { group: 3, key: 38 },
    },
    FillInfo {
        left: 5,
        right: 11,
        start: KeyIndex { group: 0, key: 24 },
    },
];

static WIDE_FILL: &'static [FillInfo] = &[
    FillInfo {
        left: 16,
        right: 16,
        start: KeyIndex { group: 2, key: 39 },
    },
];

static WIDE_FILL_DN1: &'static [FillInfo] = &[
    FillInfo {
        left: 16,
        right: 16,
        start: KeyIndex { group: 2, key: 27 },
    },
];

// All of the supported LTNS.
static LTNS: &'static [Ltn] = &[
    Ltn {
        tuning: &EDO12,
        fills: SPLIT_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo12-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO12,
        fills: WIDE_FILL_DN1,
        layout: &BOSANQUET,
        name: "dlb-edo12-bosanquet",
    },
    Ltn {
        tuning: &EDO19,
        fills: SPLIT_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo19-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO19,
        fills: WIDE_FILL_DN1,
        layout: &BOSANQUET,
        name: "dlb-edo19-bosanquet",
    },
    Ltn {
        tuning: &EDO17,
        fills: SPLIT_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo17-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: SPLIT_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo31-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: SPLIT_FILL_SHARP,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo31-sharp-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: WIDE_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo31-wicki-hayden-wide",
    },
    Ltn {
        tuning: &EDO31,
        fills: WIDE_FILL_DN1,
        layout: &BOSANQUET,
        name: "dlb-edo31-bosanquet",
    },
    Ltn {
        tuning: &EDO41,
        fills: WIDE_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo41-wicki-hayden-wide",
    },
    Ltn {
        tuning: &EDO12,
        fills: SPLIT_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo12-harmonic-split",
    },
    Ltn {
        tuning: &EDO12,
        fills: WIDE_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo12-harmonic-wide",
    },
    Ltn {
        tuning: &EDO19,
        fills: SPLIT_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo19-harmonic-split",
    },
    Ltn {
        tuning: &EDO19,
        fills: WIDE_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo19-harmonic-wide",
    },
    Ltn {
        tuning: &EDO31,
        fills: SPLIT_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo31-harmonic-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: WIDE_FILL,
        layout: &HARMONIC_TABLE,
        name: "dlb-edo31-harmonic-wide",
    },
    Ltn {
        tuning: &EDO53,
        fills: WIDE_FILL,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo53-wicki-hayden",
    },
    Ltn {
        tuning: &EDO53,
        fills: WIDE_FILL_DN1,
        layout: &WICKI_HAYDEN,
        name: "dlb-edo53-dn1-wicki-hayden",
    },
    Ltn {
        tuning: &EDO53,
        fills: WIDE_FILL_DN1,
        layout: &BOSANQUET,
        name: "dlb-edo53-bosanquet",
    },
];

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    // let test = Keyboard::load("data/lumatone_Wicki-Hayden_v3.ltn")?;
    // let test = Keyboard::load("data/factory-2-harmonic-table.ltn")?;
    // test.write_svg("test-wh.svg")?;

    let _ = create_dir("layouts");

    // Write the layout map.
    let mut keyb = Keyboard::default();
    keyb.fill_reference();
    keyb.write_svg("layouts/lumatone-layout.svg")?;

    // Generate all of the layouts.
    for ltn in LTNS {
        let mut keyb = Keyboard::default();
        for fill in ltn.fills {
            keyb.fill_layout(ltn.tuning, ltn.layout, fill);
        }

        let _ = create_dir(format!("layouts/{}", ltn.name));
        keyb.write_svg(format!("layouts/{}/{}.svg", ltn.name, ltn.name))?;
        keyb.write_ltn(format!("layouts/{}/{}.ltn", ltn.name, ltn.name))?;
    }

    Ok(())
}
