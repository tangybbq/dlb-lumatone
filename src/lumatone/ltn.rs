//! Lumatone LTN file reading.

use std::{fs::File, io::{BufRead, BufReader, Write}, path::Path};

use anyhow::Result;
use regex::Regex;

use super::{KeyIndex, KeyInfo, Keyboard, RGB8};

pub fn load<P: AsRef<Path>>(p: P) -> Result<Keyboard> {
    let board_re = Regex::new(r"^\[Board(\d+)\]$")?;
    let key_re = Regex::new(r"^Key_(\d+)=(\d+)$")?;
    let chan_re = Regex::new(r"^Chan_(\d+)=(\d+)$")?;
    let col_re = Regex::new(r"^Col_(\d+)=([0-9a-fA-F]{6})$")?;
    let invert_re = Regex::new(r"^CCInvert_(\d+)$")?;

    // For now, just ignore these, and we will use hard-coded defaults.
    let ignore_re = Regex::new(r"^(AfterTouchActive|LightOnKeyStrokes|InvertFootController|InvertSustain|ExprCtrlSensivity|VelocityIntrvlTbl|NoteOnOffVelocityCrvTbl|FaderConfig|afterTouchConfig|LumaTouchConfig)=(.*)$")?;

    let mut state = State::default();

    let mut board = Keyboard::default();

    for line in BufReader::new(File::open(p)?).lines() {
        let line = line?;
        if let Some(cap) = board_re.captures(&line) {
            state.set_group(&mut board)?;

            let group = cap.get(1).unwrap().as_str().parse::<usize>()?;
            state.group = Some(group);

            state.keys = vec![0; 56];
            state.chans = vec![0; 56];
            state.cols = vec![RGB8::white(); 56];
            state.inverts = vec![false; 56];
            continue;
        }
        if let Some(cap) = key_re.captures(&line) {
            let index = cap.get(1).unwrap().as_str().parse::<usize>()?;
            let value = cap.get(2).unwrap().as_str().parse::<u8>()?;
            state.keys[index] = value;
            continue;
        }
        if let Some(cap) = chan_re.captures(&line) {
            let index = cap.get(1).unwrap().as_str().parse::<usize>()?;
            let value = cap.get(2).unwrap().as_str().parse::<u8>()?;
            state.chans[index] = value;
            continue;
        }
        if let Some(cap) = col_re.captures(&line) {
            let index = cap.get(1).unwrap().as_str().parse::<usize>()?;
            let value = RGB8::parse(cap.get(2).unwrap().as_str())?;
            state.cols[index] = value;
            continue
        }
        if let Some(cap) = invert_re.captures(&line) {
            let index = cap.get(1).unwrap().as_str().parse::<usize>()?;
            state.inverts[index] = true;
            continue;
        }
        if let Some(_cap) = ignore_re.captures(&line) {
            continue;
        }
        println!("line: {:?}", line);
        println!("state: {:?}", state);
        break;
    }
    state.set_group(&mut board)?;
    Ok(board)
}

#[derive(Debug, Default)]
struct State {
    group: Option<usize>,
    keys: Vec<u8>,
    chans: Vec<u8>,
    cols: Vec<RGB8>,
    inverts: Vec<bool>,

    aftertouch: bool,
    light_on_strokes: bool,
    invert_foot: bool,
    invert_sustain: bool,
    expr_sensitivity: usize,
    velocity_intrvl: Vec<u16>,
    velocity: Vec<u8>,
    facer: Vec<u8>,
    after_touch: Vec<u8>,
    luma_touch: Vec<u8>,
}

impl State {
    fn set_group(&mut self, keyb: &mut Keyboard) -> Result<()> {
        let group = if let Some(group) = self.group {
            group
        } else {
            return Ok(())
        };

        for key in 0..56 {
            let channel = self.chans[key];
            let note = self.keys[key];
            keyb.set(KeyIndex { group: group as u8, key: key as u8 },
                     Some(KeyInfo {
                         channel,
                         note,
                         color: self.cols[key],
                         label: format!("{}:{}", channel, note),
                     }));
        }
        self.group = None;
        Ok(())
    }
}

/// Write out a lumatone file.  This only has the parameters that are meaningful
/// here.
pub fn save<P: AsRef<Path>>(p: P, keyb: &Keyboard) -> Result<()> {
    let mut fd = File::create(p)?;

    let default_info = KeyInfo::default();

    for group in 0..5 {
        writeln!(&mut fd, "[Board{}]", group)?;
        for key in 0..56 {
            let info = keyb.get(KeyIndex { group: group as u8, key: key as u8 }).unwrap_or(&default_info);
            writeln!(&mut fd, "Key_{}={}", key, info.note)?;
            writeln!(&mut fd, "Chan_{}={}", key, info.channel)?;
            writeln!(&mut fd, "Col_{}={}", key, info.color.to_hex())?;
        }
    }
    Ok(())
}
