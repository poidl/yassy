#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate libc;

use std::ffi::CString;
use synth;

use midi;
use midi::*;
use synth::*;
use oscillator::Oscillator;

// Number of parameters
const NPARAMS: usize = 3;

pub enum ParamName {
    Gain,
    BLIT,
    Postfilter,
}

pub trait HasFs {
    // sample rate
    fn set_fs(&mut self, f64);
}

pub struct SynthPlugin {
    pub midi_in: *const u8,
    pub audio_out: *mut f32,
    pub synth: synth::Synth,
    fs: f64,
    pub params: [*mut f32; NPARAMS],
}

impl SynthPlugin {
    pub fn new() -> SynthPlugin {
        let synth = SynthPlugin {
            midi_in: &0u8,
            audio_out: &mut 0f32,
            synth: synth::Synth::new(),
            fs: 0f64,
            params: [&mut 0.5f32, &mut 1f32, &mut 1f32],
        };
        if synth.params.len() != NPARAMS {
            panic!("Wrong number of parameters")
        }
        synth
    }
    pub fn noteon(&mut self, f0: f32, vel: f32) {
        self.synth.noteon(f0, vel);
    }
    pub fn noteoff(&mut self) {
        self.synth.noteoff();
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.synth.set_fs(fs);
    }
    pub fn get_amp(&mut self) -> f32 {
        unsafe {
            let g = *(self.params[ParamName::Gain as usize]);
            let p1 = *(self.params[ParamName::BLIT as usize]);
            let p2 = *(self.params[ParamName::Postfilter as usize]);
            self.synth.voice.osc1.use_blit = toBool(p1);
            self.synth.voice.osc1.use_postfilter = toBool(p2);
            // println!("USE BLIT: {}", self.synth.voice.osc1.use_blit);
            g * self.synth.get_amp()
            // self.synth.get_amp()
        }
    }
    pub fn cleanup(&mut self) {
        self.synth.cleanup();
    }
}

pub fn toI8(paramval: f32) -> i8 {
    // let half = 127f32/2f32;
    paramval.round() as i8
}

pub fn toBool(paramval: f32) -> bool {
    // let half = 127f32/2f32;
    if paramval < 0.5f32 {
        return false;
    }
    return true;
}
