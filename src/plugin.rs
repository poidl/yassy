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


pub struct MidiMessageProcessor<'a> {
    pub note_stack: Vec<[u8;3]>,
    pub playing: VecDeque<[u8;3]>,
    // pub note_ev: Vec<Observable<'a, types::noteon>>,
    pub released: Vec<u8>,
    // pub playing: VecDeque<u8>,
    // pub playing: Vec<u8>,
    // pub num: u8;
    pub synths: [Option<[u8;3]>; 3],
    pub synths_old: [Option<[u8;3]>; 3],
    pub noteon: Vec<Observable<'a, types::noteon>>,
    pub noteoff: Vec<Observable<'a, types::noteoff>>,
    // pub noteon: Observable<'a, types::noteon>,
    // pub noteoff: Observable<'a, types::noteoff>,
}

const NN: usize = 3;

// Observes MidiMessages, and emits noteon and noteoff observables
impl<'a> MidiMessageProcessor<'a> {
    pub fn new() -> MidiMessageProcessor<'a> {
    let p = MidiMessageProcessor { 
        note_stack: Vec::with_capacity(10),
        playing: VecDeque::with_capacity(NN),
        // note_ev: Vec::with_capacity(10),
        released: Vec::with_capacity(NN),
        // playing: VecDeque::with_capacity(3),
        // playing: Vec::with_capacity(NN),
        // num: 0u8,
        synths: [None, None, None],
        synths_old: [None, None, None],
        noteon: vec![
            Observable::new(types::noteon(0f32,0f32)),
            Observable::new(types::noteon(0f32,0f32)),
            Observable::new(types::noteon(0f32,0f32)),
            ],
        noteoff: vec![
            Observable::new(types::noteoff(0u8)),
            Observable::new(types::noteoff(0u8)),
            Observable::new(types::noteoff(0u8)),
            ],
        // noteon: [Observable::new(types::noteon(0f32,0f32)); NN],
        // noteoff: [Observable::new(types::noteoff(0u8)); NN],
        // noteon: Observable::new(types::noteon(0f32,0f32)),
        // noteoff: Observable::new(types::noteoff(0u8)),
        };
    p
    }
    fn update_synths(&mut self) {
        // set the note thats not played any more to None
        let mut index: Option<usize> = None;
        for (i, n) in self.synths.iter().enumerate() {
            match *n {
                Some(note) => {
                    let result = self.playing.iter().position(|x| x.note_number() == note.note_number());
                    match result {
                        Some(j) => {}
                        _ => {
                            index = Some(i);
                            break
                        }
                    }
                }
                _ => {}
            }
        
        }
        match index {
            Some(i) => {
                println!("index:****{}", i);
                self.synths[i] = None
            }
            _ => {}
        }

        for mm in self.playing.iter() {
            // is the note already assigned to a synth
            let result = self.synths.iter().position(|x|
            match *x {
                Some(note) => {
                    mm.note_number() == note.note_number()
                }
                _ => false
            }
            );
            match result {
                // if yes, do nothing
                Some(i) => {}
                // if no
                _ => {
                    // find the first free slot
                    let result = self.synths.iter().position(|x|
                        match *x {
                            Some(note_number) => {
                                false
                            }
                            _ => true
                        }
                    );
                    match result {
                        // if there is a free slot, assign the note number
                        Some(i) => {
                            self.synths[i] = Some(*mm);
                        }
                        _ => {
                            // TODO: find out why this sometimes panics (XRUNS, skipped midi note-off events?)
                            // panic!{"No free slot."}
                            }
                    }
                }
            }
        }
        println!("SYNTHS: ");
        for i in self.synths.iter() {
            match *i {
                Some(note) => {
                    println!("i: {} ", note.note_number())
                }
                _ =>  println!("i: None ")
            }
            
        }
        for (i, n) in self.synths.iter().enumerate() {
            match *n {
                Some(note) => {
                    if note.note_number() != self.synths_old[i].note_number() {
                        self.noteon[i].update(note)
                    }
                }
                _ => {
                    match self.synths_old[i] {
                        Some(note) => {
                            self.noteoff[i].update()
                        }
                    }
                }
            }
            
        }
        self.synths_old = self.synths;
        
    }
}

impl<'a> Observer<MidiMessage> for MidiMessageProcessor<'a> {
    fn next(&mut self, mm: midi::MidiMessage) {
        let mut release: Option<midi::MidiMessage> = None;
        if mm.noteon() {
            self.note_stack.push(mm);
            // if self.note_ev.len() <= 2 {
            //     self.note_ev.push(types::noteon(mm.f0(), mm.vel()));
            // }
            // if let Some(n) = self.released.pop() {
            //     self.playing.push(n);
            // }
            // self.noteon.update(types::noteon(mm.f0(), mm.vel()));
        } else if mm.noteoff() {
            // check if this note (identified by number/frequency) is queued
            let result = self.note_stack.iter().position(|x| x.note_number() == mm.note_number());
            match result {
                Some(i) => {
                    release = Some(self.note_stack.remove(i));

                    // let res = self.playing.iter().position(|x| x.note_number() == mm.note_number());
                    // match res {
                    //     self.playing.remove(i);
                    // }
                        // self.noteoff.update(types::noteoff(mm.note_number()));
                        // if let Some(mm) = self.note_stack.last() {
                        //     self.noteon.update(types::noteon(mm.f0(), mm.vel()));
                        // }                        
                }
                _ => {}
            }
        }


        match release {
            Some(mm) => {
                let res = self.playing.iter().position(|x| x.note_number() == mm.note_number());
                match res {
                    Some(i) => {
                        self.playing.remove(i);
                    }
                    _ => {}
                }

                }
            _ => {}

        }

        let len = self.note_stack.len() as i8;
        let i_most_recent = len - NN as i8 + 1i8;
        let mut i = len-1;
        while (i >= len - NN as i8) & (i>=0) {
            let mm = self.note_stack[i as usize];
            
            let res = self.playing.iter().position(|x| x.note_number() == mm.note_number());
            match res {
                Some(i) => {}
                _ => {
                    self.playing.push_back(mm);
                    if self.playing.len() == NN+1 {
                        self.playing.pop_front();
                    }
                }
            }
            i = i-1;
        }
        
        for mm in &self.playing {
            print!("PLAYING: {}", mm.note_number());
            println!("")
        }
        self.update_synths()
    }
}