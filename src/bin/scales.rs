//! Scale generator.

// Accidentals for copy/paste.
// ♭ ♯ 𝄫 𝄪
// Unicode doesn't seem to have half sharp and half flat, so we'll use ^ and v.

use std::collections::HashSet;

use microtone::lumatone::{SvgOut, RGB8};

fn main() -> anyhow::Result<()> {
    if true {
        let mut out = SvgOut::new();
        for (i, note) in WICKI_HAYDEN_31.iter().enumerate() {
            // No matter what bias is chosen, make sure it is odd, or the rows will shift weirdly.
            out.add(
                (14 + note.x) as u32,
                (9 + note.y) as u32,
                RGB8::white(),
                &format!("{}", i),
            );
        }

        out.save("scale.svg")?;
    }

    for scale in SCALES {
        let mut out = SvgOut::new();
        /*
        out.add(
            14, 9, RGB8::white(), "0",
        );
        */

        let mut seen = HashSet::new();
        let mut pitch = 0;
        let mut index = 1;
        let mut min_x = 14;
        let mut max_x = 14;
        let mut min_y = 9;
        let mut max_y = 9;
        for interval in scale.intervals {
            pitch = (pitch + *interval as usize) % 31;
            let coord = &WICKI_HAYDEN_31[pitch];

            let x = (14 + coord.x) as u32;
            let y = (9 + coord.y) as u32;

            out.add(
                x,
                y,
                RGB8::new(250, 250, 250),
                &format!("{}", index % scale.intervals.len()),
            );
            index += 1;
            seen.insert(((14 + coord.x) as u32, (9 + coord.y) as u32));

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        // Make placeholders for all of the keys in between.  This is actually way too many cells,
        // and we really just want the cells "between" the ones we've seen.  Will need to think
        // about this algorithm, especially with 3 axes.
        /*
        for y in min_y..max_y + 1 {
            for x in min_x..max_x + 1 {
                if !seen.contains(&(x, y)) {
                    out.add(x, y, RGB8::white(), "");
                }
            }
        }
        */

        out.save(&format!("scales/{}", scale.file))?;
        let _ = scale.name;
    }

    Ok(())
}

static SCALES: &'static [Scale] = &[
    Scale {
        file: "meantone-5.svg",
        name: "Meantone[5]",
        intervals: &[5, 5, 8, 5, 8],
    },
    Scale {
        file: "meantone-7.svg",
        name: "Meantone[7]",
        intervals: &[5, 5, 3, 5, 5, 5, 3],
    },
    Scale {
        file: "meantone-12.svg",
        name: "Meantone[12]",
        intervals: &[3, 2, 3, 2, 3, 2, 3, 3, 2, 3, 2, 3],
    },
    Scale {
        file: "orwell-9.svg",
        name: "Orwell[9]",
        intervals: &[4, 3, 4, 3, 4, 3, 4, 3, 3],
    },
    Scale {
        file: "mos-54.svg",
        name: "MOS[5-4]",
        intervals: &[5, 4, 5, 4, 5, 4, 4],
    },
];

struct Scale {
    file: &'static str,
    name: &'static str,
    intervals: &'static [u8],
}

// Placement map for Wicki-Hayden EDO-31.
static WICKI_HAYDEN_31: [Coord; 31] = [
    Coord::new(0, 0), // 0
    Coord::new(-6, -2), // 1
    Coord::new(4, 1), // 2
    Coord::new(-2, -1), // 3
    Coord::new(7, 2), // 4
    Coord::new(1, 0), // 5
    Coord::new(-5, -2), // 6
    Coord::new(5, 1), // 7
    Coord::new(-1, -1), // 8
    Coord::new(-7, -3), // 9,  Also 8, 2 which, disturbingly is the same distance.
    Coord::new(2, 0), // 10
    Coord::new(-4, -2), // 11
    Coord::new(6, 1), // 12
    Coord::new(0, -1), // 13
    Coord::new(-6, -3), // 14
    Coord::new(3, 0), // 15
    Coord::new(-3, -2), // 16
    Coord::new(7, 1), // 17
    Coord::new(1, -1), // 18
    Coord::new(-5, -3), // 19
    Coord::new(4, 0), // 20
    Coord::new(-2, -2), // 21
    Coord::new(8, 1), // 22
    Coord::new(2, -1), // 23
    Coord::new(-4, -3), // 24
    Coord::new(5, 0), // 25
    Coord::new(-1, -2), // 26
    Coord::new(-7,-4), // 27
    Coord::new(3, -1), // 28
    Coord::new(-3, -3), // 29
    Coord::new(6, 0), // 30
];

struct Coord {
    x: isize,
    y: isize,
}

impl Coord {
    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}
