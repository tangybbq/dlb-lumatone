use std::fs::create_dir;

use anyhow::{anyhow, Result};
use microtone::{config::Config, lumatone::Keyboard};

fn main() -> Result<()> {
    let cfg = Config::load("config".as_ref())?;

    let _ = create_dir("layouts");

    // Write the reference layout map (key numbers and section colors).
    let mut keyb = Keyboard::default();
    keyb.fill_reference();
    keyb.write_svg("layouts/lumatone-layout.svg")?;

    // Generate every configured output.
    for out in &cfg.outputs {
        // These were all validated by Config::load, so the lookups cannot fail.
        let tuning = cfg.tuning(&out.tuning)
            .ok_or_else(|| anyhow!("unknown tuning {:?}", out.tuning))?;
        let layout = cfg.layout(&out.layout)
            .ok_or_else(|| anyhow!("unknown layout {:?}", out.layout))?;
        let fills = cfg.fill(&out.fill)
            .ok_or_else(|| anyhow!("unknown fill {:?}", out.fill))?;

        let mut keyb = Keyboard::default();
        for fill in fills {
            keyb.fill_layout(tuning, layout, fill);
        }

        let _ = create_dir(format!("layouts/{}", out.name));
        keyb.write_svg(format!("layouts/{}/{}.svg", out.name, out.name))?;
        keyb.write_ltn(format!("layouts/{}/{}.ltn", out.name, out.name))?;
    }

    Ok(())
}
