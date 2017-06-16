
use voice;
use voice::*;
use midi;
use midi::*;
use observer;
use observer::*;
use std::collections::VecDeque;

// Number of parameters
pub const NPARAMS: usize = 2;

pub enum ParamName {
    BLIT,
    Postfilter,
}

pub struct Synth {
    pub params: [*mut f32; NPARAMS],
    pub note_queue: VecDeque<[u8;3]>,
    // pub voice: voice::Voice,
    pub observers: Vec<Box<Observer>>
}

impl Synth {
    pub fn new() -> Synth {
        let mut s = Synth { 
            params: [&mut 1f32, &mut 1f32],
            note_queue: VecDeque::with_capacity(10),
            // voice: voice::Voice::new() 
            observers: Vec::with_capacity(1)
        };
        s.observers.push(Box::new(voice::Voice::new()));
        s
    }
    // pub fn noteon(&mut self, f0: f32, vel: f32) {
    //     self.voice.next(f0, vel)
    // }
    // pub fn noteoff(&mut self) {
    //     // self.voice.on = false;
    //     self.voice.noteoff();
    // }
}

impl Observer for Synth {
    fn next(&mut self, mm: midi::MidiMessage) {

        if mm.noteon() {
            self.note_queue.push_front(*mm);
            self.notifyevent_noteon(mm.f0(), mm.vel())
        } else if mm.noteoff() {
            // check if this note (identified by number/frequency) is queued
            let result = self.note_queue.iter().position(|x| (x as midi::MidiMessage).note_number() == mm.note_number());
            match result {
                Some(i) => {
                    self.note_queue.remove(i);
                    if i == 0 {
                        self.notifyevent_noteoff();
                        if self.note_queue.len() > 0 {
                            let mm = &self.note_queue[0].clone();
                            self.notifyevent_noteon(mm.f0(), mm.vel())
                        }
                    }
                }
                _ => {}
            }
        }
        for mm in &self.note_queue {
            print!(" {}", mm.note_number());
            println!("")
        }
    }
    fn next_fs(&mut self, fs: f64) {
        self.notifyevent_fs(fs);
    }
    fn next_blit(&mut self, blit: bool) {
        self.notifyevent_blit(blit)
    }
    fn next_postfilter(&mut self, pf: bool) {
        self.notifyevent_postfilter(pf);
    }
    fn get_amp(&mut self) -> f32 {
        (*self.observers[0]).get_amp()
    }
    fn cleanup(&mut self) {
        (*self.observers[0]).cleanup();
    }
    fn next_noteon(&mut self, f0: f32, vel: f32) {
        self.notifyevent_noteon(f0,vel)
    }
    fn next_noteoff(&mut self) {
        self.notifyevent_noteoff()
    }
}

impl Synth {
    pub fn notifyevent_noteon(&mut self, f0: f32, vel: f32) {
        for o in &mut self.observers {
            o.next_noteon(f0, vel)
        }
    }
    pub fn notifyevent_noteoff(&mut self) {
        for o in &mut self.observers {
            o.next_noteoff()
        }
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
}