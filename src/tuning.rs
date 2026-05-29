//! Management of tuning systems.
//!
//! Manage tuning systems, and the various ways that they deal with names of
//! notes, and midi note/channel numbers.

use crate::lumatone::RGB8;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MidiNote {
    pub channel: u8,
    pub note: u8,
}

/// A few intervals that are used for building keyboards.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntervalStep {
    AugUnison,
    MinorSecond,
    MajorSecond,
    MinorThird,
    MajorThird,
    PerfectFourth,
    // In 12-EDO, these two are the same, but they differ in other tunings.
    AugmentedFourth,
    DimishedFifth,
    PerfectFifth,
    // If these are valid in the tuning, otherwise they are just a regular Major Second and Third.
    NeutralSecond,
    NeutralThird,
}

/// Which direction does an interval move in?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntervalDirection {
    Up,
    Down,
}

impl IntervalDirection {
    pub fn flip(self) -> IntervalDirection {
        match self {
            IntervalDirection::Up => IntervalDirection::Down,
            IntervalDirection::Down => IntervalDirection::Up,
        }
    }
}

/// An Interval itself is a step and direction.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Interval {
    step: IntervalStep,
    direction: IntervalDirection,
}

impl Interval {
    pub const fn new(step: IntervalStep, direction: IntervalDirection) -> Interval {
        Interval { step, direction }
    }

    /// Is this an "up" interval.
    pub fn is_up(self) -> bool {
        self.direction == IntervalDirection::Up
    }

    /// Flip this interval, so that up and down are reversed.
    pub fn flip(self) -> Interval {
        Interval {
            step: self.step,
            direction: self.direction.flip(),
        }
    }
}

/// A tuning system, at least as much information as is needed to produce a
/// keyboard layout and midi mapping.  Right now, the midi mapping is definitive.
pub trait Tuning {
    /// Number of steps in an octave for this tuning.
    fn octave(&self) -> usize;

    /// Adjust a note by a raw, signed number of steps. Positive raises the
    /// pitch, negative lowers it. None indicates the result is out of range.
    fn step(&self, note: MidiNote, steps: isize) -> Option<MidiNote>;

    /// Resolve a named interval to a signed step count for this tuning.
    fn interval_steps(&self, interval: Interval) -> isize {
        let steps = self.get_steps(interval.step);
        if interval.is_up() { steps } else { -steps }
    }

    /// Adjust a note by an interval. None indicates either the note is out of
    /// range, or the interval doesn't make sense with this tuning.
    fn interval(&self, note: MidiNote, interval: Interval) -> Option<MidiNote> {
        self.step(note, self.interval_steps(interval))
    }

    /// Return a nice name for this note. The 'sharp' hint is for tuning systems
    /// that have enharmonic sharps and flats, as a suggestion of which name to
    /// use.
    fn name(&self, note: MidiNote, sharp: bool) -> String;

    /// For tunings where intervals are independ (the EDOs), interval can be
    /// calculated just based on a number of steps. This should be None if this
    /// doesn't make sense, and the implementer should define their own
    /// `interval` method.
    fn get_steps(&self, interval: IntervalStep) -> isize;

    /// Guess a good color for this particular note.
    fn color(&self, note: MidiNote, sharp: bool) -> RGB8;

    /// Return middle C for this tuning.
    fn middle_c(&self) -> MidiNote;
}

/// A general Equal division of the octave.
pub struct Edo {
    /// Number of steps in an octave.
    octave: usize,
    /// Does this Edo use the channel number as the octave. None indicates no,
    /// and Some(n) indicates yes, with 'n' as the note number bias. C in a
    /// given octave will be this note number, with the rest of the octave above
    /// that.
    channel_octaves: Option<usize>,
    /// Middle C.
    middle_c: MidiNote,

    /// The values of the intervals for this Edo.
    intervals: &'static [isize],

    /// Names of the pitches, with sharp bias.
    sharp_names: &'static [&'static str],
    flat_names: &'static [&'static str],
}

pub static EDO12: Edo = Edo {
    octave: 12,
    channel_octaves: None,
    middle_c: MidiNote { channel: 1, note: 60 },
    intervals: EDO12_INTERVALS.as_slice(),
    sharp_names: EDO12_SHARP_NAMES.as_slice(),
    flat_names: EDO12_FLAT_NAMES.as_slice(),
};

static EDO12_INTERVALS: [isize; 11] = [
    1, 1, 2, 3, 4, 5, 6, 6, 7, 2, 4,
];

static EDO12_SHARP_NAMES: [&'static str; 12] = [
    "C",
    "C♯",
    "D",
    "D♯",
    "E",
    "F",
    "F♯",
    "G",
    "G♯",
    "A",
    "A♯",
    "B",
];

static EDO12_FLAT_NAMES: [&'static str; 12] = [
    "C",
    "D♭",
    "D",
    "E♭",
    "E",
    "F",
    "G♭",
    "G",
    "A♭",
    "A",
    "B♭",
    "B",
];

pub static EDO17: Edo = Edo {
    octave: 17,
    channel_octaves: Some(60),
    middle_c: MidiNote { channel: 4, note: 60 },
    intervals: EDO17_INTERVALS.as_slice(),
    sharp_names: EDO17_SHARP_NAMES.as_slice(),
    flat_names: EDO17_FLAT_NAMES.as_slice(),
};

static EDO17_INTERVALS: [isize; 11] = [
    2, 1, 3, 4, 6, 7, 9, 8, 10, 3, 6,
];

static EDO17_SHARP_NAMES: [&'static str; 17] = [
    "C",
    "D♭",
    "C♯",
    "D",
    "E♭",
    "D♯",
    "E",
    "F",
    "G♭",
    "F♯",
    "G",
    "A♭",
    "G♯",
    "A",
    "B♭",
    "A♯",
    "B",
];

static EDO17_FLAT_NAMES: [&'static str; 17] = [
    "C",
    "D♭",
    "C♯",
    "D",
    "E♭",
    "D♯",
    "E",
    "F",
    "G♭",
    "F♯",
    "G",
    "A♭",
    "G♯",
    "A",
    "B♭",
    "A♯",
    "B",
];

pub static EDO19: Edo = Edo {
    octave: 19,
    channel_octaves: Some(60),
    middle_c: MidiNote { channel: 4, note: 60 },
    intervals: EDO19_INTERVALS.as_slice(),
    sharp_names: EDO19_SHARP_NAMES.as_slice(),
    flat_names: EDO19_FLAT_NAMES.as_slice(),
};

static EDO19_INTERVALS: [isize; 11] = [
    1, 2, 3, 5, 6, 8, 9, 10, 11, 3, 6,
];

static EDO19_SHARP_NAMES: [&'static str; 19] = [
    "C",
    "C♯",
    "D♭",
    "D",
    "D♯",
    "E♭",
    "E",
    "E♯",
    "F",
    "F♯",
    "G♭",
    "G",
    "G♯",
    "A♭",
    "A",
    "A♯",
    "B♭",
    "B",
    "B♯",
];

static EDO19_FLAT_NAMES: [&'static str; 19] = [
    "C",
    "C♯",
    "D♭",
    "D",
    "D♯",
    "E♭",
    "E",
    "F♭",
    "F",
    "F♯",
    "G♭",
    "G",
    "G♯",
    "A♭",
    "A",
    "A♯",
    "B♭",
    "B",
    "C♭",
];

pub static EDO31: Edo = Edo {
    octave: 31,
    channel_octaves: Some(60),
    middle_c: MidiNote { channel: 4, note: 60 },
    intervals: EDO31_INTERVALS.as_slice(),
    sharp_names: EDO31_NAMES.as_slice(),
    flat_names: EDO31_NAMES.as_slice(),
};

static EDO31_INTERVALS: [isize; 11] = [
    2, 3, 5, 8, 10, 13, 15, 16, 18, 4, 9,
];

static EDO31_NAMES: [&'static str; 31] = [
    "C",
    "D𝄫",
    "C♯",
    "D♭",
    "C𝄪",
    "D",
    "E𝄫",
    "D♯",
    "E♭",
    "D𝄪",
    "E",
    "F♭",
    "E♯",
    "F",
    "G𝄫",
    "F♯",
    "G♭",
    "F𝄪",
    "G",
    "A𝄫",
    "G♯",
    "A♭",
    "G𝄪",
    "A",
    "B𝄫",
    "A♯",
    "B♭",
    "A𝄪",
    "B",
    "C♭",
    "B♯",
];

pub static EDO41: Edo = Edo {
    octave: 41,
    channel_octaves: Some(60),
    middle_c: MidiNote { channel: 4, note: 60 },
    intervals: EDO41_INTERVALS.as_slice(),
    sharp_names: EDO41_NAMES.as_slice(),
    flat_names: EDO41_NAMES.as_slice(),
};

static EDO41_INTERVALS: [isize; 11] = [
    4, 3, 7, 10, 14, 17, 21, 20, 24, 5, 12,
];

static EDO41_NAMES: [&'static str; 41] = [
    "C",
    "^C",
    "^^C",
    "D♭",
    "C♯",
    "vvD",
    "vD",
    "D",
    "^D",
    "^^D",
    "E♭",
    "D♯",
    "vvE",
    "vE",
    "E",
    "E",
    "vF",
    "F",
    "^F",
    "^^F",
    "G♭",
    "F♯",
    "vvG",
    "vG",
    "G",
    "^G",
    "^^G",
    "A♭",
    "G♯",
    "vvA",
    "vA",
    "A",
    "^A",
    "^^A",
    "B♭",
    "A♯",
    "vvB",
    "vB",
    "B",
    "^B",
    "vC",
];

pub static EDO53: Edo = Edo {
    octave: 53,
    channel_octaves: Some(1),
    middle_c: MidiNote { channel: 4, note: 1 },
    intervals: EDO53_INTERVALS.as_slice(),
    sharp_names: EDO53_NAMES.as_slice(),
    flat_names: EDO53_NAMES.as_slice(),
};

static EDO53_INTERVALS: [isize; 11] = [
    // Aug1
    5,
    // m2
    4,
    // M2
    9,
    // m3
    13,
    // M3
    18,
    // P4
    22,
    // Aug4
    25, // ??
    // Dim5
    25,
    // P5
    31,
    // N2
    5, // EDO53 doesn't have a neutral, but dupmajor 2 and 3rd fall half way between the fourth and
       // the fifth, so use them.
    // N3
    14,
];

static EDO53_NAMES: [&'static str; 53] = [
    // 0, C
    "C",
    "^C",
    "^^C",
    "vvC♯",
    "D♭",
    "C♯",
    "^^D♭",
    "vvD",
    "vD",
    // 9, D
    "D",
    "^D",
    "^^D",
    "vvD♯",
    "E♭",
    "D♯",
    "^^E♭",
    "vvE",
    "vE",
    // 18, E
    "E",
    "^E",
    "^^E", // "vvF",
    "vF",
    // 22, F
    "F",
    "^F",
    "^^F",
    "vvF♯",
    "G♭",
    "F♯",
    "^^G♭",
    "vvG",
    "vG",
    // 31, G
    "G",
    "^G",
    "^^G",
    "vvG♯",
    "A♭",
    "G♯",
    "^^A♭",
    "vvA",
    "vA",
    // 40, A
    "A",
    "^A",
    "^^A",
    "vvA♯",
    "B♭",
    "A♯",
    "^^B♭",
    "vvB",
    "vB",
    // 49, B
    "B",
    "^B",
    "^^B", // "vvC",
    "vC",
];

impl Tuning for Edo {
    fn get_steps(&self, interval: IntervalStep) -> isize {
        self.intervals[interval as usize]
    }

    fn octave(&self) -> usize {
        self.octave
    }

    fn step(&self, note: MidiNote, steps: isize) -> Option<MidiNote> {
        if let Some(bias) = self.channel_octaves {
            let bias = bias as isize;
            let octave = self.octave as isize;

            // Bias everything by 100 octaves.  This shouldn't be a problem even
            // with very fine tunings, and keeps the arithmetic positive.
            let pitch = (100 + note.channel as isize) * octave
                + (note.note as isize - bias)
                + steps;
            let oct = pitch.div_euclid(octave);
            if oct < 100 || oct > 227 {
                println!("Out of bound octave: {}", oct);
                return None;
            }
            let oct = oct - 100;
            let pitch = pitch.rem_euclid(octave) + bias;
            Some(MidiNote { channel: oct as u8, note: pitch as u8, })
        } else {
            let pitch = note.note as isize + steps;
            if pitch < 0 || pitch > 127 {
                return None;
            }
            Some(MidiNote { channel: note.channel, note: pitch as u8, })
        }
    }

    fn name(&self, note: MidiNote, sharp: bool) -> String {
        if let Some(bias) = self.channel_octaves {
            let pitch = note.note as usize - bias;
            let octave = note.channel;
            let names = if sharp { self.sharp_names } else { self.flat_names };
            format!("{}{}", names[pitch as usize], octave)
            // format!("{}-{}", octave, pitch)
        } else {
            // We assume that Middle C is C-4.
            let pitch = note.note as isize - self.middle_c.note as isize;
            let pitch = pitch + self.octave as isize * 4;
            let octave = pitch / (self.octave as isize);
            let pitch = pitch % (self.octave as isize);
            let names = if sharp { self.sharp_names } else { self.flat_names };
            format!("{}{}", names[pitch as usize], octave)
        }
    }

    /// To start with, just base the color on the length of the note, with a
    /// special case for C4.
    fn color(&self, note: MidiNote, sharp: bool) -> RGB8 {
        let name = self.name(note, sharp);
        if name == "C4" {
            return RGB8::new(150, 150, 192);
        }
        // Match names that start with 'C', but aren't accidentals.
        let mut iter = name.chars();
        if let Some(ch) = iter.next() {
            if ch == 'C' {
                if let Some(ch) = iter.next() {
                    if ch == '-' || ch.is_digit(10) {
                        return RGB8::new(192, 192, 65);
                    }
                }
            }
        }
        if name.len() == 2 {
            return RGB8::new(65, 65, 192);
        }

        // Pick some additional colors for the up/down variants.
        let digits: &[_] = &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
        let stripped = name.trim_end_matches(digits);
        if name.starts_with("^^") {
            if stripped.ends_with("♭") {
                return RGB8::new(192, 65, 192);
            } else {
                return RGB8::new(192, 169, 70);
            }
        }
        if name.starts_with("vv") {
            if stripped.ends_with("♯") {
                return RGB8::new(131, 117, 192);
            } else {
                return RGB8::new(192, 117, 67);
            }
        }
        if name.starts_with("^") {
            return RGB8::new(65, 192, 65);
        }
        if name.starts_with("v") {
            return RGB8::new(85, 200, 192);
        }

        // The unusual accidentals are a bit out of place in 31, so give them
        // their own colors.
        if name.starts_with("C♭") || name.starts_with("F♭") {
            // Blend the sharp and double sharp colors.
            return RGB8::new(131, 117, 192);
        }

        if name.starts_with("E♯") || name.starts_with("B♯") {
            // Blend the flat and double flat colors.
            return RGB8::new(192, 117, 67);
        }

        // If we are "up" sharps will be the normal color, likewise, flats will
        // be the normal color down, otherwise use an alternate color.
        if let Some(pos) = name.char_indices().skip(1).next() {
            let name = &name[pos.0..];
            if name.starts_with("♯") {
                return RGB8::new(192, 65, 65);
            }
            if name.starts_with("♭") {
                return RGB8::new(192, 65, 192);
            }
            if name.starts_with("𝄪") {
                return RGB8::new(192, 169, 70);
            }
            return RGB8::new(70, 192, 192);
        }

        RGB8::new(130, 192, 130)
    }

    fn middle_c(&self) -> MidiNote {
        self.middle_c
    }
}

#[test]
fn test_edo12() {
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 60 }, true), "C4");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 61 }, true), "C♯4");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 62 }, true), "D4");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 71 }, true), "B4");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 72 }, true), "C5");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 61 }, false), "D♭4");
    assert_eq!(EDO12.name(MidiNote { channel: 1, note: 48 }, true), "C3");
}
