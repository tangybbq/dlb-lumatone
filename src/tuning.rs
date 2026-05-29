//! Management of tuning systems.
//!
//! Manage tuning systems, and the various ways that they deal with names of
//! notes, and midi note/channel numbers.

use std::collections::BTreeMap;
use std::str::FromStr;

use serde::Deserialize;

use crate::lumatone::RGB8;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
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

impl FromStr for IntervalStep {
    type Err = String;

    /// Parse the short interval names used in the config files.
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "Aug1" => IntervalStep::AugUnison,
            "m2" => IntervalStep::MinorSecond,
            "M2" => IntervalStep::MajorSecond,
            "m3" => IntervalStep::MinorThird,
            "M3" => IntervalStep::MajorThird,
            "P4" => IntervalStep::PerfectFourth,
            "Aug4" => IntervalStep::AugmentedFourth,
            "dim5" => IntervalStep::DimishedFifth,
            "P5" => IntervalStep::PerfectFifth,
            "N2" => IntervalStep::NeutralSecond,
            "N3" => IntervalStep::NeutralThird,
            other => return Err(format!("unknown interval name: {:?}", other)),
        })
    }
}

/// Which direction does an interval move in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub enum IntervalDirection {
    #[default]
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

    /// The interval step (without direction).
    pub fn step(self) -> IntervalStep {
        self.step
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

    /// Resolve a named interval to a signed step count, or None if this tuning
    /// does not define the interval.
    fn try_interval_steps(&self, interval: Interval) -> Option<isize> {
        let steps = self.try_get_steps(interval.step)?;
        Some(if interval.is_up() { steps } else { -steps })
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

    /// Step count for a named interval, or None if this tuning does not define
    /// it.  This is the primitive; `get_steps` is the panicking convenience.
    fn try_get_steps(&self, interval: IntervalStep) -> Option<isize>;

    /// Step count for a named interval.  Panics if the interval is undefined;
    /// callers on the generation path rely on config validation having checked
    /// presence already.
    fn get_steps(&self, interval: IntervalStep) -> isize {
        self.try_get_steps(interval)
            .expect("interval not defined for this tuning")
    }

    /// Guess a good color for this particular note.
    fn color(&self, note: MidiNote, sharp: bool) -> RGB8;

    /// Return middle C for this tuning.
    fn middle_c(&self) -> MidiNote;
}

/// A named color scheme: a function mapping a note to a display color.  Tunings
/// select one by name in the config, and the heuristic lives in Rust so it can
/// be swapped or improved later (the current scheme is weak on large EDOs).
#[derive(Copy, Clone)]
pub struct ColorScheme(fn(&Edo, MidiNote, bool) -> RGB8);

impl ColorScheme {
    /// Resolve a color-scheme name to its implementation.
    pub fn by_name(name: &str) -> Option<ColorScheme> {
        match name {
            "ups_downs" => Some(ColorScheme(ups_downs_color)),
            _ => None,
        }
    }
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

    /// The number of steps for each named interval this tuning defines.
    intervals: BTreeMap<IntervalStep, isize>,

    /// Names of the pitches, with sharp bias, then with flat bias.
    sharp_names: Vec<String>,
    flat_names: Vec<String>,

    /// How to color the keys.
    color: ColorScheme,
}

impl Edo {
    /// Construct an Edo from already-validated config data.
    pub fn new(
        octave: usize,
        channel_octaves: Option<usize>,
        middle_c: MidiNote,
        intervals: BTreeMap<IntervalStep, isize>,
        sharp_names: Vec<String>,
        flat_names: Vec<String>,
        color: ColorScheme,
    ) -> Edo {
        Edo { octave, channel_octaves, middle_c, intervals, sharp_names, flat_names, color }
    }
}

impl Tuning for Edo {
    fn try_get_steps(&self, interval: IntervalStep) -> Option<isize> {
        self.intervals.get(&interval).copied()
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
            let names = if sharp { &self.sharp_names } else { &self.flat_names };
            format!("{}{}", names[pitch as usize], octave)
            // format!("{}-{}", octave, pitch)
        } else {
            // We assume that Middle C is C-4.
            let pitch = note.note as isize - self.middle_c.note as isize;
            let pitch = pitch + self.octave as isize * 4;
            let octave = pitch / (self.octave as isize);
            let pitch = pitch % (self.octave as isize);
            let names = if sharp { &self.sharp_names } else { &self.flat_names };
            format!("{}{}", names[pitch as usize], octave)
        }
    }

    fn color(&self, note: MidiNote, sharp: bool) -> RGB8 {
        (self.color.0)(self, note, sharp)
    }

    fn middle_c(&self) -> MidiNote {
        self.middle_c
    }
}

/// The default color scheme: base the color on the length of the note name,
/// with a special case for C4 and the up/down accidental variants.
fn ups_downs_color(edo: &Edo, note: MidiNote, sharp: bool) -> RGB8 {
    let name = edo.name(note, sharp);
    {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;

    /// Build the edo12 tuning from the real config and check note naming.
    #[test]
    fn test_edo12() {
        let cfg = Config::load("config".as_ref()).expect("load config");
        let edo12 = cfg.tuning("edo12").expect("edo12 tuning");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 60 }, true), "C4");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 61 }, true), "C♯4");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 62 }, true), "D4");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 71 }, true), "B4");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 72 }, true), "C5");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 61 }, false), "D♭4");
        assert_eq!(edo12.name(MidiNote { channel: 1, note: 48 }, true), "C3");
    }
}
