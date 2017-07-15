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
use polyphony;
use polyphony::*;
use std::io;
use std::io::Write;

// Number of parameters
pub const NPARAMS: usize = 5;
pub const NOSC: usize = 4;

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

// pub struct Producers {
//     osc: Box<OscBLIT>,
//     osc2: Box<OscBLIT>,
//     osc3: Box<OscBLIT>,
//     osc4: Box<OscBLIT>,
// }

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
    // pub producers: Producers,

    // pub oscillators: Vec<Box<OscBLIT>>,
    // pub voices: Vec<Box<voice::Voice<'a>>>,
    pub note2osc: [u8; NOSC],
    // pub voice2: Box<voice::Voice<'a>>,
    // pub voice3: Box<voice::Voice<'a>>,
    // pub voice4: Box<voice::Voice<'a>>,

    pub midi_message_processor: Box<MidiMessageProcessor<'a>>,
    pub midi_message: Observable<'a, midi::MidiMessage>,
    pub poly: Box<Polyphony<'a>>,

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
            // oscillators: vec![
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new()), 
            //     Box::new(OscBLIT::new())
            // ],
            // voices: vec![
            //     Box::new(voice::Voice::new()), 
            //     Box::new(voice::Voice::new()), 
            //     Box::new(voice::Voice::new()), 
            //     Box::new(voice::Voice::new())
            // ],
            note2osc: [0; NOSC],
            midi_message_processor: Box::new(MidiMessageProcessor::new()),     
            midi_message: Observable::new([0u8,0u8,0u8]),
            poly: Box::new(Polyphony::new()),     
            fs: Observable::new(types::fs(0f64)),
            // mixer: Box::new([0f32; 5])
        };
        plugin.connect();
        if plugin.params_ptr.len() != NPARAMS {
            panic!("Wrong number of parameters")
        }
        for (i, o) in plugin.note2osc.iter_mut().enumerate() {
            *o = i as u8
        } 
        plugin
    }
    pub fn connect(&mut self) {
        unsafe {
            for (i, osc) in self.poly.oscillators.iter_mut().enumerate() {
                let o = &mut**osc as *mut OscBLIT;
                self.fs.observers.push(&mut *o);
                self.params.blit.observers.push(&mut *o);
                // self.voices[i].f0.observers.push(&mut *o);
            }


            let midiproc = &mut*self.midi_message_processor as *mut MidiMessageProcessor;
            let poly = &mut*self.poly as *mut Polyphony;

            self.params.polyphony.observers.push(&mut *poly);
            self.params.nvoices.observers.push(&mut *poly);
            self.params.unison.observers.push(&mut *poly);

            // self.params.polyphony.observers.push(&mut *midiproc);
            // self.params.nvoices.observers.push(&mut *midiproc);
            // self.params.unison.observers.push(&mut *midiproc);

            // self.poly.observers.push()

            self.midi_message.observers.push(&mut *midiproc);
            self.poly.maxnotes.observers.push(&mut *midiproc);

            self.midi_message_processor.noteon.observers.push(&mut *poly);
            self.midi_message_processor.noteoff.observers.push(&mut *poly);


            // for (i, voice) in self.voices.iter_mut().enumerate() {
            //     let v = &mut**voice  as *mut voice::Voice;
            //     self.midi_message_processor.noteon[i].observers.push(&mut *v);
            //     self.midi_message_processor.noteoff[i].observers.push(&mut *v);
            // }


        }
        // pub fn unison_mono() {
        //     for self.
        // }
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

        let bufs = [
            *self.poly.oscillators[0].buf,
            *self.poly.oscillators[1].buf,
            *self.poly.oscillators[2].buf,
            *self.poly.oscillators[3].buf,
        ];

        let vels = [
             self.poly.voices[0].vel,
             self.poly.voices[1].vel,
             self.poly.voices[2].vel,
             self.poly.voices[3].vel,
        ];

        let vv = self.poly.voicevec.0;

        // println!{"b1: {}", b1}
        // println!{"vel1: {}", vel1}

        // self.audio_out = vel1*(b1+b2);
        // println!("asdf {}",vv[0]);
        // println!("asdf {}",vels[vv[0] as usize]);
        // println!("asdf {}",vv[0])
        self.audio_out = vels[vv[0] as usize]*bufs[0] + 
                         vels[vv[1] as usize]*bufs[1] + 
                         vels[vv[2] as usize]*bufs[2] + 
                         vels[vv[3] as usize]*bufs[3];
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
        for o in self.poly.oscillators.iter_mut() {
            o.next(pos)
        }
        self.mix();
    }
}


// impl<'a> Observer<types::polyphony> for Plugin<'a> {
//     fn next(&mut self, p: types::polyphony) {
//         if p.0 {
//             if self.unison & (self.nvoices > 1) {
//                 self.nnotes = self.nvoices / 2;
//                 // println!("Polyphony on with union, nnotes = {}", self.nnotes);
//                 return
//             }
//             self.nnotes = self.nvoices;
//             // println!("Polyphony on, union off, nnotes = {}", self.nnotes);
//             return
//         }
//         self.nnotes = 1;
//         // println!("Polyphony off");
//     }
// }

// impl<'a> Observer<types::nvoices> for Plugin<'a> {
//     fn next(&mut self, p: types::nvoices) {
//         self.nvoices = p.0;
//         // println!("Nvoices = {}", self.nvoices);
//     }
// }

// impl<'a> Observer<types::note2osc> for Plugin<'a> {
//     fn next(&mut self, p: types::note2osc) {
//         self.note2osc = p.0;
//         println!("NOTE2OSC: ");
//         for i in self.note2osc.iter() {
//             print!{" {},", i}
//         }
//         io::stdout().flush().unwrap();
//     }
// }



// impl<'a> Observer<types::unison> for Plugin<'a> {
//     fn next(&mut self, u: types::unison) {
//         if u.0 {
//             if 
//             self.unison = true;
//             return
//         }
//         self.unison = false
//         // println!("Unison = {}", self.unison);
//     }
// }


