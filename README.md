# David Brown's Lumatone Mapping Generator

This project contains a generator for Lumatone mappings. It is a fairly
straightforward Rust project. `cargo run` will generate the various mapping
files for each layout within the `layouts/mappingname` directories. It is not
necessary to run the program, unless you make changes, as the mappings have been
committed to the repository as well.

Currently, there are three things that go into a mapping:

-   The `Tuning`. This describes enough to the program how a given tuning maps
    to MIDI notes, describes the intervals in that tuning, and gives names for
    the notes. Currently, there are three tunings available: EDO12, EDO19, and
    EDO31.
-   The `Layout`.  This describes the algorithm for building the mapping.  There
    are two layouts implemented so far:
    -   [Wicki-Hayden](https://en.wikipedia.org/wiki/Wicki%E2%80%93Hayden_note_layout).
        A layout that emphasizes the diatonic scale.
    -   [Harmonic Table](https://en.wikipedia.org/wiki/Harmonic_table_note_layout).
        A layout that emphasizes harmonic relations. Scales are more difficult
        to play, but chords tend to be clustered closely.
-   The `FillInfo`.  This describes how the mapping will be placed on the
    keyboard.  This describes what range is covered, as well as where the
    starting note (usually Middle C) is placed.  Currently, a split layout, and
    a full keyboard (wide) are supported.

The combinations of these mappings are under the `layouts` directory.  For each
mapping, there is an `.ltn` file to load into the Lumatone editor, and an `.svg`
file showing the mapping.

## Mapping notes

For the Wicki-Hayden layout, all of the generated mappings are mostly useful.
For EDO31, there isn't enough room in the split, so some of the double sharps
are not present on the keyboard.  They are all present on the right side, but
the grouping is split (it is on the edges) so not really useful for playing.
The wide version of EDO31, covers everything, but only a little over 4 octaves.

The harmonic table layout works well for EDO12 and EDO19.  The split version
isn't particularly useful, as it kind of limits what keys are available, and
this layout covers sufficient range on its own.

EDO31 does not work with the harmonic table layout, as the layout is not
generative within a single octave.  The files are present, but a given note will
only be available in every other octave.

## Generation

This program is intended to make isomorphic mappings; mappings that are regular
and defined by specific rules.

To make an isomorphic layout, there needs to be a generator.  This describes the
intervals that are along each axis of the mapping.  Of the three axes,
describing two of the intervals will define the third.  The code currently
doesn't check that the generator is consistent, and inconsistent intervals will
result in a layout that depends on the fill algorithm used.

The FillInfo describes a starting key and gives left and right bounds.  The
algorithm will fill fully vertically.

## The Layouts

### Wicki-Hayden

The
[Wicki-Hayden](https://en.wikipedia.org/wiki/Wicki%E2%80%93Hayden_note_layout)
layout is a layout I haven't seen used as much on the Lumatone.  It places an
emphasis on the diatonic scale.  I found this to be one of the easiest layouts
to learn to play (coming from the standard piano layout).

It looks like this with 12-EDO, split:

![12-EDO Wicky-Hayden Split](https://raw.githubusercontent.com/tangybbq/dlb-lumatone/main/layouts/dlb-edo12-wicki-hayden-split/dlb-edo12-wicki-hayden-split.svg)

along with the [layout files](layouts/dlb-edo12-wicki-hayden-split)

For 19-EDO split:

![19-EDO Wicky-Hayden Split](https://raw.githubusercontent.com/tangybbq/dlb-lumatone/main/layouts/dlb-edo19-wicki-hayden-split/dlb-edo19-wicki-hayden-split.svg)

along with the [layout files](layouts/dlb-edo19-wicki-hayden-split)

For 31-EDO, split:

![31-EDO Wicky-Hayden Split](https://raw.githubusercontent.com/tangybbq/dlb-lumatone/main/layouts/dlb-edo31-wicki-hayden-split/dlb-edo31-wicki-hayden-split.svg)

along with the [layout files](layouts/dlb-edo31-wicki-hayden-split)

Due to size constraints, there are a few double sharps that are missing on the
left half (the bass notes).  There is a wide 31-EDO layout that doesn't have the
missing keys, but covers a lot less range.

![31-EDO Wicky-Hayden Wide](https://raw.githubusercontent.com/tangybbq/dlb-lumatone/main/layouts/dlb-edo31-wicki-hayden-wide/dlb-edo31-wicki-hayden-wide.svg)

along with the [layout files](layouts/dlb-edo31-wicki-hayden-wide)
