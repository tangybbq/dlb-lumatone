use anyhow::Result;
use microtone::{lumatone::{FillInfo, KeyIndex, Keyboard, WICKI_HAYDEN}, tuning::EDO12};

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    let mut keyb = Keyboard::default();

    // Generate the reference.
    // keyb.fill_reference();

    keyb.fill_layout(&EDO12, &WICKI_HAYDEN, FillInfo {
        left: 8,
        right: 7,
        start: KeyIndex { group: 1, key: 14 },
    });
    keyb.fill_layout(&EDO12, &WICKI_HAYDEN, FillInfo {
        left: 7,
        right: 7,
        start: KeyIndex { group: 3, key: 47 },
    });

    keyb.write_svg("image.svg")?;

    Ok(())
}
