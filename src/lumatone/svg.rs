//! Lumatone SVG mapping generation.

use anyhow::Result;
use svg::{node::element::{path::Data, Path, Style, Text}, Document};
use std::f32::consts;

use super::RGB8;

// The Lumatone keyboard consists of a regular grid of hexagons, alternate rows
// being offset by `SPACING/2.0`.

/// The distance between keys in the diagram.
const SPACING: f32 = 10.0;

/// The overall rotation of the grid.  I'm not sure why this needs a factor of 2, and this still doesn't seem quite right.
// const TILT: f32 = 8.948_f32 * 2.0 / 360.0 * (2.0 * consts::PI);
const TILT: f32 = 16.0 / 360.0 * (2.0 * consts::PI);
// const TILT: f32 = 0.0;
// Note that to_radians() is not currently const.

/// An SVG generator for a lumatone keyboard type of layout.
pub struct SvgOut {
    keys: Vec<Path>,
    labels: Vec<Text>,
}

impl SvgOut {
    pub fn new() -> SvgOut {
        SvgOut {
            keys: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// Add a single key, with a given color and label.
    pub fn add(&mut self, x: u32, y: u32, color: RGB8, label: &str) {
        self.keys.push(self.make_hex(x, y, color));
        self.labels.push(self.make_text(x, y, label));
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, p: P) -> Result<()> {
        let mut document = Document::new()
            .set("viewBox", (-20, -20, 36.0 * SPACING, 20.0 * SPACING));

        document = document.add(Style::new(
            r".black { font: 3px serif; }"
            ));

        // TODO: Save could be `self` and we wouldn't need to clone.
        for key in &self.keys {
            document = document.add(key.clone());
        }
        for label in &self.labels {
            document = document.add(label.clone());
        }

        svg::save(p, &document)?;
        Ok(())
    }

    /// Generate a path element for a basic hexagon.
    fn make_hex(&self, x: u32, y: u32, color: RGB8) -> Path {
        let (x, y) = self.coord(x, y);
        let mut data = Data::new();

        // SPACING is the distance to the edge, calculate the distance to the corners.
        let corner = SPACING / (3_f32.sqrt() / 2.0);
        for i in 0..6 {
            let angle = 2.0 * consts::PI / 6.0 * (i as f32) + TILT;
            let dx = corner / 2.0 * angle.sin();
            let dy = corner / 2.0 * angle.cos();
            if i == 0 {
                data = data.move_to((x + dx, y + dy));
            } else {
                data = data.line_to((x + dx, y + dy));
            }
        }
        data = data.close();

        // TODO: Come up with better parameters.
        Path::new()
            .set("fill", color.lighten().to_hex())
            .set("stroke", "black")
            .set("stroke-width", 0.3)
            .set("d", data)
    }

    /// Generate a text element labeling a given box.
    fn make_text(&self, x: u32, y: u32, text: &str) -> Text {
        let (x, y) = self.coord(x, y);
        Text::new(text)
            .set("class", "black")
            .set("x", x)
            .set("y", y)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
    }

    /// Given a coordinate, return the X and Y coordinates of that in SVG space.
    /// The Y coordinate for odd rows will be shifted to the right.
    fn coord(&self, x: u32, y: u32) -> (f32, f32) {
        let x = x as f32 * SPACING + ((y % 2) as f32) * (SPACING / 2.0);
        let y = y as f32 * SPACING * 3_f32.sqrt() / 2.0;

        // Use the negation of TILT, as Y coordinates are downward.
        let tilt = -TILT;
        (x * tilt.cos() - y * tilt.sin(),
         x * tilt.sin() + y * tilt.cos())
    }
}

#[test]
fn gen() {
    let data = Data::new()
        .move_to((10, 10))
        .line_by((0, 45.5))
        .line_by((50, 0))
        .line_by((0, -40))
        .close();

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1.5)
        .set("d", data);

    let document = Document::new()
        .set("viewBox", (0, 0, 70, 70))
        .add(path);

    let _ = document;
    // svg::save("image.svg", &document).unwrap();
}
