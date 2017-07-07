use types;
use std::collections::VecDeque;
use observer;
use observer::*;
use midi;
use midi::*;

pub struct MidiMessageProcessor<'a> {
    pub note_stack: Vec<[u8;3]>,
    pub synths: [Option<[u8;3]>; 3],
    pub synths_old: [Option<[u8;3]>; 3],
    pub noteon: Vec<Observable<'a, types::noteon>>,
    pub noteoff: Vec<Observable<'a, types::noteoff>>,
}

const NN: usize = 3;

// Observes MidiMessages, and emits noteon and noteoff observables
impl<'a> MidiMessageProcessor<'a> {
    pub fn new() -> MidiMessageProcessor<'a> {
    let p = MidiMessageProcessor { 
        note_stack: Vec::with_capacity(10),
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
                    let mut iter = self.note_stack.iter().rev().take(NN);        
                    let result = iter.position(|x| x.note_number() == note.note_number());
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
        let mut iter = self.note_stack.iter().rev().take(NN);
        for mm in iter {
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
        } else if mm.noteoff() {
            // check if this note (identified by number/frequency) is queued
            let result = self.note_stack.iter().position(|x| x.note_number() == mm.note_number());
            match result {
                Some(i) => {
                    self.note_stack.remove(i);                        
                }
                _ => {}
            }
        }

        self.update_synths()
    }
}