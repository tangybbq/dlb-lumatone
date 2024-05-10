use anyhow::Result;
use microtone::{lumatone::{FillInfo, KeyIndex, Keyboard, WICKI_HAYDEN}, tuning::{EDO12, EDO19, EDO31}};

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    let mut keyb = Keyboard::default();

    // Generate the reference.
    // keyb.fill_reference();

    // 12 and 19.
    if true {
        let tuning = if false {
            &EDO12
        } else if false {
            &EDO19
        } else {
            &EDO31
        };

        keyb.fill_layout(tuning, &WICKI_HAYDEN, FillInfo {
            left: 9,
            right: 9,
            start: KeyIndex { group: 3, key: 47 },
        });
        keyb.fill_layout(tuning, &WICKI_HAYDEN, FillInfo {
            left: 9,
            right: 9,
            start: KeyIndex { group: 1, key: 14 },
        });
    } else {
        let tuning = if true {
            &EDO31
        } else {
            // 53?
            todo!()
        };

        keyb.fill_layout(tuning, &WICKI_HAYDEN, FillInfo {
            left: 16,
            right: 16,
            start: KeyIndex { group: 2, key: 39 },
        });
    }

    keyb.write_svg("image.svg")?;

    Ok(())
}
