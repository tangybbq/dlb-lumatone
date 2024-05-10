//! Management of lumatone mappings
//!
//! The [lumatone](https://lumatone.io/) keyboard is an isomorphic hex-grid
//! music keyboard.  It has 208 keys, which are grouped into 5 groups of 56 keys
//! each.  The grid is rotated counterclockwise about 8.9 degrees, which makes a
//! common 3+4 staggered pattern, where going to the right by 8 keys, and down
//! to the right by 2 ends up at a key that is even.  For the Bosenquet-Wilson
//! layout, this results in the octaves being horizontal across the keyboard.

#![allow(dead_code)]

use std::{collections::BTreeMap, path::Path};

use anyhow::Result;

mod svg;

/// The lumatone itself represents the keys by a pair of numbers, the group, a
/// number between 0 and 4, and the key itself, a number between 0 and 56.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KeyIndex {
    /// Group across the keyboard.
    pub group: u8,
    /// Key within the group.
    pub key: u8,
}

impl KeyIndex {
    pub fn origin() -> KeyIndex {
        KeyIndex { group: 0, key: 0 }
    }
}

/// A delta move.  Indicates A movement.
#[derive(Debug, Copy, Clone)]
struct KeyMove {
    pub group: i8,
    pub key: u8,
}

struct MoveMap(BTreeMap<Dir, Vec<Option<KeyMove>>>);

/// Each key has the following information associated with it.
#[derive(Debug, Default, Clone)]
pub struct KeyInfo {
    /// The midi channel to send for this key.
    pub channel: u8,
    /// The midi not number to send for this key.
    pub note: u8,
    /// The color representing this key.
    pub color: RGB8,
}

/// The entire keyboard.
#[derive(Debug, Clone)]
pub struct Keyboard {
    pub keys: [[KeyInfo; 56]; 5],
}

/// For now, just use a local RGB8.  This should match other definitions.
#[derive(Debug, Default, Clone, Copy)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB8 {
    pub fn new(r: u8, g: u8, b: u8) -> RGB8 {
        RGB8 { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub const fn white() -> RGB8 {
        RGB8 { r: 255, g: 255, b: 255 }
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        let a: [KeyInfo; 56] = std::array::from_fn(|_| Default::default());
        let b = a.clone();
        let c = a.clone();
        let d = a.clone();
        let e = a.clone();
        Keyboard {
            // As of rust 1.78, Default is only implemented for arrays up to 32.
            // keys: Default::default(),
            keys: [a, b, c, d, e],
        }
    }
}

impl Keyboard {
    pub fn write_svg<P: AsRef<Path>>(&self, p: P) -> Result<()> {
        let mut writer = svg::SvgOut::new();
        let mv = MoveMap::make();

        let mut row_start = KeyIndex::origin();
        let mut last_x0 = 0;

        for (y, &(x0, xlen)) in SIZES.iter().enumerate() {
            // Move to the new position.
            if y > 0 {
                // Move to the right before down, so we always stay within the
                // keyboard.
                while x0 > last_x0 {
                    row_start = mv.trymove(row_start, Dir::Right).unwrap();
                    last_x0 += 1;
                }

                // Now move down.
                let dir = if (y & 1) == 1 {
                    Dir::DownRight
                } else {
                    Dir::DownLeft
                };
                row_start = mv.trymove(row_start, dir)
                    .unwrap();
                // println!("Move {:?} to {:?}", dir, row_start);
            }

            let mut key = row_start.clone();
            for x in x0..x0 + xlen {
                if x > x0 {
                    // If this fails, our table of positions is wrong.
                    // println!("At {:?} move right", key);
                    key = mv.trymove(key, Dir::Right).unwrap();
                }
                match self.get(key) {
                    Some(info) => {
                        let label = format!("{},{}", key.group, key.key);
                        writer.add(x, y as u32, /*info.color*/ RGB8::white(), &label);
                    }
                    None => {
                        writer.add(x, y as u32, RGB8::white(), "");
                    }
                }
            }
        }

        writer.save(p)
    }

    pub fn get(&self, index: KeyIndex) -> Option<&KeyInfo> {
        self.keys.get(index.group as usize).and_then(|k| k.get(index.key as usize))
    }
}

// A single section of the lumatone keyboard is layed out like this (without the
// tilt)). The pipes indicate the next section.
//
// 00  01
//   02  03  04  05  06
// 07  08  09  10  11  12| 00  01  ...
//   13  14  15  16  17  18| 02  03 ...
// 19  20  21  22  23  24| 07  08
//   25  26  27  28  29  30| 13  14 ...
// 31  32  33  34  35  36| 19  20 ...
//   37  38  39  40  41  42| 25  26 ...
// 43  44  45  46  47  48| 31  32 ...
//       49  50  51  52  53| 37  38 ...
//                 54  55| 43  44

/// A direction we can move along the keyboard.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub enum Dir {
    UpLeft, UpRight, Right, DownRight, DownLeft, Left,
}

impl MoveMap {
    /// Construct the key movement mapping.
    fn make() -> MoveMap {
        let mut result = BTreeMap::new();

        // Movement to the right.
        let mut right = Vec::with_capacity(56);
        for i in 0..56 {
            right.push(Some(KeyMove {group: 0, key: i + 1}));
        }
        // Two of the keys have nothing to the right.
        right[1] = None;
        right[6] = None;
        // And several of the keys move to another group.
        right[12] = Some(KeyMove { group: 1, key: 0 });
        right[18] = Some(KeyMove { group: 1, key: 2 });
        right[24] = Some(KeyMove { group: 1, key: 7 });
        right[30] = Some(KeyMove { group: 1, key: 13 });
        right[36] = Some(KeyMove { group: 1, key: 19 });
        right[42] = Some(KeyMove { group: 1, key: 25 });
        right[48] = Some(KeyMove { group: 1, key: 31 });
        right[53] = Some(KeyMove { group: 1, key: 37 });
        right[55] = Some(KeyMove { group: 1, key: 43 });

        result.insert(Dir::Right, right);

        // Movement to the left.
        let mut left = Vec::with_capacity(56);
        left.push(None);
        for i in 1..56 {
            left.push(Some(KeyMove {group: 0, key: i - 1}));
        }
        // Keys with nothing to the left.
        left[49] = None;
        left[54] = None;
        // And the ones that move to a new group.
        left[0] = Some(KeyMove { group: -1, key: 12 });
        left[2] = Some(KeyMove { group: -1, key: 18 });
        left[7] = Some(KeyMove { group: -1, key: 24 });
        left[13] = Some(KeyMove { group: -1, key: 30 });
        left[19] = Some(KeyMove { group: -1, key: 36 });
        left[25] = Some(KeyMove { group: -1, key: 42 });
        left[31] = Some(KeyMove { group: -1, key: 48 });
        left[37] = Some(KeyMove { group: -1, key: 53 });
        left[43] = Some(KeyMove { group: -1, key: 55 });

        result.insert(Dir::Left, left);

        // Movement down and right.
        let mut dr: Vec<_> = [
            /* 00 */ 2, 3,
            /* 02 */ 8, 9, 10, 11, 12,
            /* 07 */ 13, 14, 15, 16, 17, 18,
            /* 13 */ 20, 21, 22, 23, 24, 7,
            /* 19 */ 25, 26, 27, 28, 29, 30,
            /* 25 */ 32, 33, 34, 35, 36, 19,
            /* 31 */ 37, 38, 39, 40, 41, 42,
            /* 37 */ 44, 45, 46, 47, 48, 31,
            /* 43 */ 0, 49, 50, 51, 52, 53,
            /* 49 */ 0, 0, 54, 55, 43,
            /* 54 */ 0, 0,
        ].iter().map(|&n| Some(KeyMove { group: 0, key: n })).collect();
        // Keys with no movement DR.
        dr[43] = None;
        dr[49] = None;
        dr[50] = None;
        dr[54] = None;
        dr[55] = None;
        // Moves to next group.
        dr[18].as_mut().map(|k| k.group = 1);
        dr[30].as_mut().map(|k| k.group = 1);
        dr[42].as_mut().map(|k| k.group = 1);
        dr[53].as_mut().map(|k| k.group = 1);

        result.insert(Dir::DownRight, dr);

        // Movement up and left.
        let mut ul: Vec<_> = [
            /* 00 */ 0, 0,
            /* 02 */ 0, 1, 0, 0, 0,
            /* 07 */ 18, 2, 3, 4, 5, 6,
            /* 13 */ 7, 8, 9, 10, 11, 12,
            /* 19 */ 30, 13, 14, 15, 16, 17,
            /* 25 */ 19, 20, 21, 22, 23, 24,
            /* 31 */ 42, 25, 26, 27, 28, 29,
            /* 37 */ 31, 32, 33, 34, 35, 36,
            /* 43 */ 53, 37, 38, 39, 40, 41,
            /* 49 */ 44, 45, 46, 47, 48,
            /* 54 */ 51, 52,
        ].iter().map(|&n| Some(KeyMove { group: 0, key: n })).collect();
        // No movement UL
        ul[0] = None;
        ul[1] = None;
        ul[4] = None;
        ul[5] = None;
        ul[6] = None;
        // Moves to previous group.
        ul[7].as_mut().map(|k| k.group = -1);
        ul[19].as_mut().map(|k| k.group = -1);
        ul[31].as_mut().map(|k| k.group = -1);
        ul[43].as_mut().map(|k| k.group = -1);

        result.insert(Dir::UpLeft, ul);

        // Movement down and left.
        let mut dl: Vec<_> = [
            /* 00 */ 18, 2,
            /* 02 */ 7, 8, 9, 10, 11,
            /* 07 */ 30, 13, 14, 15, 16, 17,
            /* 13 */ 19, 20, 21, 22, 23, 24,
            /* 19 */ 42, 25, 26, 27, 28, 29,
            /* 25 */ 31, 32, 33, 34, 35, 36,
            /* 31 */ 53, 37, 38, 39, 40, 41,
            /* 37 */ 43, 44, 45, 46, 47, 48,
            /* 43 */ 0, 0, 49, 50, 51, 52,
            /* 49 */ 0, 0, 0, 54, 55,
            /* 54 */ 0, 0,
        ].iter().map(|&n| Some(KeyMove { group: 0, key: n })).collect();
        // Keys with no movement DR.
        dl[43] = None;
        dl[44] = None;
        dl[49] = None;
        dl[50] = None;
        dl[51] = None;
        dl[54] = None;
        dl[55] = None;
        // Moves to next group.
        dl[0].as_mut().map(|k| k.group = 1);
        dl[7].as_mut().map(|k| k.group = 1);
        dl[19].as_mut().map(|k| k.group = 1);
        dl[31].as_mut().map(|k| k.group = 1);

        result.insert(Dir::DownLeft, dl);

        // Movement up and right.
        let mut ur: Vec<_> = [
            /* 00 */ 0, 0,
            /* 02 */ 1, 0, 0, 0, 0,
            /* 07 */ 2, 3, 4, 5, 6, 0,
            /* 13 */ 8, 9, 10, 11, 12, 0,
            /* 19 */ 13, 14, 15, 16, 17, 18,
            /* 25 */ 20, 21, 22, 23, 24, 7,
            /* 31 */ 25, 26, 27, 28, 29, 30,
            /* 37 */ 32, 33, 34, 35, 36, 19,
            /* 43 */ 37, 38, 39, 40, 41, 42,
            /* 49 */ 45, 46, 47, 48, 31,
            /* 54 */ 52, 53,
        ].iter().map(|&n| Some(KeyMove { group: 0, key: n })).collect();
        // No movement UL
        ur[0] = None;
        ur[1] = None;
        ur[3] = None;
        ur[4] = None;
        ur[5] = None;
        ur[6] = None;
        ur[12] = None;
        // Moves to previous group.
        ur[18].as_mut().map(|k| k.group = -1);
        ur[30].as_mut().map(|k| k.group = -1);
        ur[42].as_mut().map(|k| k.group = -1);
        ur[53].as_mut().map(|k| k.group = -1);

        result.insert(Dir::UpRight, ur);


        MoveMap(result)
    }

    // Return the new key from this movement.
    fn trymove(&self, key: KeyIndex, dir: Dir) -> Option<KeyIndex> {
        // self.0.get(&dir).and_then(|moves| moves.get(key.key as usize))
        self.0
            .get(&dir)
            .and_then(|moves| moves[key.key as usize])
            .and_then(|mv| {
                // Check if we move off of the edge.
                if (mv.group < 0 && key.group == 0) ||
                    (mv.group > 0 && key.group == 5)
                {
                    None
                } else {
                    Some(KeyIndex {
                        group: (key.group as i8 + mv.group) as u8,
                        key: mv.key,
                    })
                }
            })
    }
}

impl Keyboard {
    /// Attempt to load a keyboard from a .ltn file.
    pub fn load<P: AsRef<Path>>(_path: P) -> Result<Keyboard> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::Dir;
    use super::KeyIndex;
    use super::Keyboard;
    use super::MoveMap;

    impl MoveMap {
        /// Verify that all movements in direction 'a' and then 'b' get back to the same place.
        fn check(&self, a: Dir, b: Dir) {
            for k1 in KeyIndex::iter_all() {
                match self.trymove(k1, a) {
                    None => {
                        // Ensure that the are no other keys, anywhere, that land on this one.
                        for kk in KeyIndex::iter_all() {
                            match self.trymove(kk, b) {
                                Some(k2) if k2 == k1 =>
                                    panic!("Move {:?} {:?} is None, but {:?} {:?} goes back.",
                                           k1, a, kk, b),
                                _ => (),
                            }
                        }
                    },
                    Some(k2) => {
                        match self.trymove(k2, b) {
                            None => panic!("Move {:?} {:?} went to {:?} but {:?} did not move back",
                                           k1, a, k2, b),
                            Some(k3) if k1 != k3 =>
                                panic!("Move {:?} {:?} went to {:?}, but {:?} to {:?}",
                                       k1, a, k2, b, k3),
                            Some(_) => {
                                // Ensure only one key lands on this one.
                                for kk in KeyIndex::iter_all() {
                                    if kk == k2 {
                                        continue;
                                    }
                                    match self.trymove(kk, b) {
                                        Some(kc) if kc == k1 =>
                                            panic!("Multiple keys move to {:?}, {:?} and {:?}",
                                                   k1, k2, kk),
                                        _ => (),
                                    }
                                }
                            }
                            /*
                            Some(k3) => println!("{:?} {:?} to {:?}, and {:?} to {:?}",
                            k1, a, k2, b, k3),
                             */
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn lumatone_default() {
        let keyb = Keyboard::default();
        println!("{:?}", keyb);
        // todo!()
    }

    /// Test keymovement.
    #[test]
    fn lumatone_move_consistent() {
        let mv = MoveMap::make();
        mv.check(Dir::Left, Dir::Right);
        mv.check(Dir::Right, Dir::Left);
        mv.check(Dir::UpLeft, Dir::DownRight);
        mv.check(Dir::DownRight, Dir::UpLeft);
        mv.check(Dir::UpRight, Dir::DownLeft);
        mv.check(Dir::DownLeft, Dir::UpRight);
    }
}

impl KeyIndex {
    fn iter_all() -> KeyIndexIter {
        KeyIndexIter {
            group: 0,
            key: 0,
        }
    }
}

struct KeyIndexIter {
    group: u8,
    key: u8,
}

impl Iterator for KeyIndexIter {
    type Item = KeyIndex;
    fn next(&mut self) -> Option<KeyIndex> {
        if self.group > 5 {
            return None;
        }
        let result = KeyIndex { group: self.group, key: self.key };
        if self.key == 55 {
            self.key = 0;
            self.group += 1;
        } else {
            self.key += 1;
        }
        Some(result)
    }
}

/// The offset and sizes of each for each row of the lumatone.
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
