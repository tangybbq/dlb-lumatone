//! Lumatone iso filling.
//!
//! Implements a graph fill for the Lumatone.

use std::collections::VecDeque;

use crate::tuning::{MidiNote, Tuning};

use super::{Dir, FillInfo, KeyIndex, KeyInfo, Keyboard, Layout, MoveMap};

pub struct Filler<'k, 't, 'l, 'f> {
    keyboard: &'k mut Keyboard,
    tuning: &'t dyn Tuning,
    layout: &'l Layout,
    info: &'f FillInfo,

    /// Cells that need to be filled in.
    work: VecDeque<Work>,

    mv: MoveMap,
}

/// A single unit of work for the filler.  Indicates a cell that should be
/// filled.
#[derive(Debug)]
struct Work {
    /// The horizontal position of this cell.  Used to limit the horizontal
    /// bounds of the work.
    x: isize,
    /// The position we want to fill.
    pos: KeyIndex,
    /// The note value for this cell.
    note: MidiNote,
    /// Which direction we should fill in.  This should be either UpLeft or
    /// UpRight, and the down will be calculated from the complement of this.
    phase: Phase,
    /// Are we in a part of the scan that is in an increasing direction (affects
    /// colors and names).
    increasing: bool,
}

/// The phase is the diagonal direction the fill.  This is described in terms of
/// the tilt angle used, with 'Left' indicating that we are filling up and to
/// the right, and down, to the left.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Phase {
    Left, Right,
}

/// The cardinal directions simplify the phases.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Cardinal {
    Left, Right, Up, Down,
}

impl<'k, 't, 'l, 'f> Filler<'k, 't, 'l, 'f> {
    /// Construct a new filler with the given information.
    pub fn new(
        keyboard: &'k mut Keyboard,
        tuning: &'t dyn Tuning,
        layout: &'l Layout,
        info: &'f FillInfo,
    ) -> Filler<'k, 't, 'l, 'f>
    {
        // Create the initial work.
        let first_cell = Work {
            x: 0,
            pos: info.start,
            note: tuning.middle_c(),
            phase: Phase::Left,
            increasing: true,
        };
        let mut work = VecDeque::new();
        work.push_back(first_cell);

        let mv = MoveMap::make();

        Filler { keyboard, tuning, layout, info, work, mv }
    }

    /// Run the filler, filling in the grid according to all of the work.
    pub fn run(&mut self) {
        let mut first = true;

        while let Some(work) = self.work.pop_front() {
            let cell =
                if let Some(cell) = self.keyboard.get_mut(work.pos) {
                    cell
                } else {
                    // Out of bounds.  This really shouldn't happen.
                    unreachable!()
                };
            if let Some(cell) = cell {
                // If the cell is already filled in, don't fill any more in from
                // this location.
                // However, lighten this square to help visualize where the boundary is.
                // But, only do this when we are filling with a different value than what is there.
                if cell.channel != work.note.channel || cell.note != work.note.note {
                    cell.color = cell.color.lighten();
                }
                continue;
            }

            // Ensure that the x position is within the requested bounds.
            if work.x > self.info.right as isize || work.x < -(self.info.left as isize) {
                // println!("  x out of bounds: {:?}", work);
                continue;
            }

            // We have all of the information for this cell.
            *cell = Some(KeyInfo {
                channel: work.note.channel,
                note: work.note.note,
                color: self.tuning.color(work.note, work.increasing),
                label: self.tuning.name(work.note, work.increasing),
                // label: format!("{}->{}", work.from, count),
            });

            // Generate additional work for everything adjacent.
            for card in Cardinal::iter() {
                let x = work.x + card.x_offset();

                // New position, if possible.
                let pos = if let Some(pos) = work.phase.pos_move(self, work.pos, card) {
                    pos
                } else {
                    continue;
                };

                // New note, if possible.
                let note = if let Some(note) = work.phase.note_move(self, work.note, card) {
                    note
                } else {
                    continue;
                };

                // Only calculate a new value for increasing in the first few, it is otherwise preserved.
                let increasing = if first { card.is_increasing() } else { work.increasing };
                let phase = card.new_phase(work.phase);

                self.work.push_back(Work { x, pos, note, phase, increasing });
            }

            first = false;
        }
    }
}

/// All of the cardinal directions, useful for iteration.
static ALL_CARDINALS: [Cardinal; 4] = [
    Cardinal::Left,
    Cardinal::Right,
    Cardinal::Up,
    Cardinal::Down,
];

impl Cardinal {
    fn iter() -> impl Iterator<Item = Cardinal> {
        ALL_CARDINALS.iter().cloned()
    }

    /// For a given cardinal direction, return the x adjustment.
    pub fn x_offset(self) -> isize {
        match self {
            Cardinal::Left => -1,
            Cardinal::Right => 1,
            _ => 0,
        }
    }

    /// Find the new phase for the given direction.
    pub fn new_phase(self, phase: Phase) -> Phase {
        match self {
            Cardinal::Left | Cardinal::Right => phase,
            _ => phase.complement(),
        }
    }

    /// Is this direction "increasing", meaning it should have higher note values.
    pub fn is_increasing(self) -> bool {
        match self {
            Cardinal::Up | Cardinal::Right => true,
            _ => false,
        }
    }
}

impl Phase {
    /// Given a cardinal direction, return the hex direction for this phase.
    fn dir(self, card: Cardinal) -> Dir {
        match card {
            Cardinal::Left => Dir::Left,
            Cardinal::Right => Dir::Right,
            Cardinal::Up => {
                match self {
                    Phase::Left => Dir::UpLeft,
                    Phase::Right => Dir::UpRight,
                }
            }
            Cardinal::Down => {
                match self {
                    // Phase::Left => Dir::DownRight,
                    // Phase::Right => Dir::DownLeft,
                    Phase::Left => Dir::DownLeft,
                    Phase::Right => Dir::DownRight,
                }
            }
        }
    }

    /// Move this cell, according to the given direction.
    pub fn pos_move(self, filler: &Filler, pos: KeyIndex, card: Cardinal) -> Option<KeyIndex> {
        let dir = self.dir(card);
        filler.mv.trymove(pos, dir)
    }

    /// Move this note, according to the given direction.
    pub fn note_move(self, filler: &Filler, note: MidiNote, card: Cardinal) -> Option<MidiNote> {
        let (interval, up) = match self.dir(card) {
            Dir::Left => (filler.layout.right, false),
            Dir::Right => (filler.layout.right, true),
            Dir::UpLeft => (filler.layout.up_left, true),
            Dir::UpRight => (filler.layout.up_right, true),
            Dir::DownLeft => (filler.layout.up_right, false),
            Dir::DownRight => (filler.layout.up_left, false),
        };
        filler.tuning.interval(note, interval, up)
    }

    /// Return the complement of this phase.
    pub fn complement(self) -> Phase {
        match self {
            Phase::Left => Phase::Right,
            Phase::Right => Phase::Left,
        }
    }
}
