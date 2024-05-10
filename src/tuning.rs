//! Management of tuning systems.
//!
//! Manage tuning systems, and the various ways that they deal with names of
//! notes, and midi note/channel numbers.

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MidiNote {
    channel: u8,
    note: u8,
}

/// A few intervals that are used for building keyboards.
pub enum Interval {
    MinorSecond,
    MajorSecond,
    PerfectFourth,
    PerfectFifth,
}

/// A tuning system, at least as much information as is needed to produce a
/// keyboard layout and midi mapping.  Right now, the midi mapping is definitive.
pub trait Tuning {
    /// Adjust a note by an interval. `up` indicates a higher pitch when true.
    /// None indicates either the note is out of range, or the interval doesn't
    /// make sense with this tuning.
    fn interval(&self, note: MidiNote, interval: Interval, up: bool) -> Option<MidiNote>;

    /// Return a nice name for this note. The 'sharp' hint is for tuning systems
    /// that have enharmonic sharps and flats, as a suggestion of which name to
    /// use.
    fn name(&self, note: MidiNote, sharp: bool) -> String;

    /// For tunings where intervals are independ (the EDOs), interval can be
    /// calculated just based on a number of steps. This should be None if this
    /// doesn't make sense, and the implementer should define their own
    /// `interval` method.
    fn get_steps(&self, interval: Interval) -> isize;
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

static EDO12_INTERVALS: [isize; 4] = [
    1, 2, 5, 7,
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

impl Tuning for Edo {
    fn get_steps(&self, interval: Interval) -> isize {
        self.intervals[interval as usize]
    }

    fn interval(&self, note: MidiNote, interval: Interval, up: bool) -> Option<MidiNote> {
        if self.channel_octaves.is_some() {
            // This needs to be different.
            todo!()
        }
        let steps = self.get_steps(interval);
        let steps = u8::try_from(steps).ok()?;
        let pitch = if up { note.note.checked_add(steps)? } else { note.note.checked_sub(steps)? };
        if pitch > 127 {
            return None;
        }
        Some(MidiNote { channel: note.channel, note: pitch, })
    }

    fn name(&self, note: MidiNote, sharp: bool) -> String {
        if self.channel_octaves.is_some() {
            todo!()
        }

        // We assume that Middle C is C-4.
        let pitch = note.note as isize - self.middle_c.note as isize;
        let pitch = pitch + self.octave as isize * 4;
        let octave = pitch / (self.octave as isize);
        let pitch = pitch % (self.octave as isize);
        let names = if sharp { self.sharp_names } else { self.flat_names };
        format!("{}{}", names[pitch as usize], octave)
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
