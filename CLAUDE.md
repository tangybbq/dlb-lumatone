# Overview

This project contains utilities for working with the Lumatone keyboard. The lumatone is a 280-key
music keyboard, where the keys are arranged in a hex-grid.

This project seeks to implement various layouts for the keyboard, using an algorithmic approach.

Beyond the simple 12-tone layouts, the first follow a few guidelines.

- Each octave of the mapping sends notes midi 60 and up for a single octave. The octave number
  itself is encoded in the channel number.
- External software in a DAW or pianoteq will implement support for this mode.
- All modes are defined by a pair of values: the interval (in EDO steps) of a horizontal movement,
  and the interval of a vertical step.  Currently both vertical axes are defined, even though
  setting two axes steps defines the third.
- There are some basic color codings to help make them playable.
- The program generates an .ltn file to load the mapping into the lumatone, and a .svg file showing
  the layout using a conventional notation.
