use anyhow::Result;
use microtone::{lumatone::{FillInfo, KeyIndex, Keyboard, WICKI_HAYDEN}, tuning::{EDO12, EDO19}};

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    let mut keyb = Keyboard::default();

    // let tuning = &EDO12;
    let tuning = &EDO19;

    // Generate the reference.
    // keyb.fill_reference();

    keyb.fill_layout(tuning, &WICKI_HAYDEN, FillInfo {
        left: 8,
        right: 8,
        start: KeyIndex { group: 1, key: 14 },
    });
    keyb.fill_layout(tuning, &WICKI_HAYDEN, FillInfo {
        left: 7,
        right: 8,
        start: KeyIndex { group: 3, key: 47 },
    });

    keyb.write_svg("image.svg")?;

    Ok(())
}
