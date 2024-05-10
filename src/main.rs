use anyhow::Result;
use microtone::lumatone::Keyboard;

fn main() -> Result<()> {
    // For now, just generate a keyboard, to view the SVG.
    let mut keyb = Keyboard::default();
    keyb.fill_reference();
    keyb.write_svg("image.svg")?;

    Ok(())
}
