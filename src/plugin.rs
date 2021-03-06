extern crate libc;

use synth;
use midi;
use midi::*;
use observer;
use observer::*;

// Number of parameters
pub const NPARAMS: usize = 3;

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
    // pub synth: synth::Synth,
    pub audio_out: *mut f32,
    pub params: [*mut f32; NPARAMS],
    pub observers: Observers,
    pub midiMessage: Observable<midi::MidiMessage>,
    pub fs: Observable<f64>,
    pub blit: Observable<bool>,
    pub postfilter: Observable<bool>,
}

impl SynthPlugin {
    pub fn new() -> SynthPlugin {
        let mut plugin = SynthPlugin {
            // synth: synth::Synth::new(),
            audio_out: &mut 0f32,
            params: [&mut 0.5f32, &mut 1f32, &mut 1f32],
            observers: Vec::with_capacity(1),
            fs: 0f64,
            blit: false,
            
            postfilter: false,
        };
        if plugin.params.len() != NPARAMS {
            panic!("Wrong number of parameters")
        }
        plugin.observers.push(Box::new(synth::Synth::new()));
        plugin
    }
    pub fn update<'a, T>(&mut self, iter: T, n_samples: u32) 
    where T: Iterator<Item=(u32, midi::MidiMessage<'a>)> {
        unsafe {
            let mut i = 0;
            for (ievent, mm) in iter {
                println!("");
                println!("Processing MIDI...");
                while i < ievent {
                    let amp = self.get_amp();
                    *self.audio_out.offset(i as isize) = amp;
                    i =  i+1;
                }
                self.notifyevent_midi(mm);
            }
            while i < n_samples {
                let amp = self.get_amp();
                *self.audio_out.offset(i as isize) = amp;
                i =  i+1;
            }
        }
    }
    pub fn notifyevent_midi(&mut self, mm: MidiMessage) {
        for o in &mut self.observers {
            o.next(mm)
        }
    }
    // pub fn notifyevent_note(&mut self, f0: f32, vel: f32) {
    //     for o in self.midi_note_observers {
    //         o.next(f0, vel)
    //     }
    // }
    pub fn set_fs(&mut self, fs: f64) {
        self.notifyevent_fs(fs);  
    }
    pub fn notifyevent_fs(&mut self, fs: f64) {
        for o in &mut self.observers {
            o.next_fs(fs)
        }
    }
    pub fn notifyevent_blit(&mut self, blit: bool) {
        for o in &mut self.observers {
            o.next_blit(blit)
        }
    }
    pub fn notifyevent_postfilter(&mut self, pf: bool) {
        for o in &mut self.observers {
            o.next_postfilter(pf)
        }
    }
    pub fn get_amp(&mut self) -> f32 {
        unsafe {
            let g = *(self.params[ParamName::Gain as usize]);
            let p1 = *(self.params[ParamName::BLIT as usize]);
            let p2 = *(self.params[ParamName::Postfilter as usize]);
            self.notifyevent_blit(to_bool(p1));
            self.notifyevent_postfilter(to_bool(p2));
            (10f32).powf(g / 20f32) * (*self.observers[0]).get_amp()
            // self.synth.get_amp()
        }
    }
    pub fn cleanup(&mut self) {
        (*self.observers[0]).cleanup();
    }
}

pub fn to_i8(paramval: f32) -> i8 {
    // let half = 127f32/2f32;
    paramval.round() as i8
}

pub fn to_bool(paramval: f32) -> bool {
    // let half = 127f32/2f32;
    if paramval < 0.5f32 {
        return false;
    }
    return true;
}
