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
pub const NPARAMS: usize = 5;

pub enum ParamName {
    // Gain,
    BLIT,
    Polyphony,
    Unison,
    Voices,
    Detune,
    // Postfilter,
}

pub trait HasFs {
    // sample rate
    fn set_fs(&mut self, f64);
}

pub struct Producers {
    osc: Box<OscBLIT>,
    osc2: Box<OscBLIT>,
    osc3: Box<OscBLIT>,
    osc4: Box<OscBLIT>,
}

pub struct Params<'a> {
    // gain: Observable<'a, types::gain>,
    blit: Observable<'a, types::blit>,
    polyphony: Observable<'a, types::polyphony>,
    unison: Observable<'a, types::unison>,
    nvoices: Observable<'a, types::nvoices>,
    detune: Observable<'a, types::detune>,
}

// A plugin
pub struct Plugin<'a> {
    pub audio_out: f32,
    pub params_ptr: [*mut f32; NPARAMS],
    pub params: Params<'a>,
    pub producers: Producers,

    pub voice: Box<voice::Voice<'a>>,
    pub voice2: Box<voice::Voice<'a>>,
    pub voice3: Box<voice::Voice<'a>>,
    pub voice4: Box<voice::Voice<'a>>,

    pub midi_message_processor: Box<MidiMessageProcessor<'a>>,
    pub midi_message: Observable<'a, midi::MidiMessage>,

    pub fs: Observable<'a, types::fs>,
    // pub mixer: Box<[f32; 5]>,
}


impl<'a> Plugin<'a> {
    pub fn new() -> Plugin<'a> {
        let mut plugin = Plugin {
            audio_out: 0f32,
            // params_ptr: [&mut 0.5f32, &mut 1f32, &mut 1f32],
            params_ptr: [&mut 1f32, &mut 0f32, &mut 0f32, &mut 1f32, &mut 0f32],
            params: Params {
                // gain: Observable::new(types::gain(0.5f32))
                blit: Observable::new(types::blit(true)),
                polyphony: Observable::new(types::polyphony(false)),
                unison: Observable::new(types::unison(false)),
                nvoices: Observable::new(types::nvoices(1)),
                detune: Observable::new(types::detune(0f32)),
            },
            producers: Producers{
                osc: Box::new(OscBLIT::new()),
                osc2: Box::new(OscBLIT::new()),
                osc3: Box::new(OscBLIT::new()),
                osc4: Box::new(OscBLIT::new()),
                },
            voice: Box::new(voice::Voice::new()), 
            voice2: Box::new(voice::Voice::new()), 
            voice3: Box::new(voice::Voice::new()), 
            voice4: Box::new(voice::Voice::new()), 
            midi_message_processor: Box::new(MidiMessageProcessor::new()),     
            midi_message: Observable::new([0u8,0u8,0u8]),
            fs: Observable::new(types::fs(0f64)),
            // mixer: Box::new([0f32; 5])
        };
        plugin.connect();
        if plugin.params_ptr.len() != NPARAMS {
            panic!("Wrong number of parameters")
        }
        plugin
    }
    pub fn connect(&mut self) {
        unsafe {

            let osc = &mut*self.producers.osc as *mut OscBLIT;
            let osc2 = &mut*self.producers.osc2 as *mut OscBLIT;
            let osc3 = &mut*self.producers.osc3 as *mut OscBLIT;
            let osc4 = &mut*self.producers.osc4 as *mut OscBLIT;

            self.fs.observers.push(&mut *osc);
            self.fs.observers.push(&mut *osc2);
            self.fs.observers.push(&mut *osc3);
            self.fs.observers.push(&mut *osc4);

            self.params.blit.observers.push(&mut *osc);
            self.params.blit.observers.push(&mut *osc2);
            self.params.blit.observers.push(&mut *osc3);
            self.params.blit.observers.push(&mut *osc4);

            let midiproc = &mut*self.midi_message_processor as *mut MidiMessageProcessor;

            self.params.polyphony.observers.push(&mut *midiproc);
            self.params.nvoices.observers.push(&mut *midiproc);
            self.params.unison.observers.push(&mut *midiproc);

            let voice = &mut*self.voice  as *mut voice::Voice;
            let voice2 = &mut*self.voice2  as *mut voice::Voice;
            let voice3 = &mut*self.voice3  as *mut voice::Voice;
            let voice4 = &mut*self.voice4  as *mut voice::Voice;

            self.midi_message.observers.push(&mut *midiproc);

            self.midi_message_processor.noteon[0].observers.push(&mut *voice);
            self.midi_message_processor.noteoff[0].observers.push(&mut *voice);
            self.midi_message_processor.noteon[1].observers.push(&mut *voice2);
            self.midi_message_processor.noteoff[1].observers.push(&mut *voice2);
            self.midi_message_processor.noteon[2].observers.push(&mut *voice3);
            self.midi_message_processor.noteoff[2].observers.push(&mut *voice3);
            self.midi_message_processor.noteon[3].observers.push(&mut *voice4);
            self.midi_message_processor.noteoff[3].observers.push(&mut *voice4);

            self.voice.f0.observers.push(&mut *osc);
            self.voice2.f0.observers.push(&mut *osc2);
            self.voice3.f0.observers.push(&mut *osc3);
            self.voice4.f0.observers.push(&mut *osc4);

        }
    }

    // pub fn get_amp(&mut self) -> f32 {
    //     unsafe {
    //         let g = *(self.params_ptr[ParamName::Gain as usize]);
    //         let p1 = *(self.params_ptr[ParamName::BLIT as usize]);
    //         let p2 = *(self.params_ptr[ParamName::Postfilter as usize]);
    //         // self.notifyevent_blit(to_bool(p1));
    //         // self.notifyevent_postfilter(to_bool(p2));
    //         (10f32).powf(g / 20f32) * self.synth.get_amp()
    //         // self.synth.get_amp()
    //     }
    // }
    // pub fn cleanup(&mut self) {
    //     self.synth.cleanup();
    // }

    pub fn update_params(&mut self) {
        unsafe {
            // let g = *(self.params_ptr[ParamName::Gain as usize]);
            let p1 = *(self.params_ptr[ParamName::BLIT as usize]);
            let p2 = *(self.params_ptr[ParamName::Polyphony as usize]);
            let p3 = *(self.params_ptr[ParamName::Unison as usize]);
            let p4 = *(self.params_ptr[ParamName::Voices as usize]);
            let p5 = *(self.params_ptr[ParamName::Detune as usize]);
            // let p2 = *(self.params_ptr[ParamName::Postfilter as usize]);
            self.params.blit.update(types::blit(to_bool(p1)));
            self.params.polyphony.update(types::polyphony(to_bool(p2)));
            self.params.unison.update(types::unison(to_bool(p3)));
            self.params.nvoices.update(types::nvoices(to_usize(p4)));
            // self.notifyevent_blit(to_bool(p1));
            // self.notifyevent_postfilter(to_bool(p2));
            // (10f32).powf(g / 20f32) * self.synth.get_amp()
            // self.synth.get_amp()
        }
    }
    pub fn mix(&mut self) {
        let b1 = *self.producers.osc.buf;
        let b2 = *self.producers.osc2.buf;
        let b3 = *self.producers.osc3.buf;
        let b4 = *self.producers.osc4.buf;
        let vel1 = self.voice.vel;
        let vel2 = self.voice2.vel;
        let vel3 = self.voice3.vel;
        let vel4 = self.voice4.vel;

        // self.audio_out = vel1*(b1+b2);
        self.audio_out = vel1*b1+ vel2*b2 + vel3*b3 + vel4*b4;
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

pub fn to_usize(paramval: f32) -> usize {

    return paramval.trunc() as usize;
}

impl<'a> Observer<u32> for Plugin<'a> {
    fn next(&mut self, pos: u32) {
        self.update_params();
        self.producers.osc.next(pos);
        self.producers.osc2.next(pos);
        self.producers.osc3.next(pos);
        self.producers.osc4.next(pos);
        self.mix();
    }
}


