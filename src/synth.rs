
use voice;
use voice::*;
use midi;
use midi::*;
use observer;
use observer::*;
use types;
use std::collections::VecDeque;
use std::rc::Rc;

// Number of parameters
pub const NPARAMS: usize = 2;

pub enum ParamName {
    BLIT,
    Postfilter,
}

pub struct Synth {
    pub params: [*mut f32; NPARAMS],
    pub note_queue: VecDeque<[u8;3]>,
    pub voice: Rc<voice::Voice>,
    // pub observers: Vec<Box<Observer>>
    pub note: Observable<(f32, f32)>,
    pub fs: Observable<types::fs>,
    pub noteoff: Observable<types::noteoff>,
}

impl Synth {
    pub fn new() -> Synth {
        let mut s = Synth { 
            params: [&mut 1f32, &mut 1f32],
            note_queue: VecDeque::with_capacity(10),
            voice: Rc::new(voice::Voice::new()),
            note: Observable::new((0f32,0f32)),
            fs: Observable::new(types::fs(0f64)),
            noteoff: Observable::new(types::noteoff)
            // observers: Vec::with_capacity(1)
        };
        s.note.observers.push(s.voice.clone());
        s.fs.observers.push(s.voice.clone());
        s.noteoff.observers.push(s.voice.clone());
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

impl Observer<MidiMessage> for Synth {
    fn next(&mut self, mm: midi::MidiMessage) {

        if mm.noteon() {
            self.note_queue.push_front(mm);
            // self.notifyevent_noteon(mm.f0(), mm.vel())
            self.note.update((mm.f0(), mm.vel()));
        } else if mm.noteoff() {
            // check if this note (identified by number/frequency) is queued
            let result = self.note_queue.iter().position(|x| x.note_number() == mm.note_number());
            match result {
                Some(i) => {
                    self.note_queue.remove(i);
                    if i == 0 {
                        self.noteoff.update(types::noteoff);
                        if self.note_queue.len() > 0 {
                            let mm = &self.note_queue[0].clone();
                            // self.notifyevent_noteon(mm.f0(), mm.vel())
                            self.note.update((mm.f0(), mm.vel()));
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
}


impl Observer<types::fs> for Synth {
    fn next(&mut self, fs: types::fs) {
        self.fs.update(fs);
    }
}

impl Synth {
    pub fn next_blit(&mut self, blit: bool) {
        self.notifyevent_blit(blit)
    }
    pub fn next_postfilter(&mut self, pf: bool) {
        self.notifyevent_postfilter(pf);
    }
    pub fn get_amp(&mut self) -> f32 {
        self.voice.get_amp()
    }
    pub fn cleanup(&mut self) {
        self.voice.cleanup();
    }
    // fn next_noteoff(&mut self) {
    //     self.notifyevent_noteoff()
    // }
//     pub fn notifyevent_noteoff(&mut self) {
//         for o in &mut self.observers {
//             o.next_noteoff()
//         }
//     }
    pub fn notifyevent_blit(&mut self, blit: bool) {
        // for o in &mut self.observers {
            // self.voice.next_blit(blit)
        // }
    }
    pub fn notifyevent_postfilter(&mut self, pf: bool) {
        // for o in &mut self.observers {
        //     o.next_postfilter(pf)
        // }
    }
}