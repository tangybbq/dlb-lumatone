//! Lumatone SVG mapping generation.

use anyhow::Result;
use svg::{node::element::{path::Data, Path}, Document};
use std::f32::consts;

// The Lumatone keyboard consists of a regular grid of hexagons, alternate rows
// being offset by `SPACING/2.0`.

/// The distance between keys in the diagram.
const SPACING: f32 = 10.0;

/// The overall rotation of the grid.  I'm not sure why this needs a factor of 2, and this still doesn't seem quite right.
const TILT: f32 = 8.948_f32 * 2.0 / 360.0 * (2.0 * consts::PI);
// const TILT: f32 = 0.0;
// Note that to_radians() is not currently const.

/// An SVG generator for a lumatone keyboard type of layout.
pub struct SvgOut {
}

impl SvgOut {
    pub fn new() -> SvgOut {
        SvgOut {
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, p: P) -> Result<()> {
        let mut document = Document::new()
            .set("viewBox", (-20, -20, 36.0 * SPACING, 20.0 * SPACING));

        for (y, &(x0, xlen)) in SIZES.iter().enumerate() {
            for x in x0..x0 + xlen {
                document = document.add(self.make_hex(x, y as u32));
            }
        }

        svg::save(p, &document)?;
        Ok(())
    }

    /// Generate a path element for a basic hexagon.
    fn make_hex(&self, x: u32, y: u32) -> Path {
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
            .set("fill", "#f0e0d0")
            .set("stroke", "black")
            .set("stroke-width", 0.3)
            .set("d", data)
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

/// The offset and sizes of each for.
static SIZES: [(u32, u32); 19] = [
    (0, 2),
    (0, 5),
    (0, 8),
    (0, 11),
    (0, 14),
    (0, 17),
    (0, 20),
    (0, 23),
    (0, 26),
    (1, 28),
    (4, 26),
    (7, 23),
    (10, 20),
    (13, 17),
    (16, 14),
    (19, 11),
    (22, 8),
    (25, 5),
    (28, 2),
    ];

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

    svg::save("image.svg", &document).unwrap();
}
