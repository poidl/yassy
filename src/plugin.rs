extern crate libc;

// use synth;
use midi;
use midi::*;
use observer;
use observer::*;
use types;
use std::rc::Rc;
use oscillator::*;
use std::collections::VecDeque;
use voice;
use midiproc;
use midiproc::*;

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

pub struct Producers {
    osc: Box<OscBLIT>,
    osc2: Box<OscBLIT>,

}

// A plugin
pub struct Plugin<'a> {
    pub audio_out: f32,
    pub params: [*mut f32; NPARAMS],
    pub producers: Producers,
    pub voice: Box<voice::Voice<'a>>,
    pub voice2: Box<voice::Voice<'a>>,
    pub midi_message_processor: Box<MidiMessageProcessor<'a>>,
    pub midi_message: Observable<'a, midi::MidiMessage>,
    pub fs: Observable<'a, types::fs>,
    // pub mixer: Box<[f32; 5]>,
}



impl<'a> Plugin<'a> {
    pub fn new() -> Plugin<'a> {
        let mut plugin = Plugin {
            audio_out: 0f32,
            params: [&mut 0.5f32, &mut 1f32, &mut 1f32],
            producers: Producers{
                osc: Box::new(OscBLIT::new()),
                osc2: Box::new(OscBLIT::new()),
                },
            voice: Box::new(voice::Voice::new()), 
            voice2: Box::new(voice::Voice::new()), 
            midi_message_processor: Box::new(MidiMessageProcessor::new()),     
            midi_message: Observable::new([0u8,0u8,0u8]),
            fs: Observable::new(types::fs(0f64)),
            // mixer: Box::new([0f32; 5])
        };
        plugin.connect();
        if plugin.params.len() != NPARAMS {
            panic!("Wrong number of parameters")
        }
        plugin
    }
    pub fn connect(&mut self) {
        unsafe {
            let osc = &mut*self.producers.osc as *mut OscBLIT;
            let osc2 = &mut*self.producers.osc2 as *mut OscBLIT;
            let midiproc = &mut*self.midi_message_processor as *mut MidiMessageProcessor;
            let voice = &mut*self.voice  as *mut voice::Voice;
            let voice2 = &mut*self.voice2  as *mut voice::Voice;

            self.midi_message.observers.push(&mut *midiproc);
            self.midi_message_processor.noteon[0].observers.push(&mut *voice);
            self.midi_message_processor.noteoff[0].observers.push(&mut *voice);
            self.midi_message_processor.noteon[1].observers.push(&mut *voice2);
            self.midi_message_processor.noteoff[1].observers.push(&mut *voice2);
            self.voice.f0.observers.push(&mut *osc);
            self.voice2.f0.observers.push(&mut *osc2);
            self.fs.observers.push(&mut *osc);
            self.fs.observers.push(&mut *osc2);

        }
    }
    // pub fn update<T>(&mut self, iter: T, n_samples: u32) 
    // where T: Iterator<Item=(u32, midi::MidiMessage)> {
    //     unsafe {
    //         let mut i = 0;
    //         for (ievent, mm) in iter {
    //             println!("");
    //             println!("Processing MIDI...");
    //             while i < ievent {
    //                 let amp = self.get_amp();
    //                 *self.audio_out.offset(i as isize) = amp;
    //                 i =  i+1;
    //             }
    //             // self.notifyevent_midi(mm);
    //             self.midi_message.update(mm);
    //         }
    //         while i < n_samples {
    //             let amp = self.get_amp();
    //             *self.audio_out.offset(i as isize) = amp;
    //             i =  i+1;
    //         }
    //     }
    // }
    // pub fn notifyevent_midi(&mut self, mm: MidiMessage) {
    //     for o in &mut self.observers {
    //         o.next(mm)
    //     }
    // }
    // pub fn notifyevent_note(&mut self, f0: f32, vel: f32) {
    //     for o in self.midi_note_observers {
    //         o.next(f0, vel)
    //     }
    // }
    // pub fn set_fs(&mut self, fs: f64) {
    //     self.notifyevent_fs(fs);  
    // }
    // pub fn notifyevent_fs(&mut self, fs: f64) {
    //     // for o in &mut self.observers {
    //     //     o.next_fs(fs)
    //     // }
    //     let ft = types::fs(fs);
    //     self.fs.update(ft);
    // }
    // // pub fn notifyevent_blit(&mut self, blit: bool) {
    // //     // for o in &mut self.observers {
    // //         self.synth.next_blit(blit)
    // //     // }
    // // }
    // pub fn notifyevent_postfilter(&mut self, pf: bool) {
    //     // for o in &mut self.observers {
    //     //     o.next_postfilter(pf)
    //     // }
    // }
    // pub fn get_amp(&mut self) -> f32 {
    //     unsafe {
    //         let g = *(self.params[ParamName::Gain as usize]);
    //         let p1 = *(self.params[ParamName::BLIT as usize]);
    //         let p2 = *(self.params[ParamName::Postfilter as usize]);
    //         // self.notifyevent_blit(to_bool(p1));
    //         // self.notifyevent_postfilter(to_bool(p2));
    //         (10f32).powf(g / 20f32) * self.synth.get_amp()
    //         // self.synth.get_amp()
    //     }
    // }
    // pub fn cleanup(&mut self) {
    //     self.synth.cleanup();
    // }
    pub fn mix(&mut self) {
        let b1 = *self.producers.osc.buf;
        let b2 = *self.producers.osc2.buf;
        let vel1 = self.voice.vel;
        let vel2 = self.voice2.vel;

        // self.audio_out = vel1*(b1+b2);
        self.audio_out = vel1*b1+ vel2*b2;
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

impl<'a> Observer<u32> for Plugin<'a> {
    fn next(&mut self, pos: u32) {
        self.producers.osc.next(pos);
        self.producers.osc2.next(pos);
        self.mix();
    }
}


