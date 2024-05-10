use anyhow::Result;
use microtone::{lumatone::{FillInfo, KeyIndex, Keyboard, WICKI_HAYDEN}, tuning::{Tuning, EDO12, EDO19, EDO31}};

// An ltn to generate.  For each, we generate an ltn, and a svg showing the
// layout.
struct Ltn {
    /// The tuning to use for this Ltn.
    tuning: &'static dyn Tuning,
    /// How to fill in the keys.
    fills: &'static [FillInfo],
    /// The base of the filename.
    name: &'static str,
}

// Various fills.
static SPLIT_FILL: &'static [FillInfo] = &[
    FillInfo {
        left: 9,
        right: 9,
        start: KeyIndex { group: 3, key: 47 },
    },
    FillInfo {
        left: 9,
        right: 9,
        start: KeyIndex { group: 1, key: 14 },
    },
];

static WIDE_FILL: &'static [FillInfo] = &[
    FillInfo {
        left: 16,
        right: 16,
        start: KeyIndex { group: 2, key: 39 },
    },
];

// All of the supported LTNS.
static LTNS: &'static [Ltn] = &[
    Ltn {
        tuning: &EDO12,
        fills: SPLIT_FILL,
        name: "dlb-edo12-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO19,
        fills: SPLIT_FILL,
        name: "dlb-edo19-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: SPLIT_FILL,
        name: "dlb-edo31-wicki-hayden-split",
    },
    Ltn {
        tuning: &EDO31,
        fills: WIDE_FILL,
        name: "dlb-edo31-wicki-hayden-wide",
    },
];

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.

    // Generate all of the layouts.
    for ltn in LTNS {
        let mut keyb = Keyboard::default();
        for fill in ltn.fills {
            keyb.fill_layout(ltn.tuning, &WICKI_HAYDEN, fill);
        }

        keyb.write_svg(format!("{}.svg", ltn.name))?;
    }

    Ok(())
}
