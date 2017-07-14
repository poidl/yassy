use types;
use std::collections::VecDeque;
use observer;
use observer::*;
use midi;
use midi::*;
use std::io;
use std::io::Write;

const NOSC: usize = 4;

pub struct MidiMessageProcessor<'a> {
    pub note_stack: Vec<[u8;3]>,
    // pub nvoices: usize,
    // pub unison: bool,
    pub maxnotes: usize,
    pub notes: [Option<[u8;3]>; NOSC],
    pub notes_old: [Option<[u8;3]>; NOSC],
    pub noteon: Observable<'a, types::noteon>,
    pub noteoff: Observable<'a, types::noteoff>,
    // pub noteon: Vec<Observable<'a, types::noteon>>,
    // pub noteoff: Vec<Observable<'a, types::noteoff>>,
}

// Observes MidiMessages, and emits noteon and noteoff observables
impl<'a> MidiMessageProcessor<'a> {
    pub fn new() -> MidiMessageProcessor<'a> {
    let p = MidiMessageProcessor { 
        note_stack: Vec::with_capacity(10),
        // nvoices: 1,
        // unison: false,
        maxnotes: 1,
        notes: [None; NOSC],
        notes_old: [None; NOSC],
        noteon: Observable::new(types::noteon(0f32,0f32)),
        noteoff: Observable::new(types::noteoff(0u8)),

        // noteon: vec![
        //     Observable::new(types::noteon(0f32,0f32)),
        //     Observable::new(types::noteon(0f32,0f32)),
        //     Observable::new(types::noteon(0f32,0f32)),
        //     Observable::new(types::noteon(0f32,0f32)),
        //     ],
        // noteoff: vec![
        //     Observable::new(types::noteoff(0u8)),
        //     Observable::new(types::noteoff(0u8)),
        //     Observable::new(types::noteoff(0u8)),
        //     Observable::new(types::noteoff(0u8)),
        //     ],
        };
    p
    }
    fn update_notes(&mut self) {
        // set the note thats not played any more to None
        let mut i_remove: Option<usize> = None;
        for (i, n) in self.notes.iter().enumerate() {
            match *n {
                Some(note) => {
                    let mut iter = self.note_stack.iter().rev().take(self.maxnotes);        
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
                // println!("i_remove:****{}", i);
                self.notes[i] = None
            }
            _ => {}
        }
        let mut iter = self.note_stack.iter().rev().take(self.maxnotes);
        for mm in iter {
            // is the note already assigned to a synth
            let result = self.notes.iter().position(|x|
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
                    let result = self.notes.iter().position(|x|
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
                            self.notes[i] = Some(*mm);
                        }
                        _ => {
                            // TODO: find out why this sometimes panics (XRUNS, skipped midi note-off events?)
                            panic!{"No free slot."}
                            }
                    }
                }
            }
        }
        // println!("notes: ");
        // for i in self.notes.iter() {
        //     match *i {
        //         Some(note) => {
        //             println!("i: {} ", note.note_number())
        //         }
        //         _ =>  println!("i: None ")
        //     }
            
        // }
        for (i, n) in self.notes.iter().enumerate() {
            match *n {
                Some(note) => {
                    match self.notes_old[i] {
                        Some(note2) => {
                            if note.note_number() != note2.note_number() {
                                // self.noteon[i].update(types::noteon(note.f0(), note.vel()))
                                self.noteon.update(types::noteon(note.f0(), note.vel()))
                            }

                        }
                        _ => {
                            // self.noteon[i].update(types::noteon(note.f0(), note.vel()))
                            self.noteon.update(types::noteon(note.f0(), note.vel()))
                            }
                    }
                }
                _ => {
                    match self.notes_old[i] {
                        Some(note) => {
                            // self.noteoff[i].update(types::noteoff(note.note_number()))
                            self.noteoff.update(types::noteoff(note.note_number()))
                        }
                        _ => {}
                    }
                }
            }
            
        }
        self.notes_old = self.notes;
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

        self.update_notes();

        print!("NOTES: ");
        for i in self.notes.iter() {
            match *i {
                Some(mm) => {
                    print!{" {},", mm.note_number()}
                }
                _ => {}
            }
        }
        io::stdout().flush().unwrap();
        println!("");
    }
}

impl<'a> Observer<types::maxnotes> for MidiMessageProcessor<'a> {
    fn next(&mut self, m: types::maxnotes) {
        self.maxnotes = m.0 as usize

    }
}

// impl<'a> Observer<types::note2osc> for MidiMessageProcessor {
//     fn next(&mut self, p: types::note2osc) {
//         while self.noteon.observers.len > 0 {
//             self.noteon.observers.pop()
//         }

//     }
// }
// impl<'a> Observer<types::polyphony> for MidiMessageProcessor {
//     fn next(&mut self, p: types::polyphony) {
//         if p.0 {
//             if self.unison & (self.nvoices > 1) {
//                 self.maxnotes = self.nvoices / 2;
//                 // println!("Polyphony on with union, maxnotes = {}", self.maxnotes);
//                 return
//             }
//             self.maxnotes = self.nvoices;
//             // println!("Polyphony on, union off, maxnotes = {}", self.maxnotes);
//             return
//         }
//         self.maxnotes = 1;
//         // println!("Polyphony off");
//     }
// }

// impl<'a> Observer<types::nvoices> for MidiMessageProcessor {
//     fn next(&mut self, p: types::nvoices) {
//         self.nvoices = p.0;
//         // println!("Nvoices = {}", self.nvoices);
//     }
// }

// impl<'a> Observer<types::unison> for MidiMessageProcessor {
//     fn next(&mut self, p: types::unison) {
//         if p.0 {
//             self.unison = true;
//             return
//         }
//         self.unison = false
//         // println!("Unison = {}", self.unison);
//     }
// }

// impl<'a> Observer<types::unison> for MidiMessageProcessor {
//     fn next(&mut self, p: types::) {
//         self.nvoices = p.0;
//     }
// }