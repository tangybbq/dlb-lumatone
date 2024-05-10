use anyhow::Result;
use microtone::lumatone::Keyboard;

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    let keyb = Keyboard::default();
    keyb.write_svg("image.svg")?;

    Ok(())
}
