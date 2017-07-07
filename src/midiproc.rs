use types;
use std::collections::VecDeque;
use observer;
use observer::*;
use midi;
use midi::*;

pub struct MidiMessageProcessor<'a> {
    pub note_stack: Vec<[u8;3]>,
    pub playing: VecDeque<[u8;3]>,
    pub released: Vec<u8>,
    pub synths: [Option<[u8;3]>; 3],
    pub synths_old: [Option<[u8;3]>; 3],
    pub noteon: Vec<Observable<'a, types::noteon>>,
    pub noteoff: Vec<Observable<'a, types::noteoff>>,
}

const NN: usize = 2;

// Observes MidiMessages, and emits noteon and noteoff observables
impl<'a> MidiMessageProcessor<'a> {
    pub fn new() -> MidiMessageProcessor<'a> {
    let p = MidiMessageProcessor { 
        note_stack: Vec::with_capacity(10),
        playing: VecDeque::with_capacity(NN),
        released: Vec::with_capacity(NN),
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
        };
    p
    }
    fn update_synths(&mut self) {
        // set the note thats not played any more to None
        let mut i_remove: Option<usize> = None;
        for (i, n) in self.synths.iter().enumerate() {
            match *n {
                Some(note) => {
                    let result = self.playing.iter().position(|x| x.note_number() == note.note_number());
                    match result {
                        Some(j) => {}
                        _ => {
                            i_remove = Some(i);
                            break
                        }
                    }
                }
                _ => {}
            }
        
        }
        match i_remove {
            Some(i) => {
                println!("i_remove:****{}", i);
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
                            println!("self.playing.len(): {}", self.playing.len());
                            // TODO: find out why this sometimes panics (XRUNS, skipped midi note-off events?)
                            panic!{"No free slot."}
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
                    match self.synths_old[i] {
                        Some(note2) => {
                            if note.note_number() != note2.note_number() {
                                self.noteon[i].update(types::noteon(note.f0(), note.vel()))
                            }

                        }
                        _ => {self.noteon[i].update(types::noteon(note.f0(), note.vel()))}
                    }
                }
                _ => {
                    match self.synths_old[i] {
                        Some(note) => {
                            self.noteoff[i].update(types::noteoff(note.note_number()))
                        }
                        _ => {}
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